//! [DonEditor] HTTP 서버와 stdio 브릿지가 공유하는 전송 크기 제한.
/// JSON-RPC 메시지 하나의 최대 UTF-8 바이트 수.
pub const MAX_MESSAGE: usize = 16 * 1024 * 1024;
/// 도구 결과는 텍스트·구조화 결과로 중복 인코딩되므로 전송 한도보다 작게 제한한다.
pub const MAX_RESULT: usize = 4 * 1024 * 1024;
