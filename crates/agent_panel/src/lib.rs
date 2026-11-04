//! [DonEditor] GUI 패널과 독립적인 LLM 스트리밍 구현을 함께 제공한다.

#[cfg(feature = "gui")]
mod agent_panel;
/// 설정 화면에서도 API 키 등록 상태를 조회·저장할 수 있도록 공개한다.
#[cfg(feature = "gui")]
pub mod credentials;
pub mod provider;
#[cfg(feature = "gui")]
pub use agent_panel::*;
