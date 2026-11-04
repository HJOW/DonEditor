# Agent 패널

2026-11-04 작업 체크포인트: 아래 기능은 초기 구현 상태다. 전체 앱의 GUI 통합 컴파일·실제 패널 표시/숨김/아이콘 클릭·OS 자격 증명 저장/삭제·유효한 API 키를 쓰는 실제 공급자 호출은 아직 검증을 완료하지 않았다. 공급자 프로토콜은 같은 소스 모듈을 GUI 없이 가짜 HTTP 응답으로 검사한다. 실제 계정이나 유료 API 요청은 사용하지 않았다.

보기 메뉴의 Agent 패널 또는 도크의 반짝임 아이콘으로 연다. 기본 위치는 오른쪽이며, 패널의 닫기 버튼으로 숨겨도 도크 아이콘으로 다시 열 수 있다. 도크 아이콘의 우클릭 메뉴에서 왼쪽으로 옮길 수 있다.

제공자 버튼으로 OpenAI와 Anthropic을 선택하고 모델 ID를 직접 입력한다. 기본 모델은 OpenAI `gpt-6-astra`, Anthropic `claude-sonnet-5-5`다. API 사용은 선택한 제공자의 계정에 청구된다.

API 키 입력란은 화면에서 마스킹한다. **키 저장**은 Windows 자격 증명 관리자, macOS Keychain, Linux Secret Service에 `DonEditor.Agent` 서비스의 제공자별 항목으로 저장한다. 설정 JSON이나 대화 DB에는 키를 기록하지 않는다. 저장된 키가 없으면 에디터 프로세스의 `OPENAI_API_KEY` 또는 `ANTHROPIC_API_KEY` 환경 변수를 읽는다. **키 삭제**는 OS 저장소의 항목만 지우며 환경 변수는 바꾸지 않는다. Linux에서 저장하려면 실행 중인 Secret Service와 D-Bus 세션이 필요하다.

메시지를 입력하고 **보내기**를 누르면 응답이 Markdown으로 표시된다. **현재 컨텍스트 첨부**를 누른 뒤 보내면 현재 파일·선택 영역·열린 탭 정보가 메시지에 포함되어 선택한 API 제공자로 전송된다. **중단**은 진행 중인 응답과 후속 도구 호출을 취소한다.

Agent와 MCP는 `don_tools`의 같은 도구 정의와 실행기를 사용한다. 파일 수정 등 변경 작업은 공용 승인 화면에서 검토하며, 편집은 변경 내용을 확인한 뒤 승인한다. 도구 요청의 JSON이 완성되지 않았거나 스트림이 잘렸으면 실행하지 않는다. 한 번의 사용자 메시지에서 자동으로 이어가는 도구 응답은 최대 12회이며, 그 뒤에는 새 메시지로 계속할 수 있다.

MCP의 `list_actions`로 Agent 액션의 입력 스키마도 조회할 수 있다. `dispatch_action`을 통해 `agent_panel::Configure`에 `{"provider":"anthropic","model":"claude-sonnet-5-5"}`를, `agent_panel::SendPrompt`에 `{"text":"현재 코드의 문제를 확인해 줘"}`를 전달한다. 인자 없는 `ToggleFocus`, `AttachContext`, `NewChat`, `ShowHistory`, `Send`, `Stop`도 같은 `agent_panel::` 이름 공간에 있다. API 키는 이 액션 인자로 받지 않는다.

기록은 `list_agent_conversations` 도구에서 제목과 번호를 조회한 다음 `agent_panel::SelectConversation`에 `{"index":0}`처럼 표시 순서의 번호를 전달해 열고, `agent_panel::ClearHistory`로 삭제할 수 있다. `agent_panel::MovePanel`의 `{"position":"left"}` 또는 `{"position":"right"}`로 패널을 옮긴다. 이 액션도 MCP의 일반 액션 실행 승인 절차를 따른다.

**새 대화**로 대화를 나누고 **대화 기록**에서 워크스페이스별 최근 20개 대화를 다시 열 수 있다. 기록은 해당 사용자의 Don Editor 로컬 데이터 DB에 저장한다. **기록 삭제**로 저장된 대화를 지울 수 있다.

## 검증

2026-11-04 Windows/Rust 1.99.1에서 GUI 없는 계약 테스트 **9개 통과, 실패 0개**를 확인했다. 한글·CRLF·여러 data 줄, 도구 인수 조각, Claude 서명/비공개 블록의 왕복 보존, 잘린 응답·토큰 한도 거절, 인증 오류의 비밀 비노출, 생성 취소 시 응답 본문 해제, 중단한 도구 대화의 재개를 검사한다. 네 Rust 소스 파일의 `rustfmt --check`도 통과했다. GUI 기능이 켜진 빌드·테스트와 clippy는 아직 완료하지 않았다.

실제 API 키나 외부 유료 요청 없이 공급자 요청 형식과 SSE 응답을 검사한다. 저장소 루트에서 실행한다.

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
$env:CARGO_BUILD_JOBS = '1'
$env:CARGO_TARGET_DIR = 'target/contracts'
cargo test -p agent_panel --no-default-features --lib
```

기본 `gui` 기능은 실제 앱 패널·OS 자격 증명 저장소를 포함한다. `--no-default-features`는 동일한 `provider.rs`를 그대로 검증하며, UI와 OS 자격 증명 저장·실제 API 계정의 권한/잔액은 별도의 실행 검증 대상이다.

구현 형식의 근거는 [OpenAI Chat Completions 스트리밍 이벤트](https://developers.openai.com/api/reference/resources/chat/subresources/completions/streaming-events), [Claude Messages 스트리밍](https://platform.claude.com/docs/en/build-with-claude/streaming), [Claude 도구 호출 시 서명 블록 보존](https://platform.claude.com/docs/en/build-with-claude/thinking-tool-workflows), [Claude API 버전](https://platform.claude.com/docs/en/api/versioning), [Claude 모델 목록](https://platform.claude.com/docs/en/models/overview)이다. 기본 모델과 API 버전 `2026-10-21`은 2026-11-04 문서 기준이며 제공자 정책이나 계정에 따라 이용 가능한 모델이 다를 수 있다.
