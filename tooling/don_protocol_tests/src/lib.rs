//! [DonEditor] GPUI 없이 실제 MCP 전송·브릿지·텍스트 연산을 검증하는 계약 테스트.
extern crate self as paths;
#[path = "../../../crates/mcp_server/src/limits.rs"]
pub mod limits;
#[path = "../../../crates/mcp_server/src/sessions.rs"]
pub mod sessions;
use std::path::PathBuf;

/// 전송 모듈이 요구하는 데이터 경로를 테스트 전용 폴더에 격리한다.
pub fn data_dir() -> &'static PathBuf {
    static DIRECTORY: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIRECTORY.get_or_init(|| std::env::temp_dir().join("don-protocol-tests"))
}
#[path = "../../../crates/mcp_server/src/protocol.rs"]
pub mod protocol;
#[path = "../../../crates/don_tools/src/registry.rs"]
pub mod registry;
#[path = "../../../crates/mcp_server/src/transport.rs"]
pub mod transport;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::Arc,
    };
    struct Echo;
    impl protocol::Backend for Echo {
        /// 한글과 도구 인자를 변경 없이 반환한다.
        fn call(&self, _: &str, args: Value) -> Result<Value, String> {
            Ok(json!({"echo":args}))
        }
        /// 실제 편집기 공용 명세를 게시한다.
        fn tools(&self) -> Vec<Value> {
            registry::definitions()
        }
    }
    /// 실제 TCP 연결을 이용해 인증 헤더·Origin·본문·수명 주기를 검증한다.
    #[test]
    fn authenticated_http_lifecycle() {
        let directory = std::env::temp_dir().join(format!("don-mcp-{}", uuid::Uuid::new_v4()));
        let path = directory.join("connection.json");
        let server = transport::Server::start(0, path.clone(), Arc::new(Echo)).unwrap();
        let port = server.connection.port;
        let session = std::cell::RefCell::new(String::new());
        let send = |auth: &str, origin: &str, method: &str, body: &str| {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let session_header = if session.borrow().is_empty() {
                String::new()
            } else {
                format!("MCP-Session-Id: {}\r\n", session.borrow())
            };
            write!(stream,"{method} /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {auth}\r\n{origin}{session_header}Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\n\r\n{body}",body.len()).unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            if let Some(value) = response.lines().find_map(|line| line.strip_prefix("MCP-Session-Id: ")) {
                *session.borrow_mut() = value.trim().to_owned();
            }
            response
        };
        let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize"}).to_string();
        assert!(send("wrong", "", "POST", &initialize).starts_with("HTTP/1.1 401"));
        assert!(
            send(
                &server.connection.token,
                "Origin: https://evil.example\r\n",
                "POST",
                &initialize
            )
            .starts_with("HTTP/1.1 403")
        );
        assert!(send(&server.connection.token, "", "GET", "").starts_with("HTTP/1.1 405"));
        let response = send(&server.connection.token, "", "POST", &initialize);
        assert!(response.starts_with("HTTP/1.1 200"));
        assert!(response.contains("2025-06-18"));
        let call=json!({"jsonrpc":"2.0","id":"한글","method":"tools/call","params":{"name":"echo","arguments":{"text":"안녕🙂"}}}).to_string();
        assert!(send(&server.connection.token, "", "POST", &call).contains("안녕🙂"));
        let notification = json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string();
        assert!(send(&server.connection.token, "", "POST", &notification).starts_with("HTTP/1.1 202"));
        assert!(path.exists());
        drop(server);
        assert!(!path.exists());
        std::fs::remove_dir(&directory).unwrap();
    }
}
