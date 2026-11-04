//! [DonEditor] stdio JSON-RPC를 실행 중인 편집기의 루프백 MCP HTTP로 중계한다.
use anyhow::{Context as _, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread,
    time::Duration,
};

/// 연결 파일에서만 읽는 인증 정보. 토큰을 로그에 기록하지 않는다.
#[derive(Deserialize)]
struct Connection {
    port: u16,
    token: String,
}
/// stdio 입력과 HTTP 응답의 크기 상한.
const MAX_MESSAGE: u64 = 16 * 1024 * 1024;
/// stdin을 막지 않고 동시에 중계할 일반 요청 수.
const MAX_ACTIVE: usize = 8;
/// 취소 알림은 일반 요청과 별도 작업자로 전달하되 대기열 크기도 제한한다.
const MAX_NOTIFICATIONS: usize = 16;

/// 초기화한 서버와 세션을 함께 보관하여 토큰이 다른 서버에 세션을 보내지 않는다.
struct Session {
    connection: Connection,
    id: String,
}

/// HTTP 초기화 응답의 세션 헤더와 JSON 본문.
struct HttpResponse {
    session_id: Option<String>,
    body: Option<Value>,
}

/// 여러 작업자의 stdout 프레임을 한 줄 단위로 직렬화한다.
type Output = Arc<Mutex<std::io::Stdout>>;
/// 알림은 제한된 전용 대기열로 전달하여 요청 8개가 대기해도 취소할 수 있다.
type Notification = (Arc<Session>, String);

/// 정상 종료·오류·스레드 생성 실패 모두에서 활성 요청 허가를 반환한다.
struct ActiveRequest(Arc<AtomicUsize>);
impl Drop for ActiveRequest {
    /// 작업자가 사라지면 활성 요청 수를 줄인다.
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// 사용자 데이터 디렉터리와 동일한 플랫폼별 기본 경로를 구한다.
fn default_connection_path() -> PathBuf {
    if let Some(path) = std::env::var_os("DONEDITOR_MCP_CONNECTION") {
        return PathBuf::from(path);
    }
    #[cfg(windows)]
    {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("DonEditor/mcp/connection.json")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("Library/Application Support/DonEditor/mcp/connection.json")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        PathBuf::from(std::env::var_os("XDG_DATA_HOME").unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".local/share")
                .into_os_string()
        }))
        .join("doneditor/mcp/connection.json")
    }
}

/// 연결 파일도 작은 크기로 제한하고 헤더에 사용할 인증 정보의 형식을 검사한다.
fn load_connection(path: &Path) -> Result<Connection> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(4097).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 4096, "MCP 연결 파일이 너무 큽니다");
    let connection: Connection = serde_json::from_slice(&bytes)?;
    ensure!(
        connection.port != 0 && connection.token.len() == 64 && connection.token.bytes().all(|b| b.is_ascii_hexdigit()),
        "MCP 연결 정보가 올바르지 않습니다"
    );
    Ok(connection)
}

/// 한 HTTP 요청을 보내며 초기화 뒤에는 세션 헤더를 반드시 포함한다.
fn forward(
    connection: &Connection,
    session_id: Option<&str>,
    method: &str,
    message: &str,
    timeout: Duration,
) -> Result<HttpResponse> {
    let mut stream =
        TcpStream::connect_timeout(&(Ipv4Addr::LOCALHOST, connection.port).into(), Duration::from_secs(5))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(Duration::from_secs(15)))?;
    let session_header = session_id
        .map(|id| format!("MCP-Session-Id: {id}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "{method} /mcp HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\n{session_header}Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-06-18\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        connection.port,
        connection.token,
        message.len(),
        message
    )?;
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.by_ref().take(16385).read_line(&mut status)?;
    let mut count = status.len();
    ensure!(
        count <= 16384 && status.ends_with('\n'),
        "HTTP 상태 줄이 올바르지 않습니다"
    );
    let mut parts = status.split_whitespace();
    ensure!(
        parts.next().is_some_and(|version| version.starts_with("HTTP/1.")),
        "HTTP 응답이 올바르지 않습니다"
    );
    let status = parts.next().unwrap_or_default().to_owned();
    let mut length = None;
    let mut response_session = None;
    // 작은 헤더만 허용하고 본문 길이를 엄격하게 검사한다.
    loop {
        let mut line = String::new();
        reader.by_ref().take(16385).read_line(&mut line)?;
        count += line.len();
        ensure!(count <= 16384, "HTTP 헤더가 너무 큽니다");
        if line == "\r\n" {
            break;
        }
        ensure!(!line.is_empty(), "HTTP 헤더가 중단되었습니다");
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                ensure!(length.is_none(), "HTTP 응답 길이가 중복되었습니다");
                length = Some(value.trim().parse::<u64>()?);
            } else if name.eq_ignore_ascii_case("mcp-session-id") {
                ensure!(response_session.is_none(), "MCP 세션 헤더가 중복되었습니다");
                response_session = Some(uuid::Uuid::parse_str(value.trim())?.to_string());
            } else if name.eq_ignore_ascii_case("transfer-encoding") {
                anyhow::bail!("청크 HTTP 응답은 지원하지 않습니다");
            }
        }
    }
    if status == "202" || (method == "DELETE" && status == "204") {
        return Ok(HttpResponse {
            session_id: response_session,
            body: None,
        });
    }
    ensure!(status == "200", "MCP HTTP 오류: {status}");
    let length = length.context("HTTP 응답 길이가 없습니다")?;
    ensure!(length <= MAX_MESSAGE, "MCP 응답이 너무 큽니다");
    let mut body = vec![0; length as usize];
    reader.read_exact(&mut body)?;
    Ok(HttpResponse {
        session_id: response_session,
        body: Some(serde_json::from_slice(&body)?),
    })
}

/// JSON-RPC 프레임 외의 진단·연결 정보는 stdout으로 보내지 않는다.
fn reply(output: &Output, response: &Value) -> Result<()> {
    let mut output = output
        .lock()
        .map_err(|_| anyhow::anyhow!("MCP 출력 잠금을 사용할 수 없습니다"))?;
    writeln!(output, "{response}")?;
    output.flush()?;
    Ok(())
}

/// 중계 오류는 고정 문구로 표현해 연결 파일과 응답의 비밀 값이 노출되지 않게 한다.
fn rpc_error(output: &Output, id: &Value, code: i32, message: &str) -> Result<()> {
    reply(
        output,
        &json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}),
    )
}

/// 세션을 삭제하여 EOF·입력 오류·재초기화 시 승인 대기 요청도 취소한다.
fn end_session(session: &Session) {
    if forward(
        &session.connection,
        Some(&session.id),
        "DELETE",
        "",
        Duration::from_secs(5),
    )
    .is_err()
    {
        eprintln!("don-mcp: MCP 세션 종료 요청을 전달하지 못했습니다");
    }
}

/// 일반 요청과 독립적인 작업자가 초기화 완료·취소 알림을 계속 전달한다.
fn notification_worker() -> Result<SyncSender<Notification>> {
    let (sender, receiver) = sync_channel::<Notification>(MAX_NOTIFICATIONS);
    thread::Builder::new()
        .name("don-mcp-notifications".into())
        .spawn(move || {
            // 알림을 순서대로 보내되 stdin과 일반 요청 작업자는 기다리게 하지 않는다.
            while let Ok((session, message)) = receiver.recv() {
                if forward(
                    &session.connection,
                    Some(&session.id),
                    "POST",
                    &message,
                    Duration::from_secs(5),
                )
                .is_err()
                {
                    eprintln!("don-mcp: MCP 알림을 전달하지 못했습니다");
                }
            }
        })?;
    Ok(sender)
}

/// 초기화만 동기로 처리하고 후속 요청은 제한된 작업자로 병렬 중계한다.
fn relay(
    input: &mut impl BufRead,
    output: &Output,
    path: &Path,
    session: &mut Option<Arc<Session>>,
    notifications: &SyncSender<Notification>,
    active: &Arc<AtomicUsize>,
) -> Result<()> {
    // 응답을 기다리는 동안에도 stdin의 취소 알림과 EOF를 읽을 수 있어야 한다.
    loop {
        let mut message = String::new();
        let read = input.by_ref().take(MAX_MESSAGE + 1).read_line(&mut message)?;
        if read == 0 {
            break;
        }
        ensure!(message.len() as u64 <= MAX_MESSAGE, "MCP 입력이 너무 큽니다");
        if message.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&message) {
            Ok(request) => request,
            Err(_) => {
                rpc_error(output, &Value::Null, -32700, "Parse error")?;
                continue;
            }
        };
        let id = request.get("id").cloned();
        if !request.is_object()
            || request["jsonrpc"] != "2.0"
            || !request["method"].is_string()
            || id
                .as_ref()
                .is_some_and(|id| !id.is_null() && !id.is_string() && !id.is_number())
        {
            rpc_error(output, &Value::Null, -32600, "Invalid Request")?;
            continue;
        }
        let method = request["method"].as_str().unwrap();
        if method == "initialize" {
            let Some(id) = id else {
                continue;
            };
            if active.load(Ordering::Acquire) != 0 {
                rpc_error(output, &id, -32000, "Cannot initialize while requests are active")?;
                continue;
            }
            if let Some(previous) = session.take() {
                end_session(&previous);
            }
            let result = (|| {
                let connection = load_connection(path)?;
                let response = forward(&connection, None, "POST", message.trim(), Duration::from_secs(15))?;
                let body = response.body.context("초기화 응답 본문이 없습니다")?;
                if body.get("result").is_some() {
                    *session = Some(Arc::new(Session {
                        connection,
                        id: response.session_id.context("초기화 응답에 MCP 세션이 없습니다")?,
                    }));
                }
                Ok::<_, anyhow::Error>(body)
            })();
            match result {
                Ok(body) => reply(output, &body)?,
                Err(_) => rpc_error(
                    output,
                    &id,
                    -32603,
                    "Cannot initialize MCP session; enable the editor server",
                )?,
            }
            continue;
        }
        let Some(session) = session.as_ref() else {
            if let Some(id) = id {
                let (code, detail) = if method == "server/discover" {
                    (-32601, "Method not found")
                } else {
                    (-32002, "MCP session is not initialized")
                };
                rpc_error(output, &id, code, detail)?;
            }
            continue;
        };
        let Some(id) = id else {
            if notifications
                .try_send((session.clone(), message.trim().to_owned()))
                .is_err()
            {
                eprintln!("don-mcp: MCP 알림 대기열이 가득 찼습니다");
            }
            continue;
        };
        if active
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_ACTIVE).then_some(count + 1)
            })
            .is_err()
        {
            rpc_error(output, &id, -32000, "Too many active MCP requests (maximum 8)")?;
            continue;
        }
        let permit = ActiveRequest(active.clone());
        let session = session.clone();
        let output = output.clone();
        let request_id = id.clone();
        let worker_output = output.clone();
        if thread::Builder::new()
            .name("don-mcp-request".into())
            .spawn(move || {
                let _permit = permit;
                match forward(
                    &session.connection,
                    Some(&session.id),
                    "POST",
                    message.trim(),
                    Duration::from_secs(310),
                ) {
                    Ok(response) => {
                        if let Some(body) = response.body {
                            let _ = reply(&worker_output, &body);
                        }
                    }
                    Err(_) => {
                        let _ = rpc_error(&worker_output, &request_id, -32603, "MCP request forwarding failed");
                    }
                }
            })
            .is_err()
        {
            rpc_error(&output, &id, -32603, "Cannot start MCP request worker")?;
        }
    }
    Ok(())
}

/// stdio stdout에는 MCP 메시지만 출력하며 EOF에서는 실행 중인 세션을 취소한다.
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut path = default_connection_path();
    if let Some(argument) = args.next() {
        match argument.as_str() {
            "--connection" => {
                path = PathBuf::from(args.next().context("--connection 뒤에 파일 경로가 필요합니다")?);
                ensure!(args.next().is_none(), "추가 인자는 지원하지 않습니다");
            }
            "--help" | "-h" => {
                eprintln!("don-mcp [--connection <connection.json>]\n편집기에서 MCP 서버를 먼저 켜세요.");
                return Ok(());
            }
            _ => anyhow::bail!("지원하지 않는 옵션입니다"),
        }
    }
    let output = Arc::new(Mutex::new(std::io::stdout()));
    let notifications = notification_worker()?;
    let active = Arc::new(AtomicUsize::new(0));
    let mut session = None;
    let result = relay(
        &mut std::io::stdin().lock(),
        &output,
        &path,
        &mut session,
        &notifications,
        &active,
    );
    if let Some(session) = session {
        end_session(&session);
    }
    result
}
/// 오류를 종료 코드와 stderr로 전달한다.
fn main() {
    if let Err(error) = run() {
        eprintln!("don-mcp: {error}");
        std::process::exit(1);
    }
}
