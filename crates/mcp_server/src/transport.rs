//! [DonEditor] 루프백 전용·Bearer 인증 Streamable HTTP와 연결 정보 파일.
use crate::protocol::{self, Backend};
use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

/// 최대 JSON 요청 크기. 대용량 파일 전송도 무제한 메모리를 쓰지 않는다.
pub const MAX_BODY: usize = crate::limits::MAX_MESSAGE;
/// 토큰은 연결 정보 파일로만 전달하며 stdout·로그·명령줄에 출력하지 않는다.
#[derive(Clone, Serialize, Deserialize)]
pub struct Connection {
    pub port: u16,
    pub token: String,
    pub pid: u32,
}

/// 연결 정보의 기본 경로. --user-data-dir 설정도 반영한다.
pub fn connection_path() -> PathBuf {
    paths::data_dir().join("mcp").join("connection.json")
}

/// Unix 권한 또는 Windows 사용자 ACL로 토큰 디렉터리를 보호한다.
fn private_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    {
        // 콘솔 창을 띄우지 않고 현재 사용자의 SID를 조회한다.
        let who = run_windows_tool("whoami.exe", &["/user", "/fo", "csv", "/nh"])?;
        ensure!(who.status.success(), "사용자 SID 조회 실패");
        let text = String::from_utf8(who.stdout)?;
        let sid = text.trim().rsplit(',').next().unwrap_or_default().trim_matches('"');
        ensure!(
            sid.starts_with("S-1-") && sid.bytes().all(|b| b.is_ascii_digit() || b == b'S' || b == b'-'),
            "사용자 SID 형식 오류"
        );
        // 기존 명시적 ACE도 지운 뒤 현재 사용자만 허용하는 보호 디렉터리로 만든다.
        let reset = run_windows_tool("icacls.exe", &[path.as_os_str(), "/reset".as_ref()])?;
        ensure!(reset.status.success(), "MCP 토큰 디렉터리 권한 초기화 실패");
        let grant = format!("*{sid}:(OI)(CI)F");
        let acl = run_windows_tool(
            "icacls.exe",
            &[
                path.as_os_str(),
                "/inheritance:r".as_ref(),
                "/grant:r".as_ref(),
                grant.as_ref(),
            ],
        )?;
        ensure!(acl.status.success(), "MCP 토큰 디렉터리 권한 설정 실패");
    }
    Ok(())
}

/// Windows 의 짧은 보안 도구(`whoami`, `icacls`)를 콘솔 창 없이 실행하고 끝날 때까지 기다린다.
///
/// 프로젝트의 `disallowed_methods` 규칙은 `std::process::Command` 가 스레드를 오래 막을 수 있다는 이유로 쓰지 말라고 하지만,
/// 이 함수는 MCP 서버를 **사용자가 켤 때(기본값은 꺼짐) 한 번**, 연결 정보 폴더의 접근 권한을 정하려고 부르는
/// 100ms 안팎의 짧은 명령 3개에만 쓴다. 서버 시작은 설정 반영 흐름(`Server::start`)이 동기적으로 폴더 보호를 끝낸 뒤에
/// 토큰 파일을 만들어야 하는 구조(전송 시험 24개와 실사용 확인이 이 순서에 의존)라서, 시작 전체를 비동기로 바꾸는 대신
/// 허용하는 이유를 여기에 남긴다.
#[cfg(windows)]
#[allow(clippy::disallowed_methods)]
fn run_windows_tool<S: AsRef<std::ffi::OsStr>>(program: &str, args: &[S]) -> Result<std::process::Output> {
    use std::os::windows::process::CommandExt;
    // CREATE_NO_WINDOW: 콘솔 창을 띄우지 않는다
    Ok(std::process::Command::new(program)
        .args(args)
        .creation_flags(0x0800_0000)
        .output()?)
}

/// 서버의 수명과 토큰을 함께 관리한다.
pub struct Server {
    stop: Arc<AtomicBool>,
    pub connection: Connection,
    path: PathBuf,
    listener_thread: Option<thread::JoinHandle<()>>,
    sessions: Arc<crate::sessions::Sessions>,
}
impl Server {
    /// 수신 소켓을 연 뒤 보호된 파일에 연결 정보를 기록한다.
    pub fn start(port: u16, path: PathBuf, backend: Arc<dyn Backend + Send + Sync>) -> Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        let connection = Connection {
            port: listener.local_addr()?.port(),
            token: format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple()),
            pid: std::process::id(),
        };
        private_directory(path.parent().context("연결 파일 경로 오류")?)?;
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // 기존 실행 정보는 원자적 이름 변경으로 교체한다.
        let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(&connection)?)?;
        file.sync_all()?;
        drop(file);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        fs::rename(&temporary, &path)?;
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let details = connection.clone();
        let sessions = Arc::new(crate::sessions::Sessions::new());
        let thread_sessions = sessions.clone();
        let listener_thread = thread::Builder::new().name("don-mcp-http".into()).spawn(move || {
            let clients = Arc::new(AtomicUsize::new(0));
            // 최대 8개 UI 요청 외에 취소·세션 종료 요청을 받을 연결 여유를 둔다.
            while !stop_thread.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        if clients.load(Ordering::Relaxed) >= 16 {
                            let _ = respond(&mut stream, 503, None);
                            continue;
                        }
                        clients.fetch_add(1, Ordering::Relaxed);
                        let clients = clients.clone();
                        let backend = backend.clone();
                        let details = details.clone();
                        let stop = stop_thread.clone();
                        let sessions = thread_sessions.clone();
                        thread::spawn(move || {
                            let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
                            let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
                            let _ = serve(&mut stream, &details, backend.as_ref(), &stop, &sessions);
                            clients.fetch_sub(1, Ordering::Relaxed);
                        });
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(_) => break,
                }
            }
        })?;
        Ok(Self {
            stop,
            connection,
            path,
            listener_thread: Some(listener_thread),
            sessions,
        })
    }
}
impl Drop for Server {
    /// 이 서버가 남긴 연결 정보만 삭제하며 새 인스턴스의 파일은 보존한다.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.sessions.cancel_all();
        // 수신 소켓 종료까지만 기다려 같은 포트를 즉시 다시 사용할 수 있게 한다.
        if let Some(thread) = self.listener_thread.take() {
            let _ = thread.join();
        }
        if let Ok(bytes) = fs::read(&self.path)
            && serde_json::from_slice::<Connection>(&bytes).is_ok_and(|c| c.token == self.connection.token)
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// 길이와 모든 바이트를 확인해 토큰 비교의 조기 종료를 피한다.
fn token_matches(expected: &str, actual: &str) -> bool {
    let mut difference = expected.len() ^ actual.len();
    for (index, byte) in expected.bytes().enumerate() {
        difference |= usize::from(byte ^ actual.as_bytes().get(index).copied().unwrap_or(0));
    }
    difference == 0
}

/// 한 요청을 읽고 인증·Origin·Host·크기 제한을 검사한 뒤 처리한다.
fn serve(
    stream: &mut TcpStream,
    connection: &Connection,
    backend: &(dyn Backend + Send + Sync),
    stop: &AtomicBool,
    sessions: &crate::sessions::Sessions,
) -> Result<()> {
    let mut reader = BufReader::new(&mut *stream);
    let mut first = String::new();
    reader.by_ref().take(8193).read_line(&mut first)?;
    let mut words = first.split_whitespace();
    let method = words.next().unwrap_or_default();
    let path = words.next().unwrap_or_default();
    if first.len() > 8192 || words.next() != Some("HTTP/1.1") {
        return respond(reader.get_mut(), 400, None);
    }
    let mut headers = BTreeMap::new();
    let mut bytes = first.len();
    // 중복 헤더를 거절하고 헤더 전체에도 크기 상한을 둔다.
    loop {
        let mut line = String::new();
        reader.by_ref().take(16385).read_line(&mut line)?;
        bytes += line.len();
        if bytes > 16384 {
            return respond(reader.get_mut(), 413, None);
        }
        if line == "\r\n" {
            break;
        }
        let Some((key, value)) = line.trim_end_matches("\r\n").split_once(':') else {
            return respond(reader.get_mut(), 400, None);
        };
        if headers
            .insert(key.to_ascii_lowercase(), value.trim().to_string())
            .is_some()
        {
            return respond(reader.get_mut(), 400, None);
        }
    }
    let expected_host = format!("127.0.0.1:{}", connection.port);
    let localhost = format!("localhost:{}", connection.port);
    if !headers
        .get("host")
        .is_some_and(|host| host == &expected_host || host == &localhost)
    {
        return respond(reader.get_mut(), 403, None);
    }
    if headers
        .get("origin")
        .is_some_and(|origin| origin != &format!("http://{expected_host}") && origin != &format!("http://{localhost}"))
    {
        return respond(reader.get_mut(), 403, None);
    }
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    if !token_matches(&connection.token, authorization) {
        return respond(reader.get_mut(), 401, None);
    }
    if stop.load(Ordering::Acquire) {
        return respond(reader.get_mut(), 503, None);
    }
    if path != "/mcp" {
        return respond(reader.get_mut(), 404, None);
    }
    if method == "DELETE" {
        let Some(session) = headers.get("mcp-session-id") else {
            return respond(reader.get_mut(), 400, None);
        };
        if !sessions.contains(session) {
            return respond(reader.get_mut(), 404, None);
        }
        sessions.remove(session);
        return respond(reader.get_mut(), 204, None);
    }
    if method != "POST" {
        return respond(reader.get_mut(), 405, None);
    }
    if headers.contains_key("transfer-encoding") {
        return respond(reader.get_mut(), 400, None);
    }
    if !headers
        .get("content-type")
        .is_some_and(|value| value.split(';').next() == Some("application/json"))
    {
        return respond(reader.get_mut(), 415, None);
    }
    let Some(length) = headers.get("content-length").and_then(|v| v.parse::<usize>().ok()) else {
        return respond(reader.get_mut(), 411, None);
    };
    if length > MAX_BODY {
        return respond(reader.get_mut(), 413, None);
    }
    if let Some(version) = headers.get("mcp-protocol-version")
        && version != "2025-06-18"
    {
        return respond(reader.get_mut(), 400, None);
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let request: Value = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(_) => {
            return respond(
                reader.get_mut(),
                200,
                Some(&protocol::error(Value::Null, -32700, "Parse error")),
            );
        }
    };
    let rpc_method = request["method"].as_str().unwrap_or_default();
    let id = request.get("id").cloned();
    let created = rpc_method == "initialize" && id.is_some();
    let session = if created {
        match sessions.create() {
            Ok(session) => Some(session),
            Err(_) => return respond(reader.get_mut(), 503, None),
        }
    } else if let Some(session) = headers.get("mcp-session-id") {
        if !sessions.contains(session) {
            return respond(reader.get_mut(), 404, None);
        }
        Some(session.clone())
    } else if rpc_method == "server/discover" {
        // 최신 클라이언트의 사전 탐색은 method-not-found를 받고 구형 초기화로 폴백한다.
        None
    } else {
        return respond(reader.get_mut(), 400, None);
    };
    if rpc_method == "notifications/cancelled" {
        if let Some(session) = &session {
            sessions.cancel(session, &request["params"]["requestId"]);
        }
        return respond(reader.get_mut(), 202, None);
    }
    let cancelled = if let (Some(session), Some(id)) = (&session, &id) {
        match sessions.register(session, id) {
            Ok(cancelled) => cancelled,
            Err(message) => {
                return respond(
                    reader.get_mut(),
                    200,
                    Some(&protocol::error(id.clone(), -32600, &message)),
                );
            }
        }
    } else {
        Arc::new(AtomicBool::new(false))
    };
    let monitor = if matches!(rpc_method, "tools/call" | "resources/read") {
        Some(DisconnectMonitor::start(reader.get_ref(), cancelled.clone())?)
    } else {
        None
    };
    // 직접 백엔드를 복제하지 않고 참조 어댑터로 명세 처리기에 전달한다.
    struct RefBackend<'a>(&'a (dyn Backend + Send + Sync), Arc<AtomicBool>);
    impl Backend for RefBackend<'_> {
        /// 도구 실행은 원래 UI 전달 백엔드를 따른다.
        fn call(&self, name: &str, args: Value) -> Result<Value, String> {
            self.0.call_cancellable(name, args, self.1.clone())
        }
        /// 공용 레지스트리의 명세를 반환한다.
        fn tools(&self) -> Vec<Value> {
            self.0.tools()
        }
    }
    let mut response = protocol::handle(request, &RefBackend(backend, cancelled.clone()));
    if cancelled.load(Ordering::Acquire) && id.is_some() {
        response = Some(protocol::error(id.clone().unwrap(), -32800, "Request cancelled"));
    }
    if let (Some(session), Some(id)) = (&session, &id) {
        sessions.finish(session, id);
    }
    drop(monitor);
    match response {
        Some(response) => respond_with_session(
            reader.get_mut(),
            200,
            Some(&response),
            created.then_some(session.as_deref()).flatten(),
        ),
        None => respond(reader.get_mut(), 202, None),
    }
}

/// 응답 대기 중 TCP가 닫히면 해당 요청의 UI 승인도 취소한다.
struct DisconnectMonitor {
    done: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl DisconnectMonitor {
    /// HTTP 본문을 모두 읽은 후에만 별도 소켓 핸들로 종료를 감시한다.
    fn start(stream: &TcpStream, cancelled: Arc<AtomicBool>) -> Result<Self> {
        let stream = stream.try_clone()?;
        stream.set_read_timeout(Some(Duration::from_millis(100)))?;
        let done = Arc::new(AtomicBool::new(false));
        let done_thread = done.clone();
        let thread = thread::spawn(move || {
            // 데이터를 소비하지 않고 연결 종료와 오류만 감지한다.
            while !done_thread.load(Ordering::Acquire) {
                match stream.peek(&mut [0u8]) {
                    Ok(0) => {
                        cancelled.store(true, Ordering::Release);
                        break;
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(50)),
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => {
                        cancelled.store(true, Ordering::Release);
                        break;
                    }
                }
            }
        });
        Ok(Self {
            done,
            thread: Some(thread),
        })
    }
}
impl Drop for DisconnectMonitor {
    /// 응답 전 감시 스레드를 종료해 새 요청에 영향을 주지 않게 한다.
    fn drop(&mut self) {
        self.done.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// 연결을 닫는 완전한 HTTP 응답을 쓴다.
fn respond(stream: &mut TcpStream, status: u16, body: Option<&Value>) -> Result<()> {
    respond_with_session(stream, status, body, None)
}

/// 초기화 응답에만 세션을 발급하며 모든 응답의 크기를 공통 상한으로 제한한다.
fn respond_with_session(
    stream: &mut TcpStream,
    status: u16,
    body: Option<&Value>,
    session: Option<&str>,
) -> Result<()> {
    let mut text = body.map(Value::to_string).unwrap_or_default();
    if text.len() > crate::limits::MAX_MESSAGE {
        text = protocol::error(
            body.and_then(|b| b.get("id")).cloned().unwrap_or(Value::Null),
            -32603,
            "Response exceeds 16 MiB",
        )
        .to_string();
    }
    let session = session
        .map(|session| format!("MCP-Session-Id: {session}\r\n"))
        .unwrap_or_default();
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        411 => "Length Required",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        _ => "Service Unavailable",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nAllow: POST, DELETE\r\n{session}Cache-Control: no-store\r\n\r\n{text}",
        text.len()
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 단순 토큰 비교가 접두사·길이·다른 문자를 허용하지 않는지 확인한다.
    #[test]
    fn tokens_require_exact_match() {
        assert!(token_matches("abc", "abc"));
        assert!(!token_matches("abc", "ab"));
        assert!(!token_matches("abc", "abcd"));
        assert!(!token_matches("abc", "abd"));
    }
}
