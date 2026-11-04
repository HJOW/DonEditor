//! [DonEditor] 공식 MCP 클라이언트 상호운용 테스트용 서버. 실제 편집기는 제어하지 않는다.
use don_protocol_tests::{protocol::Backend, registry, transport::Server};
use serde_json::{Value, json};
use std::{io::Read, path::PathBuf, sync::Arc};
struct Fixture;
impl Backend for Fixture {
    /// 명세대로 입력을 검사하고 고정 문서 내용을 반환한다.
    fn call(&self, name: &str, args: Value) -> Result<Value, String> {
        registry::validate(name, &args).map_err(|e| e.to_string())?;
        Ok(json!({"text":"Don Editor 한글🙂","name":name}))
    }
    /// 실제 제품 도구 명세를 제공한다.
    fn tools(&self) -> Vec<Value> {
        registry::definitions()
    }
}
/// 연결 파일 경로를 받고 stdin이 닫힐 때까지 테스트 서버를 유지한다.
fn main() -> anyhow::Result<()> {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("연결 파일 경로가 필요합니다"))?,
    );
    let _server = Server::start(0, path, Arc::new(Fixture))?;
    let _ = std::io::stdin().read(&mut [0u8]);
    Ok(())
}
