//! [DonEditor] 실제 TCP에서 MCP 요청 취소·연결 종료·서버 중지와 크기 제한을 검사한다.

use don_protocol_tests::{
    limits::{MAX_MESSAGE, MAX_RESULT},
    protocol::Backend,
    transport::{Connection, Server},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, Shutdown, TcpStream},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

/// 테스트 실패가 끝없는 승인 대기나 소켓 대기로 남지 않게 하는 시간 상한이다.
const TEST_WAIT: Duration = Duration::from_secs(10);
/// 생성·삭제할 수 있는 테스트 전용 디렉터리 접두사이다.
const DIRECTORY_PREFIX: &str = "don-mcp-transport-";

/// 이 테스트가 만든 단일 임시 디렉터리만 정리하는 가드이다.
struct TestDirectory {
    base: PathBuf,
    path: PathBuf,
}
impl TestDirectory {
    /// 임시 경로를 정규화한 뒤 충돌하지 않는 폴더를 만든다.
    fn new() -> Self {
        let base = std::env::temp_dir().canonicalize().unwrap();
        let path = base.join(format!("{DIRECTORY_PREFIX}{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self { base, path }
    }

    /// 서버가 기록할 연결 파일은 테스트 전용 하위 디렉터리에 둔다.
    fn connection_path(&self) -> PathBuf {
        self.path.join("mcp").join("connection.json")
    }
}
impl Drop for TestDirectory {
    /// 부모·파일명을 검증한 절대 경로만 재귀 삭제한다.
    fn drop(&mut self) {
        if self.path.parent() == Some(self.base.as_path())
            && self
                .path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(DIRECTORY_PREFIX))
        {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// 승인창이 열려 사용자의 응답을 기다리는 도구를 모사한다.
struct BlockingBackend {
    entered: mpsc::Sender<Arc<AtomicBool>>,
    finished: mpsc::Sender<bool>,
}
impl Backend for BlockingBackend {
    /// 전송 계층이 취소 가능한 진입점을 사용하지 않으면 명확하게 실패한다.
    fn call(&self, _: &str, _: Value) -> Result<Value, String> {
        Err("취소 토큰이 없는 백엔드 진입점입니다".into())
    }

    /// 테스트는 도구 호출 자체만 사용하므로 별도 명세는 필요하지 않다.
    fn tools(&self) -> Vec<Value> {
        vec![]
    }

    /// 실제 전송 계층이 전달한 플래그가 취소될 때까지 대기한다.
    fn call_cancellable(&self, _: &str, _: Value, cancelled: Arc<AtomicBool>) -> Result<Value, String> {
        self.entered
            .send(cancelled.clone())
            .map_err(|error| error.to_string())?;
        let deadline = Instant::now() + TEST_WAIT;
        // 테스트의 승인 대기는 취소를 관찰하면 즉시 종료하며 실패 시에도 유한 시간만 기다린다.
        while !cancelled.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        let observed = cancelled.load(Ordering::Acquire);
        let _ = self.finished.send(observed);
        Err(if observed {
            "취소됨"
        } else {
            "취소 전달 시간 초과"
        }
        .into())
    }
}

/// 실행 중인 백엔드로 요청이 도착했는지와 취소를 관찰했는지 전달하는 수신부이다.
type BlockingFixture = (
    TestDirectory,
    Server,
    mpsc::Receiver<Arc<AtomicBool>>,
    mpsc::Receiver<bool>,
);

/// 테스트마다 독립적인 서버와 승인 대기 상태를 만든다.
fn blocking_fixture() -> BlockingFixture {
    let directory = TestDirectory::new();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let backend = Arc::new(BlockingBackend {
        entered: entered_tx,
        finished: finished_tx,
    });
    let server = Server::start(0, directory.connection_path(), backend).unwrap();
    (directory, server, entered_rx, finished_rx)
}

/// 검증에 필요한 HTTP 상태·헤더·JSON-RPC 본문이다.
struct Response {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Option<Value>,
}

/// 응답 대기와 별개로 요청을 보내 취소·삭제 요청을 다른 TCP 연결로 전달할 수 있게 한다.
fn open_request(connection: &Connection, method: &str, session: Option<&str>, body: Option<&Value>) -> TcpStream {
    let mut stream = TcpStream::connect_timeout(&(Ipv4Addr::LOCALHOST, connection.port).into(), TEST_WAIT).unwrap();
    stream.set_read_timeout(Some(TEST_WAIT)).unwrap();
    stream.set_write_timeout(Some(TEST_WAIT)).unwrap();
    let body = body.map(Value::to_string).unwrap_or_default();
    let session = session
        .map(|value| format!("MCP-Session-Id: {value}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "{method} /mcp HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\n{session}Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-06-18\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        connection.port,
        connection.token,
        body.len()
    )
    .unwrap();
    stream
}

/// 응답 길이를 검사한 뒤 해당 바이트만 읽어 연결 종료 타이밍에 의존하지 않는다.
fn read_response(stream: TcpStream) -> Response {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let status = line.split_whitespace().nth(1).unwrap().parse().unwrap();
    let mut headers = BTreeMap::new();
    // HTTP 헤더를 읽어 상태 및 세션 발급을 검증한다.
    loop {
        line.clear();
        assert_ne!(reader.read_line(&mut line).unwrap(), 0, "HTTP 헤더 중단");
        if line == "\r\n" {
            break;
        }
        let (name, value) = line.trim_end().split_once(':').unwrap();
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }
    let length: usize = headers.get("content-length").unwrap().parse().unwrap();
    assert!(length <= MAX_MESSAGE, "서버가 전송 크기 제한을 넘었습니다: {length}");
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).unwrap();
    let body = (!bytes.is_empty()).then(|| serde_json::from_slice(&bytes).unwrap());
    Response { status, headers, body }
}

/// 동기적으로 완료되는 요청을 보낸다.
fn request(connection: &Connection, method: &str, session: Option<&str>, body: Option<&Value>) -> Response {
    read_response(open_request(connection, method, session, body))
}

/// MCP 초기화 응답의 세션 ID를 이후 요청에 그대로 사용한다.
fn initialize(connection: &Connection) -> String {
    let response = request(
        connection,
        "POST",
        None,
        Some(&json!({
            "jsonrpc":"2.0", "id":"initialize", "method":"initialize",
            "params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"transport-tests","version":"1"}}
        })),
    );
    assert_eq!(response.status, 200);
    assert_eq!(response.body.unwrap()["result"]["protocolVersion"], "2025-06-18");
    response.headers.get("mcp-session-id").unwrap().clone()
}

/// 승인 대기 도구를 시작하고 취소 전에는 아직 실행 중임을 확인한다.
fn pending_request(
    connection: &Connection,
    session: &str,
    entered: &mpsc::Receiver<Arc<AtomicBool>>,
) -> (TcpStream, Arc<AtomicBool>) {
    let stream = open_request(
        connection,
        "POST",
        Some(session),
        Some(&json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"wait","arguments":{}}})),
    );
    let cancelled = entered.recv_timeout(TEST_WAIT).unwrap();
    assert!(!cancelled.load(Ordering::Acquire));
    (stream, cancelled)
}

/// 취소된 요청은 일반 성공 결과 대신 취소 오류를 반환해야 한다.
fn assert_cancelled_response(stream: TcpStream) {
    let response = read_response(stream);
    assert_eq!(response.status, 200);
    assert_eq!(response.body.unwrap()["error"]["code"], -32800);
}

/// 취소 알림을 별도 연결로 보내 승인 대기 작업까지 취소가 전달되는지 확인한다.
#[test]
fn cancellation_notification_reaches_pending_backend() {
    let (_directory, server, entered, finished) = blocking_fixture();
    let session = initialize(&server.connection);
    let (pending, cancelled) = pending_request(&server.connection, &session, &entered);
    let notification = request(
        &server.connection,
        "POST",
        Some(&session),
        Some(&json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":7,"reason":"test"}})),
    );
    assert_eq!(notification.status, 202);
    assert!(
        finished.recv_timeout(TEST_WAIT).unwrap(),
        "백엔드에 취소가 전달되지 않았습니다"
    );
    assert!(cancelled.load(Ordering::Acquire));
    assert_cancelled_response(pending);
    let ping = request(
        &server.connection,
        "POST",
        Some(&session),
        Some(&json!({"jsonrpc":"2.0","id":8,"method":"ping"})),
    );
    assert_eq!(ping.status, 200, "요청 취소가 세션 전체를 삭제하면 안 됩니다");
}

/// 클라이언트 소켓을 닫으면 알림을 보내지 않아도 승인 대기 작업을 취소한다.
#[test]
fn tcp_disconnect_cancels_pending_backend() {
    let (_directory, server, entered, finished) = blocking_fixture();
    let session = initialize(&server.connection);
    let (pending, cancelled) = pending_request(&server.connection, &session, &entered);
    pending.shutdown(Shutdown::Both).unwrap();
    drop(pending);
    assert!(
        finished.recv_timeout(TEST_WAIT).unwrap(),
        "TCP 종료가 취소로 전달되지 않았습니다"
    );
    assert!(cancelled.load(Ordering::Acquire));
}

/// 세션 DELETE는 모든 승인 대기 요청을 취소하고 이전 세션 ID를 무효화한다.
#[test]
fn deleting_session_cancels_backend_and_invalidates_session() {
    let (_directory, server, entered, finished) = blocking_fixture();
    let session = initialize(&server.connection);
    let (pending, cancelled) = pending_request(&server.connection, &session, &entered);
    let deleted = request(&server.connection, "DELETE", Some(&session), None);
    assert_eq!(deleted.status, 204);
    assert!(deleted.body.is_none());
    assert!(finished.recv_timeout(TEST_WAIT).unwrap());
    assert!(cancelled.load(Ordering::Acquire));
    assert_cancelled_response(pending);
    let ping = request(
        &server.connection,
        "POST",
        Some(&session),
        Some(&json!({"jsonrpc":"2.0","id":8,"method":"ping"})),
    );
    assert_eq!(ping.status, 404);
}

/// 서버 중지는 승인 대기를 취소하고 listener를 닫아 같은 포트로 곧바로 재시작할 수 있게 한다.
#[test]
fn dropping_server_cancels_backend_and_releases_fixed_port() {
    let (directory, server, entered, finished) = blocking_fixture();
    let connection = server.connection.clone();
    let session = initialize(&connection);
    let (pending, cancelled) = pending_request(&connection, &session, &entered);
    drop(server);
    assert!(finished.recv_timeout(TEST_WAIT).unwrap());
    assert!(cancelled.load(Ordering::Acquire));
    assert_cancelled_response(pending);
    assert!(!directory.connection_path().exists());
    let (entered_tx, _) = mpsc::channel();
    let (finished_tx, _) = mpsc::channel();
    let restarted = Server::start(
        connection.port,
        directory.connection_path(),
        Arc::new(BlockingBackend {
            entered: entered_tx,
            finished: finished_tx,
        }),
    )
    .expect("listener 종료 후 같은 고정 포트를 즉시 다시 열 수 있어야 합니다");
    assert_eq!(restarted.connection.port, connection.port);
    assert_ne!(restarted.connection.token, connection.token);
    initialize(&restarted.connection);
}

/// 결과 및 명세의 크기 상한을 넘기는 가짜 백엔드이다.
struct LargeBackend;
impl Backend for LargeBackend {
    /// 중복 인코딩 전에 검사할 4MiB 초과 도구 결과를 만든다.
    fn call(&self, _: &str, _: Value) -> Result<Value, String> {
        Ok(json!({"text":"x".repeat(MAX_RESULT)}))
    }

    /// 도구 목록처럼 protocol 결과 검사 밖의 출력도 HTTP 상한을 적용해야 한다.
    fn tools(&self) -> Vec<Value> {
        vec![json!({"name":"large","description":"x".repeat(MAX_MESSAGE + 1),"inputSchema":{"type":"object"}})]
    }
}

/// 대용량 결과는 전송을 끊거나 과도한 응답을 보내지 않고 명확한 오류로 반환한다.
#[test]
fn oversized_tool_and_http_results_return_bounded_errors() {
    let directory = TestDirectory::new();
    let server = Server::start(0, directory.connection_path(), Arc::new(LargeBackend)).unwrap();
    let session = initialize(&server.connection);
    let result = request(
        &server.connection,
        "POST",
        Some(&session),
        Some(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"large","arguments":{}}})),
    );
    assert_eq!(result.status, 200);
    assert_eq!(result.body.unwrap()["result"]["isError"], true);
    let definitions = request(
        &server.connection,
        "POST",
        Some(&session),
        Some(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"})),
    );
    assert_eq!(definitions.status, 200);
    assert_eq!(definitions.body.unwrap()["error"]["code"], -32603);
}

/// icacls 저장 파일은 Windows에서 UTF-16LE이고 그 밖에는 UTF-8일 수 있다.
#[cfg(windows)]
fn saved_acl(path: &std::path::Path, output: &std::path::Path) -> String {
    use std::os::windows::process::CommandExt as _;
    let result = std::process::Command::new("icacls.exe")
        .arg(path)
        .arg("/save")
        .arg(output)
        .arg("/q")
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(result.status.success(), "icacls /save 실패");
    let bytes = fs::read(output).unwrap();
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.get(1) == Some(&0) {
        // icacls 버전에 따라 UTF-16LE 저장 파일에 BOM이 없을 수도 있다.
        let start = if bytes.starts_with(&[0xff, 0xfe]) { 2 } else { 0 };
        let words: Vec<_> = bytes[start..]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16(&words).unwrap()
    } else {
        String::from_utf8(bytes).unwrap()
    }
}

/// SDDL 언어와 무관하게 Everyone SID에 대한 명시적 허용 ACE를 찾는다.
#[cfg(windows)]
fn explicit_everyone_allow(sddl: &str) -> bool {
    sddl.split('(').any(|ace| {
        let fields: Vec<_> = ace.split(')').next().unwrap_or_default().split(';').collect();
        fields.len() == 6 && fields[0] == "A" && !fields[1].contains("ID") && matches!(fields[5], "WD" | "S-1-1-0")
    })
}

/// 본인 임시 폴더에만 ACE를 추가한 뒤 서버가 이를 제거하는지 확인하는 선택 검사이다.
#[cfg(windows)]
#[test]
#[ignore = "DONEDITOR_TEST_WINDOWS_ACL=1로 명시한 본인 임시 폴더 ACL 검사만 실행"]
fn windows_start_resets_existing_explicit_everyone_acl() {
    use std::os::windows::process::CommandExt as _;
    assert_eq!(
        std::env::var("DONEDITOR_TEST_WINDOWS_ACL").as_deref(),
        Ok("1"),
        "선택 ACL 검사는 환경 변수로 명시적으로 켜야 합니다"
    );
    let directory = TestDirectory::new();
    let connection_path = directory.connection_path();
    let private = connection_path.parent().unwrap();
    fs::create_dir(private).unwrap();
    let grant = std::process::Command::new("icacls.exe")
        .arg(private)
        .args(["/grant", "*S-1-1-0:(OI)(CI)R"])
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(grant.status.success());
    let before = saved_acl(private, &directory.path.join("before.acl"));
    assert!(
        explicit_everyone_allow(&before),
        "검사 전 명시적 Everyone ACE를 확인해야 합니다: {before:?}"
    );
    let server = Server::start(0, connection_path.clone(), Arc::new(LargeBackend)).unwrap();
    assert!(
        !explicit_everyone_allow(&saved_acl(private, &directory.path.join("after.acl"))),
        "서버가 기존 명시적 Everyone ACE를 제거하지 않았습니다"
    );
    assert!(!explicit_everyone_allow(&saved_acl(
        &connection_path,
        &directory.path.join("file.acl")
    )));
    drop(server);
}
