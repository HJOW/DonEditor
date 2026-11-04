//! [DonEditor] 실제 stdio 프로세스의 병렬 중계·취소·EOF 종료를 검사한다.
use don_protocol_tests::{protocol::Backend, registry, transport::Server};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// 자식 프로세스와 stdout을 제한 시간 안에서 관찰한다.
struct Bridge {
    child: Child,
    input: Option<ChildStdin>,
    replies: Receiver<String>,
    reader: Option<JoinHandle<()>>,
    stdout: String,
}
impl Bridge {
    /// stdout을 별도로 읽어 응답 누락 때문에 테스트가 영구 대기하지 않게 한다.
    fn start(path: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_don-mcp-contract-bridge"))
            .args(["--connection", path.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (sender, replies) = channel();
        let reader = thread::spawn(move || {
            // 줄 단위로 모아 stdout의 비 JSON 진단 출력도 검출한다.
            for line in BufReader::new(stdout).lines() {
                if sender.send(line.unwrap()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            replies,
            reader: Some(reader),
            stdout: String::new(),
        }
    }
    /// MCP JSON-RPC 한 줄을 전달한다.
    fn send(&mut self, request: Value) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{request}").unwrap();
        input.flush().unwrap();
    }
    /// 입력 순서가 아닌 실제 완료 순서로 응답을 읽는다.
    fn response(&mut self) -> Value {
        let line = self
            .replies
            .recv_timeout(Duration::from_secs(10))
            .expect("stdio 응답 제한 시간 초과");
        self.stdout.push_str(&line);
        self.stdout.push('\n');
        serde_json::from_str(&line).expect("stdout에는 JSON-RPC만 있어야 합니다")
    }
    /// EOF로 요청을 취소하고 정상 종료·비밀 비노출을 검사한다.
    fn finish(&mut self, token: &str) {
        drop(self.input.take());
        let deadline = Instant::now() + Duration::from_secs(10);
        // 제한 시간 초과 시 Drop에서 이 테스트의 자식만 정리한다.
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "EOF 이후 종료되지 않았습니다");
            thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success());
        self.reader.take().unwrap().join().unwrap();
        // EOF와 경쟁해서 도착한 취소 응답도 JSON이어야 한다.
        for line in self.replies.try_iter() {
            serde_json::from_str::<Value>(&line).unwrap();
            self.stdout.push_str(&line);
            self.stdout.push('\n');
        }
        let mut stderr = String::new();
        self.child.stderr.take().unwrap().read_to_string(&mut stderr).unwrap();
        assert!(!self.stdout.contains(token));
        assert!(!stderr.contains(token));
    }
}
impl Drop for Bridge {
    /// 검증 실패 시에도 자식과 stdout 판독 스레드를 남기지 않는다.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

struct Echo;
impl Backend for Echo {
    /// 호출 인자를 정상 결과로 돌려준다.
    fn call(&self, _: &str, args: Value) -> Result<Value, String> {
        Ok(json!({"echo":args}))
    }
    /// 편집기 실제 도구 명세를 사용한다.
    fn tools(&self) -> Vec<Value> {
        registry::definitions()
    }
}

/// 초기화 응답 뒤에 후속 요청을 보낸다.
fn initialize(bridge: &mut Bridge) {
    bridge.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize"}));
    let response = bridge.response();
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object(), "초기화 실패: {response}");
    bridge.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
}

/// 알림은 출력하지 않고 한글 도구 응답을 보존하며 응답 두 개를 받은 뒤 EOF를 보낸다.
#[test]
fn stdio_bridge_round_trip() {
    let directory = std::env::temp_dir().join(format!("don-bridge-{}", uuid::Uuid::new_v4()));
    let path = directory.join("connection.json");
    let server = Server::start(0, path.clone(), Arc::new(Echo)).unwrap();
    let mut bridge = Bridge::start(&path);
    initialize(&mut bridge);
    bridge.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_buffer_text","arguments":{"text":"한글🙂"}}}));
    let response = bridge.response();
    assert_eq!(response["id"], 2);
    assert_eq!(response["result"]["structuredContent"]["echo"]["text"], "한글🙂");
    bridge.finish(&server.connection.token);
    assert_eq!(bridge.stdout.lines().count(), 2);
    drop(server);
    std::fs::remove_dir(&directory).unwrap();
}

/// 승인 대기를 모의하며 실제 서버 취소 토큰을 관찰한다.
struct Waiting {
    started: Sender<Value>,
    cancelled: Sender<Value>,
}
impl Backend for Waiting {
    /// 일반 요청은 곧바로 응답한다.
    fn call(&self, _: &str, args: Value) -> Result<Value, String> {
        Ok(json!({"echo":args}))
    }
    /// 취소된 요청은 부작용 없이 빠져나온다.
    fn call_cancellable(&self, name: &str, args: Value, cancelled: Arc<AtomicBool>) -> Result<Value, String> {
        if args["wait"] != true {
            return self.call(name, args);
        }
        let tag = args["tag"].clone();
        self.started.send(tag.clone()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        // 취소 전달 결함도 무한 대기 대신 테스트 실패로 끝낸다.
        while !cancelled.load(Ordering::Acquire) {
            if Instant::now() >= deadline {
                return Err("취소 신호 제한 시간 초과".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        self.cancelled.send(tag).unwrap();
        Err("취소되었습니다".into())
    }
    /// 편집기 실제 도구 명세를 게시한다.
    fn tools(&self) -> Vec<Value> {
        registry::definitions()
    }
}

/// 테스트마다 서버와 브릿지를 격리한다.
fn waiting() -> (Server, std::path::PathBuf, Bridge, Receiver<Value>, Receiver<Value>) {
    let directory = std::env::temp_dir().join(format!("don-bridge-{}", uuid::Uuid::new_v4()));
    let path = directory.join("connection.json");
    let (started, started_rx) = channel();
    let (cancelled, cancelled_rx) = channel();
    let server = Server::start(0, path.clone(), Arc::new(Waiting { started, cancelled })).unwrap();
    let mut bridge = Bridge::start(&path);
    initialize(&mut bridge);
    (server, directory, bridge, started_rx, cancelled_rx)
}

/// 대기 도구 호출을 보낸다.
fn block(bridge: &mut Bridge, id: u64) {
    bridge.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"get_buffer_text","arguments":{"wait":true,"tag":id}}}));
}

/// 대기 중인 요청이 다른 응답과 취소 알림을 막지 않아야 한다.
#[test]
fn pending_request_does_not_block_other_responses_or_cancellation() {
    let (server, directory, mut bridge, started, cancelled) = waiting();
    block(&mut bridge, 2);
    assert_eq!(started.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
    bridge.send(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_buffer_text","arguments":{"text":"먼저 완료"}}}));
    assert_eq!(bridge.response()["id"], 3);
    bridge.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}}));
    assert_eq!(cancelled.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
    let response = bridge.response();
    assert_eq!(response["id"], 2);
    assert!(response["error"].is_object() || response["result"]["isError"] == true);
    bridge.finish(&server.connection.token);
    drop(server);
    std::fs::remove_dir(&directory).unwrap();
}

/// stdin EOF는 기다리는 도구를 세션 DELETE로 취소해야 한다.
#[test]
fn eof_cancels_pending_session() {
    let (server, directory, mut bridge, started, cancelled) = waiting();
    block(&mut bridge, 2);
    assert_eq!(started.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
    bridge.finish(&server.connection.token);
    assert_eq!(cancelled.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
    drop(server);
    std::fs::remove_dir(&directory).unwrap();
}

/// 여덟 요청이 대기해도 초과 요청은 즉시 거절하고 취소 알림은 계속 처리한다.
#[test]
fn active_limit_does_not_block_cancellation() {
    let (server, directory, mut bridge, started, cancelled) = waiting();
    // 요청 한도를 모두 채운다.
    for id in 10..18 {
        block(&mut bridge, id);
    }
    // 요청이 실제 서버에 도착했음을 확인한다.
    for _ in 0..8 {
        started.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    bridge.send(json!({"jsonrpc":"2.0","id":18,"method":"ping"}));
    let response = bridge.response();
    assert_eq!(response["id"], 18);
    assert_eq!(response["error"]["code"], -32000);
    bridge.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":10}}));
    assert_eq!(cancelled.recv_timeout(Duration::from_secs(5)).unwrap(), 10);
    assert_eq!(bridge.response()["id"], 10);
    bridge.finish(&server.connection.token);
    // 남은 일곱 요청은 EOF로 취소되어야 한다.
    for _ in 0..7 {
        cancelled.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    drop(server);
    std::fs::remove_dir(&directory).unwrap();
}
