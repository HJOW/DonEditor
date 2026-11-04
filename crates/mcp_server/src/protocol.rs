//! [DonEditor] MCP 2025-06-18 JSON-RPC 처리. HTTP 세션·취소는 전송 계층에서 관리한다.
use serde_json::{Value, json};
use std::sync::{Arc, atomic::AtomicBool};

/// 도구 호출과 리소스 조회를 UI 스레드에 위임하는 공용 경계.
pub trait Backend {
    /// GPUI 객체를 직접 참조하지 않고 직렬화된 요청을 실행한다.
    fn call(&self, name: &str, arguments: Value) -> Result<Value, String>;
    /// 모든 도구 명세를 돌려준다.
    fn tools(&self) -> Vec<Value>;
    /// 취소 가능한 전송은 이 진입점으로 UI 작업에 취소 토큰을 전달한다.
    fn call_cancellable(&self, name: &str, arguments: Value, _cancelled: Arc<AtomicBool>) -> Result<Value, String> {
        self.call(name, arguments)
    }
}

/// JSON-RPC 오류 봉투를 구성한다.
pub fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

/// 결과를 중복 인코딩하기 전에 UTF-8 크기를 검사한다.
fn result_text(value: &Value) -> Result<String, String> {
    let text = value.to_string();
    if text.len() > crate::limits::MAX_RESULT {
        Err("도구 결과가 4 MiB를 초과합니다. 더 작은 파일이나 범위를 사용하세요".into())
    } else {
        Ok(text)
    }
}

/// MCP 요청을 처리하며 알림에는 응답을 생성하지 않는다.
pub fn handle(request: Value, backend: &impl Backend) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    if !request.is_object() || request["jsonrpc"] != "2.0" || !request["method"].is_string() {
        return Some(error(id, -32600, "Invalid JSON-RPC request"));
    }
    if !request.get("id").is_some() {
        return None;
    }
    if !(id.is_null() || id.is_string() || id.is_number()) {
        return Some(error(Value::Null, -32600, "Invalid request id"));
    }
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    let result = match request["method"].as_str().unwrap() {
        "initialize" => {
            json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{},"resources":{}},"serverInfo":{"name":"Don Editor","version":env!("CARGO_PKG_VERSION")},"instructions":"Actions run in the active editor window. Mutations require approval inside Don Editor."})
        }
        "ping" => json!({}),
        "tools/list" => json!({"tools":backend.tools()}),
        "tools/call" => {
            let Some(name) = params["name"].as_str() else {
                return Some(error(id, -32602, "Missing tool name"));
            };
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match backend.call(name, args) {
                Ok(value) => match result_text(&value) {
                    Ok(text) => {
                        json!({"content":[{"type":"text","text":text}],"structuredContent":value,"isError":false})
                    }
                    Err(message) => json!({"content":[{"type":"text","text":message}],"isError":true}),
                },
                Err(message) => json!({"content":[{"type":"text","text":message}],"isError":true}),
            }
        }
        "resources/list" => json!({"resources":[
            {"uri":"doneditor://active-buffer","name":"Active editor","mimeType":"application/json"},
            {"uri":"doneditor://project-tree","name":"Project tree","mimeType":"application/json"},
            {"uri":"doneditor://diagnostics","name":"Diagnostics","mimeType":"application/json"}
        ]}),
        "resources/templates/list" => json!({"resourceTemplates":[]}),
        "resources/read" => {
            let Some(uri) = params["uri"].as_str() else {
                return Some(error(id, -32602, "Missing resource URI"));
            };
            let name = match uri {
                "doneditor://active-buffer" => "get_buffer_text",
                "doneditor://project-tree" => "list_project_files",
                "doneditor://diagnostics" => "get_diagnostics",
                _ => return Some(error(id, -32002, "Unknown resource")),
            };
            match backend.call(name, json!({})) {
                Ok(value) => match result_text(&value) {
                    Ok(text) => json!({"contents":[{"uri":uri,"mimeType":"application/json","text":text}]}),
                    Err(message) => return Some(error(id, -32603, &message)),
                },
                Err(message) => return Some(error(id, -32603, &message)),
            }
        }
        _ => return Some(error(id, -32601, "Method not found")),
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 중복 JSON 인코딩 전에 결과를 제한해 stdio와 HTTP의 동작을 일치시킨다.
    #[test]
    fn oversized_results_are_rejected() {
        assert!(result_text(&json!({"text":"한글🙂"})).is_ok());
        assert!(result_text(&json!({"text":"x".repeat(crate::limits::MAX_RESULT)})).is_err());
    }
    struct Fake;
    impl Backend for Fake {
        /// 테스트 도구의 결과와 실패를 재현한다.
        fn call(&self, name: &str, args: Value) -> Result<Value, String> {
            if name == "fail" {
                Err("거절".into())
            } else {
                Ok(json!({"args":args}))
            }
        }
        /// 최소 도구 목록을 제공한다.
        fn tools(&self) -> Vec<Value> {
            vec![json!({"name":"echo"})]
        }
    }
    /// 초기화·도구 오류·리소스·알림·잘못된 요청을 확인한다.
    #[test]
    fn lifecycle_and_errors() {
        let initialize = handle(json!({"jsonrpc":"2.0","id":1,"method":"initialize"}), &Fake).unwrap();
        assert_eq!(initialize["result"]["protocolVersion"], "2025-06-18");
        assert!(handle(json!({"jsonrpc":"2.0","method":"notifications/initialized"}), &Fake).is_none());
        assert_eq!(
            handle(
                json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"fail"}}),
                &Fake
            )
            .unwrap()["result"]["isError"],
            true
        );
        assert_eq!(
            handle(
                json!({"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"invalid"}}),
                &Fake
            )
            .unwrap()["error"]["code"],
            -32002
        );
        assert_eq!(handle(json!([]), &Fake).unwrap()["error"]["code"], -32600);
        assert_eq!(
            handle(json!({"jsonrpc":"2.0","id":4,"method":"missing"}), &Fake).unwrap()["error"]["code"],
            -32601
        );
    }
}
