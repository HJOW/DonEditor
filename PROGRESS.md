# Don Editor 진행 상황

## ⚠ 문서 보호 규칙 (사용자 지시, 2026-11-04)

- **README.md의 앞 두 단락은 사용자가 직접 다시 작성했다. 사용자가 그 부분의 수정을 명시적으로 요청하지 않는 한 절대 수정하지 않는다.**
  1. **맨 앞 단락:** 제목 `# Don Editor` 바로 아래부터 `## 이름의 유래` 제목 직전까지의 소개 단락 전체 ("Don Editor 는 한국 개발자를 위한 서브용 텍스트 편집기입니다."로 시작해 사용 안내 링크 문장으로 끝나는 부분).
  2. **이름의 유래 단락:** `## 이름의 유래` 제목과 그 아래 본문.
- 이 보호는 문구·줄바꿈·띄어쓰기·링크·줄 끝 공백 등 서식까지 포함한다. 맞춤법 교정, 문서 최신화, 버전·기능 설명 갱신, 자동 서식 정리 등 어떤 이유로도 고치지 않는다.
- README.md의 나머지 부분(`## 목표 기능` 이후)은 기존처럼 수정할 수 있다. 다만 다른 부분을 고칠 때도 위 두 단락이 바뀌지 않았는지 diff로 확인한다.
- 위 두 단락의 내용이 실제 기능과 맞지 않게 되면 직접 고치지 말고 사용자에게 알리고 수정 여부를 묻는다.



## 현재 상태 요약 (2026-11-05 갱신 — 다음 작업 때 가장 먼저 읽는다)

**사용자 요청 작업은 모두 구현·검증·커밋했다.** 커밋: Markdown 미리보기/편집 전환 `475437a`, 인코딩 표시·다시 불러오기·저장 보호 `c29fe81`, Typst 미리보기·Markdown→Typst 변환·PDF·인쇄 `4421531`, MCP 확인 `2e9ff1c`, 릴리스 ZIP 기록 `1808ab8`, clippy·회귀·README 정리 `d257aff`~`2e622db`. 자동 검사는 markdown_preview 40·don_charset 9·don_typst 17·language 99·text 24·worktree 26·gram 35·MCP 전송 25개(Windows ACL 선택 테스트 포함)·i18n 11개가 통과하고, 새 crate와 `don_tools`·`mcp_server`·`agent_panel`의 clippy 경고는 0건이다.

**남은 항목과 이유(모두 사용자 결정 또는 이 환경에서 할 수 없는 것):**
- 사용자 결정 대기: Don Editor 자체 기여 안내·행동 강령 필요 여부, 배포 시 Don Editor 소스의 공개 위치(현재는 배포본에 수정 소스를 동봉), `README.md` 보호 단락 내용 갱신이 필요하면 사용자 요청 후에만.
- 사용자 지시로 제외: Linux/macOS 직접 실행·tar.gz 실물 검증, 실제 OpenAI/Claude API 호출 검증(유료·키 필요), Claude Desktop/Code 실제 연결(외부 클라이언트의 API 호출이 따름).
- 이 환경에서 불가: 깨끗한 Windows/ARM64 설치 검증, 실제 프린터 출력과 PDF 뷰어 `print` 동작 확인(기본 프린터가 가상 프린터), 탐색기 UI에서 `.md`를 연결 프로그램으로 직접 지정하는 수동 확인.
- 계속 진행형: 기능 인벤토리의 미확인 영역(실행 확인 22개/미확인 28개), 'Gram' 문자열 잔여 백로그.
- 알려진 경미한 문제: ① 이번 세션의 릴리스 ZIP은 커밋 `2e9ff1c` 기준(이후 변경은 동작 불변 정리), ② 상태 표시줄 `Failed to load environment variables`(콘솔 로그상 `H:` 드라이브에서 PowerShell 환경 로드 실패 — 이 PC의 환경 문제로 보이며 이전부터 있던 메시지), ③ MCP 승인 창의 일부 요소가 화면 가운데 위쪽에 뭉쳐 보이는 레이아웃(기능은 동작), ④ `don_protocol_tests`의 브릿지 취소 테스트가 부하에서 가끔 실패하는 타이밍 경쟁(8회 연속 통과로 확인).

## 2026-11-05 회귀 테스트·clippy 정리 (사용자 목표 진행)

- **변경 crate 전체 테스트:** `cargo test -p text -p language -p worktree --lib`가 **text 24개·language 99개·worktree 26개 모두 통과**했다(이번 세션에서 `text::Buffer::clear_history`, `language::Buffer::reload_with_encoding`, `worktree::write_file`의 인코딩 변환을 바꿨으므로 회귀 확인). 이 테스트들은 `markdown_preview` 40개, `don_charset` 9개, `don_typst` 17개, i18n 11개와 함께 통과한 상태다.
- **clippy 정리:** 이전부터 남아 있던 `don_tools` 1건과 `mcp_server` 5건을 정리해 `don_tools`·`mcp_server`·`agent_panel`·새 crate 4개(`don_charset`·`don_typst`·`typst_preview`·`encoding_selector`) 모두 `--no-deps` 경고 0건이다. 접을 수 있는 `if` 3건은 코드를 고쳤다. `mcp_server`의 Windows 보안 도구(`whoami`·`icacls`) 동기 실행 3건은 **구조를 바꾸지 않고 허용 사유를 코드에 명시**했다(`run_windows_tool`): MCP 서버를 사용자가 켤 때(기본값은 꺼짐) 한 번만 부르는 100ms 안팎의 짧은 명령이고, 시작을 비동기로 바꾸면 전송 시험·실사용 확인이 의존하는 "폴더 보호 후 토큰 파일 생성" 순서를 건드리기 때문이다.
- **MCP 전송 테스트:** `don_protocol_tests` 전체(기본 14개·브릿지 4개·전송 6개·키링 1개)가 통과했고, 환경 변수 `DONEDITOR_TEST_WINDOWS_ACL=1`로 켜는 **Windows ACL 선택 테스트도 통과**해 이번 ACL 실행 코드 정리에 문제가 없음을 확인했다. 한 번 `active_limit_does_not_block_cancellation`가 "EOF 직후 잘린 JSON 한 줄"로 실패했으나(다른 작업이 CPU를 쓰는 중이었고 브릿지 코드는 바꾸지 않음) 같은 테스트를 8번 연속 실행해 모두 통과했다. 부하 상황에서 가끔 생기는 기존 테스트의 타이밍 경쟁으로 보이며 원인을 고치지는 않았다.
- **`gram` 크레이트 테스트 35개 통과:** 액션 네임스페이스·기본 설정/테마/언어/키맵·메뉴 언어·파일 열기 요청(`--wait`·`--reuse`·SSH/Git URL 해석 포함)·열기/저장/분할/탭 복원·충돌 저장 등. 이번 세션이 바꾼 키맵 컨텍스트(`MarkdownPreview || TypstPreview`)와 새 네임스페이스(`encoding_selector`, `typst`)가 목록 검사를 통과했다. 처음에는 `test_action_namespaces`가 실패했는데, 원인은 **이전 단계(Agent 패널·MCP 도입)에서 기대 목록에 `agent_panel`·`mcp_server`를 넣지 않은 기존 결함**이어서 두 항목을 추가해 고쳤다.
- **기존 기능 회귀 스모크 점검(R8, Windows 디버그 빌드, 격리 데이터 폴더 `target/gui-test-regress-fcf47f1e/`):** 작은 Git 프로젝트(Rust·Python·JSON·Markdown 파일, 수정 1건)를 열고 한국어 UI에서 확인했다 — 프로젝트 패널 트리·파일 아이콘, Rust 문법 강조·개요 경로(`src\main.rs › fn main`)·Git 줄 표시·인라인 blame(`tester, Just now`)·제목 표시줄 브랜치, **파일 찾기**(`Ctrl+P`), **버퍼 검색**(`Ctrl+F`, 일치 2건 `1/2`), **프로젝트 검색**(`Ctrl+Shift+F`, 멀티버퍼 결과), **터미널**(`Ctrl+\``, PowerShell에서 한글 `echo` 입출력 정상), **Git 패널**(`Ctrl+Shift+G`, 변경 1건 +1 −0, 한국어 버튼·커밋 목록), **설정 창**(`Ctrl+,`, 13개 페이지·에이전트 포함), **테마 선택**(`Ctrl+K Ctrl+T`), **명령 팔레트**(영문+한글 이름 병기), **내장 도움말**(`gram: open docs` → 한국어 `Don Editor 사용 안내`), **Agent 패널**(`Ctrl+Alt+A`로 우측에 열림, 제공자 콤보박스·모델 ID·키 없음 안내와 인라인 키 입력). 시작 시 Agent 패널이 숨겨져 있는 것도 다시 확인했다. [기능 인벤토리](FEATURE-INVENTORY.md)의 실행 확인 영역이 3개에서 **12개**로 늘었다(미확인 38개는 계속 완료 처리하지 않는다).
- **기존 기능 회귀 스모크 점검 2차(R8, 같은 디버그 빌드·같은 격리 폴더):** **이미지 뷰어**(160x100 PNG를 열면 이미지와 크기·확대 정보 표시), **SVG**(`.svg`를 열면 편집기와 함께 `Preview pic.svg` 미리보기 탭), **탭 전환**(`Ctrl+Tab`), **줄 이동**(3을 입력하면 "3번째 줄로 이동" 안내와 함께 3줄로 이동), **언어 선택기**(상태 표시줄 언어 이름 클릭 → 언어 목록), **줄 끝 선택기**(CRLF 선택), **개요 패널**(Rust 파일에서 `fn main` 항목), **최근 프로젝트**(`Ctrl+R` 선택 창과 원격/로컬 폴더 열기 단축키가 열림 — 격리 데이터라 기록이 비어 있어 항목을 골라 여는 동작은 못 봄), **키맵 편집기**(메뉴 "키맵 열기" → 한국어 열 제목의 단축키 표). 명령 팔레트의 `keymap editor: open`은 이 화면을 열지 않았다(메뉴 경로는 동작). 기능 인벤토리 실행 확인 영역이 12개에서 **20개**(미확인 30개)로 늘었다. 확인 도중 개요 패널의 빈 상태 안내가 영어(`No outlines available`, `Toggle Panel With ctrl-B`)로 남아 있는 것을 발견했다.
- **개요 패널 빈 상태 안내 한국어화:** 위에서 발견한 문구 3개(`No outlines available`·`No matches for query`·`Toggle Panel With {keystroke}`)를 i18n 키 `outline.empty`·`outline.no_matches`·`outline.toggle_hint`로 옮기고 한국어 문구("표시할 개요가 없습니다" 등)를 추가했다(카탈로그 1,824 → 1,827키). `cargo check -p outline_panel` 경고 0, i18n 테스트 11개 통과, rustfmt 통과. **고친 뒤의 화면은 다시 실행해 보지 않았다**(코드·카탈로그 검증까지만).
- **CLI 도움말의 Gram 표기 정리:** `doneditor --help`가 기본 데이터 폴더를 `%LOCALAPPDATA%\Gram`(macOS `Application Support/Gram`, Linux `$XDG_DATA_HOME/gram`)으로 안내했으나 실제 폴더는 `DonEditor`(crates/paths)라 사용자를 오도했다. 데이터 폴더 안내와 `paths`·`--version`·`--completions` 예시·`--foreground`·`--gram`·`--wsl`·`--uninstall` 설명의 제품명/명령 이름을 Don Editor·`doneditor`로 바꿨다(`--gram` 플래그 이름과 내부 오류 문구는 호환성 때문에 유지). `cargo build -p cli` 성공, `--help` 출력으로 확인.
- **F10 메뉴 단축키가 동작하지 않던 회귀 수정:** 기본 키맵의 `f10` 바인딩이 열 메뉴 이름을 옛 이름 `"Gram"`으로 넘기는데, 제품 이름 변경(Phase 5)으로 메뉴 이름은 `Don Editor`가 되었고 제목 표시줄은 `메뉴 이름 == 인자`로 찾기 때문에 F10을 눌러도 아무 메뉴도 열리지 않았다(키맵 편집기 화면의 `app menu: open application menu "Gram" F10` 줄에서 발견). `default-windows.jsonc`·`default-linux.jsonc`·`linux/minimal.jsonc`의 인자를 `"Don Editor"`로 바로잡았고, 같은 문제가 다시 생기지 않도록 `gram` 테스트 `test_keymap_application_menu_names_exist`(기본 키맵의 `OpenApplicationMenu` 인자가 영·한 두 언어의 실제 메뉴 이름에 있는지 검사)를 추가했다. `cargo test -p gram --lib`가 **36개 모두 통과**했다(기존 35개 + 신규 1개, 수정한 기본 설정·키맵 파싱 테스트 포함). 화면에서 F10이 실제로 메뉴를 여는지는 이 시점에 PC가 잠금 화면이라 눈으로 확인하지 못했다.
- **사용자가 직접 보는 파일의 제품명 정리:** 새 사용자 설정/키맵 파일을 만들 때 그대로 기록되는 `initial_user_settings.jsonc`·`initial.jsonc`의 첫머리 안내("Gram settings" 등)와, "기본 설정 열기"에서 보이는 `default.jsonc` 주석 37줄의 `Gram`을 `Don Editor`로 바꿨다. 글꼴 식별자(`.GramMono`·`.GramSans`)·아이콘 테마 이름 `Gram (Default)`·문서 링크(`gram://docs/...`)·명령 팔레트 이름(`gram: open ...`)은 실제 동작과 맞추기 위해 그대로 둔다.
- **`editor` crate 테스트:** 이번 세션이 `Editor`에 대체 화면 슬롯(`alternate_view`)과 렌더링 분기를 추가했으므로 `cargo test -p editor --lib --offline`로 회귀를 확인했다 — **475개 통과, 실패 0, 무시 5**.
- **GUI 점검 중단 사유:** 이 시점부터 PC가 잠금 화면(`LogonUI` 실행 중, 화면 캡처가 "The handle is invalid"로 실패)이라 앱 창을 만들 수 없어 추가 GUI 회귀 점검(Vim·Helix·작업 실행·자동 완성 등)과 `Failed to load environment variables` 원인 진단(앱 로그 수집)은 하지 못했다. 앱을 직접 실행해 확인한 명령줄(`--printenv`를 PowerShell에서 H: 경로·`\?\H:` 경로·C: 경로로 실행)은 모두 정상이라 PowerShell 호출 자체의 문제는 아니며, 앱 안에서만 실패하는 원인은 로그가 필요하다.
- **배포 ZIP 참고:** 위 릴리스 ZIP은 커밋 `2e9ff1c` 기준이다. 이후 커밋(`d257aff` 인쇄 실행을 표준 비동기 명령으로, `97296d7`·이번 clippy 정리)은 동작을 바꾸지 않는 정리라 ZIP을 다시 만들지 않았다.

## 2026-11-05 Windows 릴리스 배포본 재생성과 실행 확인 (Markdown 미리보기·캐릭터셋·Typst 반영)

- 사용자 목표("PROGRESS.md의 작업을 모두 진행")에 따라 위 세 기능(Markdown 미리보기/편집 전환 `475437a`, 인코딩 표시·다시 불러오기 `c29fe81`, Typst 지원 `4421531`)과 MCP 확인(`2e9ff1c`)을 반영한 Windows x86_64 배포본을 새로 만들었다. `script/package-zip.ps1 -Yes -OutDir dist/windows-20261105-040131`을 Visual Studio 개발자 셸(amd64)에서 실행했고 `cargo build --locked --package gram --package cli --package mcp_server --release`가 **83분 59초**에 성공했다(종료 코드 0).
- 배포 파일: `dist/windows-20261105-040131/DonEditor-0.1.0-windows-x86_64.zip` (95,883,435바이트, 91.4 MiB). SHA-256 `933f5f2e42b59bd0e8665d184af63043b4fd22045a512e2769b3526b7263b5a0`이며 동봉 `.sha256`과 별도 계산값이 일치한다. `dist/`는 `.gitignore` 대상이라 저장소에는 커밋하지 않는다.
- **크기 변화:** GUI `doneditor.exe`가 이전 릴리스(3.4.0 기반, 173,635,584바이트 ≈ 165.6 MiB)에서 **202.5 MiB(약 +37 MiB, +22%)**로, ZIP이 77,238,393바이트에서 95,883,435바이트(+18.6 MB, +24%)로 늘었다. Typst 컴파일러·PDF/SVG 내보내기와 그 의존 crate 117개, Typst 내장 글꼴(Libertinus Serif·New Computer Modern·DejaVu Sans Mono)이 원인으로 보이나 PDB 기여량 분석은 하지 않았다([용량 보고서](WINDOWS-SIZE-REPORT.md)는 이전 3.4.0 배포본 기준이며 갱신하지 않음). CLI는 `bin/doneditor.exe` 2.5 MiB, MCP 브릿지 `bin/don-mcp.exe` 0.5 MiB.
- **압축 해제 확인(`extracted/`):** 구성은 `doneditor.exe`·`bin/`·`licenses/`(193개 파일: Typst·typst-assets·md2typ 라이선스와 NOTICE, 새 외부 crate 117개의 고지 `typst-deps/`와 `typst-dependencies.md`)·`source/`·`LICENSE`·`licenses.md`·`README.md`·`SOURCE.txt`다. `bin/doneditor.exe --version`은 `Don Editor 0.1.0 (2e9ff1c…)`를 출력하고 `bin/don-mcp.exe`는 사용법을 출력한다.
- **압축 해제본 GUI 실행 확인(격리 데이터 폴더 `smoke-data`):** `.typ`를 열면 미리보기 모드로 한글·수식·목록·표가 조판되고(상태 표시줄 `Typst  UTF-8`), `.md`는 미리보기로, EUC-KR `.txt`는 올바른 한글과 `EUC-KR` 표시로 열렸다. 디버그 빌드에서 확인한 흐름이 릴리스에서도 같다. 릴리스 빌드에서의 변환·PDF·인쇄 메뉴, 인코딩 다시 불러오기, MCP 실행은 따로 반복하지 않았다.
- **한계:** Linux/macOS 배포본(tar.gz)은 사용자 지시에 따라 직접 만들거나 실행하지 않았다. 깨끗한 Windows 환경·ARM64·코드 서명은 확인하지 않았다.

## Typst 지원, Markdown→Typst 변환, 인쇄·PDF 변환 (사용자 요청 2026-11-04 접수, 2026-11-05 구현 — 결과는 이 절 맨 아래 "구현 결과" 참고)

- **선행 작업:** Markdown 미리보기/편집 모드 작업(같은 탭의 대체 화면 `Editor::set_alternate_view`, 툴바 `미리보기 | 편집` 토글, 열 때 미리보기로 시작)은 2026-11-04에 구현·Windows GUI 검증을 마쳤다(아래 "Markdown 미리보기/편집 모드" 절). 아래 작업은 이 구조를 재사용한다.
- **참고 저장소:** Typst <https://github.com/typst/typst> (오픈소스 조판 소프트웨어이자 자체 마크업 언어), md2typ <https://github.com/nervosys/md2typ> (Markdown→Typst 변환 참고 라이브러리).

### 요구사항

1. **Typst 언어 지원:** `.typ` 확장자로 Typst 언어를 감지하고 기본 문법 강조를 제공한다. AGENTS.md의 26개 언어 목록 외에 사용자가 추가 요청한 언어이므로 PROGRESS.md의 R2 표·[기능 인벤토리](FEATURE-INVENTORY.md)·자체 도움말의 지원 언어 목록에 함께 반영한다(AGENTS.md는 사용자 요청 없이 수정하지 않는다).
2. **Typst 미리보기/편집 모드:** Typst도 마크업 언어이므로 Markdown과 같이 같은 탭 안에서 미리보기 모드와 편집 모드를 전환하는 토글을 제공하고, **파일을 불러오면 미리보기 모드로 먼저 보여 준다.** 빈 파일 처리·분할/탭 이동 시 모드 유지·저장/변경 표시 등 동작 규칙은 Markdown과 맞춘다.
3. **Markdown→Typst 변환:** Markdown 파일을 연 상태에서 Typst로 변환하는 기능을 **메뉴를 통해** 제공한다. 변환 규칙은 md2typ를 참고한다. **(2026-11-04 사용자 결정) 메뉴를 선택하면 곧바로 파일 저장 창을 띄워 변환 결과를 `.typ` 파일로 즉시 저장한다.** 저장 창에서 취소하면 아무 파일도 만들지 않는다.
4. **인쇄와 PDF 변환:** Markdown과 Typst 문서를 인쇄하거나 PDF로 변환(저장)할 수 있어야 한다. **Markdown 파일에 이 기능을 쓰면 자동으로 Typst로 변환한 뒤 인쇄/PDF 변환을 수행한다.** **(2026-11-04 사용자 결정) 이때 중간 변환 결과(Typst)는 파일로 저장하지 않는다.** 메모리에서만 변환해 쓰고, 구현상 임시 파일이 꼭 필요하면 작업 후 지운다. PDF 변환은 결과 PDF만 저장한다.

### 착수 시 확인·결정할 사항 (아래 항목은 계획 당시 기록이며, 조사·결정 결과는 "구현 결과" 절에 있다)

- **라이선스 (2026-11-04 사용자 확인):** Typst와 md2typ는 모두 **Apache License 2.0**이며, GPL-3.0 프로그램인 Don Editor에서 **라이선스상 이용 가능**하다. 다만 다음을 반드시 지킨다.
  - 프로그램 안에 **두 프로젝트의 Apache-2.0 라이선스 파일을 첨부**하고 **Credit(원저작자·저작권 고지)을 표시**한다. 앱 내 라이선스 고지(`assets/licenses.md`)와 배포 zip/tar.gz의 `licenses/`에 모두 넣고, 라이선스 검사기 대상에도 추가한다.
  - Apache-2.0 4절에 따라 원본 저장소에 `NOTICE` 파일이 있으면 그 내용도 함께 고지하고, 원본 코드를 옮겨 와 수정한 파일에는 원저작 고지를 유지하고 수정 사실을 표시한다.
  - 실제로 쓰는 Typst 하위 crate(컴파일러·렌더러·PDF 등)와 그 추가 의존성의 라이선스도 착수 시 Cargo.lock 기준으로 개별 확인한다(두 본 저장소의 라이선스 확인과 별개).
  - md2typ는 의존성으로 쓸지, 변환 규칙만 참고해 직접 구현할지를 구현 언어·유지 상태를 보고 정한다. 어느 쪽이든 위 고지 의무는 같다.
- **문법 강조:** 내장 tree-sitter Typst 문법 후보와 라이선스, 기존 `register_native_grammars` 등록 방식과의 호환성을 조사한다.
- **미리보기 렌더링:** Typst 컴파일러를 라이브러리로 내장해 페이지를 렌더링(SVG/이미지 등)하는 방식을 조사한다. 편집 중 재컴파일 지연(debounce)·컴파일 오류 표시·문서 기준 상대 경로(이미지·`#import`)·글꼴(D2Coding 포함 한글 글꼴)·실행 파일 용량 증가([용량 보고서](WINDOWS-SIZE-REPORT.md) 기준)를 함께 검토한다.
- **변환 결과 처리:** 저장 방식은 결정됨(요구사항 3·4 참고). 저장 창의 기본 위치·파일 이름은 원본 Markdown과 같은 폴더·같은 이름(`.typ`)으로 제안하는 것을 기본안으로 하고, 같은 이름 파일이 있을 때의 덮어쓰기 확인은 OS 저장 창의 동작을 따른다. 저장한 `.typ` 파일을 바로 탭으로 열지와 메뉴 위치(파일 메뉴/편집기 우클릭 메뉴 등)는 착수 시 사용자에게 확인한다.
- **인쇄:** GPUI에는 인쇄 API가 없으므로 PDF 생성 후 OS 인쇄 대화상자로 넘기는 방식 등 Windows/Linux/macOS별 인쇄 경로를 조사한다. PDF 저장은 저장 위치 선택 대화상자를 사용한다.
- **공통:** 모든 신규 기능은 한국어 UI 문자열(i18n 영·한), MCP 액션 제어(R3), 상수·함수·반복문 한국어 주석, 자동 테스트와 실제 GUI 검증, 자체 도움말 갱신을 포함한다. README.md의 보호된 앞 두 단락은 수정하지 않는다.

### 구현 결과 (2026-11-05)

- **조사 결과:** Typst 문법 강조는 **원본에 이미 내장**되어 있다(`languages`의 tree-sitter 문법 `codebook-tree-sitter-typst` 0.12.1 MIT, 확장자 `typ`·`typst`, 개요·들여쓰기 쿼리 포함). 별도 추가 없이 `.typ` 편집 화면의 문법 강조를 확인했다. Typst 컴파일러는 crates.io의 `typst` 0.15.1 계열(Apache-2.0, MSRV 1.92 ≤ 툴체인 1.99.1)을 라이브러리로 쓴다. md2typ는 crates.io에 없고 단일 변환 함수(`lib.rs` 약 400줄, pulldown-cmark 0.10)라서 **코드를 가져오지 않고 변환 규칙만 참고해 새로 구현**했다(제목 `=`, `#emph`/`#strong`/`#strike`, `#link`, `#quote`, `#line`, `#footnote` 등 같은 대응). md2typ의 표 변환(`columns: (left, …)`)이 Typst 문법에 맞지 않고 이스케이프가 부족해(`* _ $ < > @ ~ ` 등) 고쳐서 구현했다.
- **신규 crate `don_typst`(GUI 무관):** `markdown.rs`(Markdown→Typst 변환: 제목·문단·강조·취소선·링크·이미지(원격은 링크로)·목록(번호 유지·중첩·체크박스 글자)·표(열 정렬)·인용·각주·코드(백틱 울타리 자동 확장)·수평선·HTML 주석 제거·`<br>`, 한글이 있으면 `lang: "ko"` 서문), `world.rs`(Typst `World`: 편집 중인 글자를 원본으로, 문서 폴더 기준 파일 읽기(폴더 밖·패키지·저장 안 한 문서 거절), 글꼴 = Typst 내장 + D2Coding + 시스템 한글 글꼴(맑은 고딕 등, 있을 때만)), `don_typst.rs`(컴파일, 줄·열이 있는 진단, 쪽별 SVG, PDF). **단위 테스트 17개 통과** — 특히 변환 결과가 실제로 Typst로 컴파일되고 PDF가 만들어지는지, 오류의 줄 번호, 폴더 밖 파일 거절을 확인한다.
- **신규 crate `typst_preview`(화면):**
  - **미리보기/편집 모드:** Markdown과 같은 방식(편집기 탭의 대체 화면 슬롯)으로 Typst 파일을 열면 **쪽 미리보기로 먼저** 보여 주고 툴바 `미리보기 | 편집` 토글(명령 팔레트 `typst: toggle preview mode`)로 전환한다. 쪽은 백그라운드에서 컴파일해 SVG를 그림으로 바꿔 세로로 늘어놓고(스크롤바 포함), 편집하면 450ms 뒤 자동 재컴파일, 오류는 줄·열 번호와 함께 화면 위에 보여 주며 성공한 쪽이 있으면 그대로 둔다. 빈 파일·분할 복제본·세션 복원 규칙은 Markdown과 같다. Markdown과 Typst가 "열 때 모드 결정" 표시를 서로 가로채지 않도록 `Editor`의 API를 `alternate_view_initialized`/`mark_alternate_view_initialized`로 나눴다.
  - **키보드 스크롤:** PageUp/Down·위/아래·Ctrl+Home/End 키 바인딩을 Markdown 미리보기와 공유한다(키맵 컨텍스트 `MarkdownPreview || TypstPreview`).
  - **메뉴(파일 메뉴 → 저장 아래):** `Markdown을 Typst로 변환…`, `PDF로 내보내기…`, `인쇄…`. 액션 `typst::ConvertMarkdownToTypst {path}`·`typst::ExportPdf {path}`·`typst::Print`·`typst::TogglePreviewMode`(MCP `dispatch_action`으로 실행 가능, `path`를 주면 저장 창 없이 그 경로에 씀).
  - **사용자 결정(2026-11-04)대로:** 변환 메뉴는 **곧바로 저장 창을 띄워 `.typ`로 즉시 저장**(원본과 같은 폴더·같은 이름 제안, 확장자가 없으면 `.typ` 추가, 취소하면 아무것도 만들지 않음). PDF 내보내기·인쇄는 **Markdown을 메모리에서 Typst로 바꿔 컴파일하며 중간 Typst 파일을 저장하지 않는다**(PDF 내보내기는 결과 PDF만 저장).
  - **인쇄:** GPUI에 인쇄 API가 없어 임시 PDF(`%TEMP%\doneditor-print`, 하루 뒤 정리)를 만든 뒤 Windows는 연결 프로그램의 `Print` 동작으로, Linux/macOS는 `lp`로 보내고, 실패하면 PDF 뷰어로 열어 준다(뷰어에서 인쇄). 컴파일·저장은 백그라운드에서 한다.
- **글꼴:** 기본 글꼴 D2Coding을 앱 에셋에서 읽어 Typst에 넘기고, Typst 내장 글꼴(Libertinus Serif·New Computer Modern·DejaVu Sans Mono), Windows 맑은 고딕 등 시스템 한글 글꼴이 있으면 함께 쓴다. 변환 결과의 서문은 `("Libertinus Serif", "D2Coding")` 순서라 한글은 D2Coding(고정폭)으로 나온다 — 보기에 어색하면 서문의 글꼴 목록에 시스템 한글 글꼴을 앞세우는 개선이 가능하다.
- **라이선스 고지(사용자 확인 2026-11-04 반영):** `assets/licenses.md`에 "Typst and md2typ (Apache License 2.0)" 절(저작권 Credit·사용 범위·md2typ 코드 미사용 명시), `assets/licenses/`에 `typst-LICENSE-APACHE`·`typst-NOTICE`·`typst-assets-LICENSE-APACHE`·`typst-assets-NOTICE`(내장 글꼴 OFL 등)·`md2typ-LICENSE-APACHE`, 새로 들어온 **외부 crate 117개**의 라이선스 파일(170개, `typst-deps/`)과 목록(`typst-dependencies.md`)을 추가했다. 새 crate의 라이선스 표현식은 MIT/Apache-2.0/Unicode-3.0/BSD-2·3/Unlicense/0BSD·CC0뿐이며 GPL-3.0과 함께 쓸 수 있다. 소스 아카이브에 라이선스 파일이 없는 22개(주로 Typst 자체 하위 crate)는 목록 파일에 별도 안내했다. 내부 crate 146개 SPDX 검사 통과.
- **문서·문자열:** 자체 도움말 `docs/index.md`에 Typst·변환·PDF·인쇄 절을 추가했다. i18n 신규 키 20개(메뉴 3·미리보기/오류/인쇄 안내)를 영·한에 넣어 카탈로그는 **1,824키**다(테스트 11개 통과).
- **실제 GUI 확인 (Windows, 디버그 빌드, 격리 데이터 폴더 `target/gui-test-typst-4baf5ae8/`):**
  - `.typ`를 열면 미리보기로 시작해 한글·수식·목록·표·본문이 A4 쪽으로 조판되고(2쪽 문서, 마우스 휠·PageDown으로 이동), 상태 표시줄에 `Typst  UTF-8`이 보인다. `편집`을 누르면 Typst 문법 강조 편집 화면이 되고, 입력한 줄(`*굵은 글씨*` 포함)이 다시 `미리보기`에 반영된다.
  - 오류 문서는 "Typst 컴파일에 실패했습니다." + "3번째 줄 2열: unknown variable: unknown_function"을 표시한다.
  - 파일 메뉴에 변환·PDF·인쇄 항목이 보인다. **Markdown에서 `Markdown을 Typst로 변환…`을 누르면 곧바로 "다른 이름으로 저장" 창**(같은 폴더, 제안 이름 `변환 테스트`)이 뜨고 Enter로 `.typ`가 저장된다. 저장된 `.typ`를 열면 제목·목록·번호 목록·체크박스·인용·표(정렬)·각주·코드·수평선이 올바르게 조판된다.
  - **새 Markdown에서 `PDF로 내보내기…` → 저장 창 → Enter:** `pdf-test.pdf`(45,846바이트, PDF 1.7, 1쪽, Libertinus Serif·D2Coding 포함)만 만들어지고 **중간 `.typ` 파일은 폴더에 생기지 않았다.**
  - `인쇄…`: 임시 PDF가 만들어지고 운영체제에 넘겨졌다. 이 PC는 기본 프린터가 가상 프린터(Microsoft Print to PDF), PDF 기본 앱(Edge)에 `print` 동작이 없어 대체 경로(열기)로 갔고 Windows의 "앱 선택" 창이 떠서 닫았다(종이 출력 없음). 실제 프린터 출력과 PDF 뷰어가 `print` 동작을 지원하는 환경은 확인하지 못했다.
- **MCP 제어 확인(R3, 2026-11-05):** 공식 MCP Python SDK로 실제 앱(MCP 서버를 켠 격리 데이터 폴더, `don-mcp` stdio 브릿지)에 연결해 `tests/test_mcp_live_actions.py`로 확인했다. `list_actions`에 새 액션 7개(`typst::TogglePreviewMode`·`ConvertMarkdownToTypst`·`ExportPdf`·`Print`, `markdown::TogglePreviewMode`, `encoding_selector::Toggle`·`ReloadWithEncoding`)가 모두 등록되어 있고, `dispatch_action`은 편집기의 **"도구 실행 승인" 창**(액션 이름·인자 표시, 거절/승인)을 거쳐 실행된다. 승인 후 `typst::ConvertMarkdownToTypst {path}`가 저장 창 없이 `.typ`(1,113바이트)를, `typst::ExportPdf {path}`가 `.pdf`(45,846바이트, `%PDF-1.7`)를 만들었고 **다른 중간 파일은 생기지 않았다.** `encoding_selector::ReloadWithEncoding {"encoding":"utf-8"}`는 `windows-1252`로 잘못 감지돼 깨져 있던 `mixed.txt`(`ì²« ì¤„…`)를 `첫 줄은 UTF-8 입니다.`로 복원했다(`get_buffer_text`로 확인). 승인 창은 일부 요소가 화면 가운데 위쪽에 뭉쳐 보이는 레이아웃이지만 기능은 동작한다(이번 변경과 무관한 기존 표시 문제로 보이며 조사하지 않음).
- **clippy(2026-11-05):** 새 crate `don_charset`·`don_typst`·`typst_preview`·`encoding_selector`의 `cargo clippy --no-deps`가 **경고 0건**이다. 처음에 나온 2건(인쇄 실행의 `std::process::Command::status` 사용 금지 규칙, `len() > 0`)은 인쇄 실행을 프로젝트 표준 비동기 명령(`util::command::new_smol_command`, Windows 콘솔 창 숨김 포함)으로 바꾸고 `is_empty()`를 쓰도록 고쳤으며, 고친 뒤 인쇄 메뉴를 다시 실행해 임시 PDF가 만들어지고 운영체제로 넘어가는 것을 확인했다.
- **알려진 한계·남은 것:** 미리보기와 편집 위치는 서로 맞추지 않는다(Markdown은 맞춤). `@preview` 패키지는 지원하지 않는다. 변환 결과 파일을 저장 뒤 자동으로 열지는 않는다. Linux/macOS의 인쇄(`lp`)·시스템 한글 글꼴·실제 프린터 출력, 릴리스 크기 증가(Typst 도입 후 실행 파일 크기)는 확인하지 못했다. 디버그 빌드 기준으로 상태 표시줄의 `Failed to load environment variables`는 이전부터 있던 메시지다(콘솔 로그상 한글 경로에서 PowerShell 환경 로드가 실패하는 것으로 보이며 이번 변경과 무관해 조사하지 않음).

## 상태 표시줄 캐릭터셋 표시·선택·다시 불러오기·저장 (사용자 요청 2026-11-04 접수, 2026-11-05 구현·검증 완료 — 구현 결과는 이 절 맨 아래 "구현 결과" 참고)

### 요구사항

1. 파일을 열면 화면 오른쪽 아래 상태 표시줄의 줄 번호·프로그래밍 언어 이름(클릭하면 언어 선택) **옆에 현재 파일의 캐릭터셋을 항상 표시**한다.
2. 캐릭터셋을 클릭해 다른 캐릭터셋을 고르면 **그 파일을 고른 캐릭터셋으로 디스크에서 다시 불러온다.**
   - **용도 (2026-11-04 사용자 설명):** 파일을 불러왔는데 **이미 글자가 깨져 있을 때** 바로잡기 위한 기능이다. 또 작업에 따라 **한 파일에 여러 캐릭터셋이 섞인** 경우, 다른 캐릭터셋으로 강제로 다시 불러오기를 반복하면서 **부분씩 복원**하는 데에도 쓴다(예: EUC-KR로 다시 불러와 맞게 보이는 부분을 복사 → UTF-8로 다시 불러와 깨진 자리에 붙여 넣기).
   - **다시 불러오기는 절대 바로 저장하지 않는다.** 디스크의 파일은 그대로 두고 편집기 안의 내용만 다시 해독한다. 부분 복원은 매번 **디스크의 원본 바이트**를 다른 캐릭터셋으로 다시 읽어야 가능하므로, 복원 도중 원본이 덮어써지면 안 된다. 저장은 사용자가 직접 저장할 때만 일어난다.
   - **강제 적용:** 자동 감지 결과나 BOM과 관계없이 고른 캐릭터셋으로 해독한다. 해독할 수 없는 바이트는 대체 문자(U+FFFD)로 보이는 것이 정상이며, 오류로 막지 않는다.
3. 파일을 저장하면 **현재 선택된 캐릭터셋으로 저장**한다.

### 현재 프로그램의 캐릭터셋 처리 방식 (2026-11-04 코드 확인, 실행 검증 아님)

- **감지 (`crates/worktree/src/worktree.rs` `decode_file_text`·`decode_byte_header`·`decode_byte_full`):** 파일을 열 때 앞 1,024바이트(`FILE_ANALYSIS_BYTES`)를 먼저 보고 다음 순서로 정한다.
  1. **BOM**이 있으면 그 인코딩(UTF-8/UTF-16LE/UTF-16BE)으로 해독하고 BOM을 떼며 `has_bom = true`로 기록한다.
  2. BOM이 없으면 NUL 바이트 분포로 판단한다. 짝수/홀수 위치 NUL이 한쪽으로 몰리면 **BOM 없는 UTF-16BE/LE**, 알려진 이진 헤더(PDF·ZIP·PNG·JPEG·GIF)나 이진으로 보이는 분포면 "Binary files are not supported" 오류로 열지 않는다.
  3. 그 밖에는 전체를 **UTF-8로 해석**해 보고, 성공하면 UTF-8이다. 단 ESC(`\x1b`) 문자가 있으면 ISO-2022-JP 계열일 수 있어 다시 감지한다.
  4. UTF-8이 아니면 **`chardetng`**(국가 도메인 힌트 없음, `guess(None, …)`)로 추정한 인코딩으로 해독한다. 한국어 CP949/EUC-KR 파일은 encoding_rs의 `EUC-KR`(실제 동작은 windows-949 확장 포함)로 감지되는 것이 기대되지만, 내용이 짧으면 오감지될 수 있다(미검증).
- **보관:** 감지 결과는 `language::Buffer`의 `encoding: &'static encoding_rs::Encoding`과 `has_bom`에 저장된다(`buffer_store.rs` `open_buffer`). 새(제목 없는) 버퍼는 UTF-8·BOM 없음이다. `Buffer::set_encoding`/`set_has_bom` API는 있지만 **UI에서 호출하는 곳이 없다.**
- **저장 (`buffer_store.rs` `save_local_buffer` → `worktree.rs` `write_file`):** 버퍼의 `encoding`·`has_bom`을 그대로 써서 저장한다. 즉 **지금도 연 파일은 감지된 인코딩으로 다시 저장된다.** UTF-8(BOM 없음)은 `fs.save`로 바로 쓰고, UTF-16LE/BE는 직접 변환(BOM 선택적), 그 밖의 인코딩은 `encoding.encode()`를 쓴다(UTF-8 BOM은 직접 붙임). 줄 끝(LF/CRLF)은 저장 시 반영된다.
- **다시 불러오기 (`language/src/buffer.rs` `Buffer::reload`):** 디스크 바이트를 **버퍼의 현재 `encoding`으로 다시 해독**한다(외부에서 파일이 바뀐 경우 포함). 따라서 "인코딩 지정 → reload"의 기반은 있다.
- **상태 표시줄 (`crates/encoding_selector`의 `ActiveBufferEncoding`):** 언어 이름 바로 오른쪽(오른쪽 끝)에 인코딩 이름 버튼이 **이미 있다.** 그러나
  - 설정 `status_bar.active_encoding_button`의 기본값이 `"non_utf8"`이라 **UTF-8(BOM 없음) 파일에서는 숨겨진다.** UTF-8 BOM이면 `UTF-8 (BOM)`으로 보인다.
  - **클릭 동작이 비어 있다**(`on_click` 이 `// No-op`). crate 이름은 `encoding_selector`이지만 선택 화면은 없다.
  - 편집기를 observe해 갱신하므로, 인코딩만 바뀌고 편집기가 갱신되지 않으면 표시가 늦을 수 있다.
- **원격:** proto(`BufferState`)에 인코딩 필드가 없다. SSH/WSL 원격 프로젝트에서 클라이언트 쪽 표시·선택이 실제 원격 버퍼 인코딩과 맞는지는 착수 시 확인한다. 참고로 `language::File::load`·`Fs::load`는 UTF-8로만 읽으므로(`String::from_utf8`/`read_to_string`), 이 경로를 쓰는 기능이 있다면 비 UTF-8 파일에서 실패할 수 있다. 사용처는 착수 시 조사한다.

### 구현 시 주의할 점 (코드에서 확인한 위험)

- **저장 시 표현할 수 없는 문자:** encoding_rs의 `Encoding::encode()`는 대상 인코딩에 없는 문자(예: EUC-KR 파일의 이모지)를 **HTML 숫자 참조(`&#128512;` 형태)로 바꿔 쓰며**, 현재 코드는 오류 여부(`had_errors`)를 무시한다. 이대로면 저장 시 내용이 조용히 바뀐다. 저장 전에 검사해 경고하고 저장 중단/UTF-8로 저장 등을 고르게 해야 한다.
- **BOM 우선 해독:** `Buffer::reload`가 쓰는 `encoding.decode()`는 BOM이 있으면 지정한 인코딩 대신 **BOM의 인코딩을 쓴다**. 사용자가 고른 인코딩으로 강제로 다시 불러오려면 BOM 처리 없는 해독을 쓰고 `has_bom`도 함께 갱신해야 한다.
- **저장 안 된 변경 (2026-11-04 사용자 결정):** 변경이 있는 상태에서 다른 캐릭터셋으로 다시 불러오려 하면 **확인을 한 번 받는다**("저장하지 않은 변경을 버리고 ○○ 캐릭터셋으로 다시 불러올까요?"). 확인하면 변경을 버리고 강제로 다시 불러오며(디스크에는 쓰지 않음), 취소하면 캐릭터셋과 내용을 그대로 둔다. 참고로 기존 `reload`는 변경이 있으면 적용하지 않고 충돌 상태(`has_conflict`)만 표시하므로 이 경로를 그대로 쓸 수 없다.
- **파일을 새로 연 것과 같은 상태 (2026-11-04 사용자 결정):** 다른 캐릭터셋으로 다시 불러오면 **파일을 다시 불러온 것과 마찬가지로** 처리한다. 실행 취소 기록을 비우고, 변경 없음(디스크와 같음) 상태로 표시하며, 충돌 표시도 지운다. 기존 `reload`는 차이를 실행 취소 가능한 변경으로 적용하므로, 다시 불러오기 뒤 실행 취소 기록을 초기화하는 처리를 따로 만든다. 커서·스크롤 위치는 가능한 범위에서 유지할지 착수 시 정한다.
- **자동 저장 설정:** 사용자가 `autosave`를 켜 둔 경우 다시 불러오기 자체가 저장을 일으키지 않는지 확인한다(다시 불러오기 직후 버퍼는 디스크와 같은 것으로 표시되어야 한다).
- **대체 문자 저장 경고:** 강제 해독으로 생긴 U+FFFD가 남은 채 저장하면 원래 바이트를 잃는다. 저장 시 대체 문자가 있으면 경고하는 방안을 검토한다.
- **"다시 불러오기"와 "저장 캐릭터셋만 바꾸기"의 구분:** 요구사항 2는 디스크 바이트를 다른 캐릭터셋으로 다시 해석하는 기능이다. 반면 EUC-KR 파일을 UTF-8 파일로 **변환**하거나, 부분 복원을 마친 내용을 마지막으로 다시 불러온 캐릭터셋과 다른 캐릭터셋으로 저장하려면, 내용은 그대로 두고 저장 캐릭터셋만 바꾸는 동작(VS Code의 "인코딩하여 저장"에 해당)이 따로 필요하다. 선택 화면에서 두 동작을 함께 제공할지 착수 시 사용자에게 확인한다.
- **제공 목록:** encoding_rs가 지원하는 WHATWG 인코딩(UTF-8, UTF-8 BOM, UTF-16LE/BE, EUC-KR(CP949), ISO-2022-JP, Shift_JIS, EUC-JP, GBK, gb18030, Big5, windows-125x, ISO-8859-x, KOI8 등) 중 표시 목록과 한국어 표시 이름(예: `EUC-KR (CP949)`)을 정한다.

### 구현 방향 (초안)

- `status_bar.active_encoding_button`의 Don Editor 기본값을 `"enabled"`로 바꿔 UTF-8도 항상 표시한다(사용자 설정으로 숨김 가능 유지).
- `encoding_selector`에 언어 선택(`language_selector`)·줄 끝 선택(`line_ending_selector`)과 같은 선택 창을 구현하고, 상태 표시줄 버튼과 명령 팔레트·MCP 액션에서 연다.
- 선택 시: 미저장 변경이 있으면 한 번 확인 → `set_encoding`/`set_has_bom` → 디스크 원본 바이트를 BOM 처리 없이 지정 캐릭터셋으로 다시 해독(**디스크 쓰기 없음**) → 실행 취소 기록 초기화·변경 없음 표시 → 상태 표시 갱신. 저장은 사용자가 저장할 때만 기존 `write_file` 경로로 하고, 표현 불가 문자·대체 문자 검사를 추가한다.
- 자동 테스트에 "다시 불러오기 후 디스크 파일 바이트가 바뀌지 않음", "변경 상태에서 확인 → 강제 다시 불러오기 / 취소 → 그대로 유지", "다시 불러온 뒤 실행 취소 기록이 비고 변경 없음 상태", "두 캐릭터셋이 섞인 파일의 부분 복원 후 저장 결과"를 포함한다.
- 한국어 사용자를 고려해 `chardetng`에 한국 힌트를 주는 등 감지 개선이 필요한지, 짧은 CP949 파일 감지 결과를 실제로 확인한다.
- 공통: i18n 영·한, 한국어 주석, 자동 테스트(감지·해독·인코딩 저장 왕복·표현 불가 문자)와 실제 GUI 검증, 자체 도움말 갱신. README.md의 보호된 앞 두 단락은 수정하지 않는다.

### 구현 결과 (2026-11-05)

- **신규 crate `don_charset`:** GUI와 무관한 인코딩 공용 처리. 지원 인코딩 37개 목록(유니코드·한국어·일본어·중국어·유럽·키릴·그리스·터키·히브리·아랍·태국·베트남 계열 묶음), 이름(WHATWG 레이블)으로 인코딩 찾기, **강제 해독**(`decode_forced`: 자동 감지·BOM과 무관하게 지정 인코딩으로 읽고, 지정 인코딩과 같은 종류의 BOM만 떼어 기록, 해독 불가 바이트는 U+FFFD로 두고 알림), **표현 불가 문자 검사 인코딩**(`encode_checked`: UTF-8/UTF-16 BOM 처리 포함, 표현할 수 없는 첫 문자와 줄 번호를 돌려줌). 단위 테스트 9개 통과.
- **상태 표시줄(요구사항 1):** `status_bar.active_encoding_button` 기본값을 `non_utf8` → `enabled`로 바꿔 UTF-8 파일에서도 줄 번호·언어 이름 옆에 인코딩을 **항상** 표시한다(`UTF-8`, `UTF-8 (BOM)`, `UTF-16LE (BOM)`, `EUC-KR`, `windows-1252` 등). 사용자 설정으로 `non_utf8`/`disabled`로 되돌릴 수 있다. 표시는 편집기뿐 아니라 버퍼도 지켜봐서 다시 불러오기 직후 바로 갱신된다.
- **다른 인코딩으로 다시 불러오기(요구사항 2):** 인코딩 이름을 누르면 `encoding_selector` 선택 화면(검색 가능 목록, 현재 인코딩 체크, "이름 — 언어 계열" 표시, 명령 팔레트 `encoding selector: toggle`)이 열리고 고른 인코딩으로 디스크 파일을 `Buffer::reload_with_encoding`으로 **강제로** 다시 읽는다. 사용자 결정(2026-11-04)대로 구현했다.
  - 저장하지 않은 변경이 있으면 **확인을 한 번** 받는다(확인 → 변경 버림, 취소 → 인코딩·내용 그대로).
  - **파일을 새로 연 것과 같은 상태:** 실행 취소·다시 실행 기록을 비우고(`text::Buffer::clear_history` 신설) 변경 없음·충돌 없음 상태로 표시한다. 차이만 적용해 커서·스크롤 위치는 가능한 범위에서 유지한다.
  - **디스크에는 쓰지 않는다.** 해독 불가 바이트가 있으면 알림 토스트로 알린다. 제목 없는 새 파일·원격 파일은 안내 후 거절한다.
  - 선택 화면 없이 쓰는 `encoding_selector::ReloadWithEncoding { encoding, force }` 액션(MCP `dispatch_action`·키 바인딩용)을 추가했다.
- **저장(요구사항 3):** 저장은 기존처럼 버퍼의 현재 인코딩·BOM으로 한다(원래부터 그랬음을 코드로 확인). 추가로 `worktree::write_file`이 `don_charset::encode_checked`를 쓰도록 바꿔 **대상 인코딩으로 표현할 수 없는 문자가 있으면 글자를 바꿔 쓰지 않고 저장을 중단**하고 문제의 문자·줄 번호를 알린다(원본은 encoding_rs 기본 동작인 `&#128512;` 같은 HTML 숫자 참조로 조용히 바꿔 썼다). 기존 인코딩 저장 시험(Shift_JIS·UTF-8/16 BOM)은 그대로 통과해야 한다.
- **문자열:** 신규 26개(계열 이름 16·선택 화면 안내·저장 오류·상태 툴팁)를 영·한에 추가하고 안 쓰는 `status.encoding`을 삭제해 카탈로그는 **1,804키**다. 자체 도움말 `docs/index.md`에 "문자 인코딩(캐릭터셋)" 절을 추가했다.
- **실제 GUI 확인 (Windows, 디버그 빌드, 격리 데이터 폴더 `target/gui-test-charset-eb9f3bed/`):**
  - EUC-KR 파일을 열면 한글이 정상으로 보이고 상태 표시줄에 `EUC-KR`이 표시된다. `UTF-8`/`UTF-8 (BOM)`/`UTF-16LE (BOM)` 파일도 각각 표시됨.
  - 인코딩 이름을 누르면 선택 화면이 열리고(현재 EUC-KR에 체크) `UTF-8`을 고르면 깨진 글자(`�`)와 "해독할 수 없는 바이트" 안내가 뜨며 상태 표시줄이 `UTF-8`로 바뀐다. **디스크 파일 해시는 한 번도 바뀌지 않았고** 탭에 변경 표시도 없다. `EUC-KR`을 고르면 다시 정상으로 복원된다.
  - 다시 불러온 직후 Ctrl+Z를 눌러도 이전(깨진) 내용으로 돌아가지 않는다(실행 취소 기록 비움).
  - 변경(`ZZZ` 입력) 후 다시 불러오기 → 확인 창("저장하지 않은 변경을 버리고 UTF-8 인코딩으로 다시 불러올까요?")이 뜬다. **취소**하면 편집 내용·인코딩이 그대로, **확인**하면 편집 내용이 버려지고 변경 표시가 사라진다.
  - EUC-KR 파일에 표현할 수 없는 글자(태국어 `ก`)를 넣고 Ctrl+S → "저장하지 못했습니다 … 'ก'(10번째 줄) 문자는 EUC-KR 인코딩으로 표현할 수 없습니다. …" 오류 창이 뜨고 디스크 파일은 그대로다.
  - **섞인 인코딩 부분 복원 시나리오:** 앞뒤 줄은 UTF-8, 가운데 줄은 EUC-KR 바이트인 파일은 자동 감지가 `windows-1252`로 잘못 판단해 전부 깨져 보인다. UTF-8로 다시 불러오면 1·3번째 줄이, EUC-KR로 다시 불러오면 2번째 줄이 정상으로 보여, 인코딩을 바꿔 가며 부분씩 복원할 수 있음을 확인했다(디스크 파일은 그대로).
- **알려진 한계·남은 것:**
  - 감지 자체(`chardetng`)는 바꾸지 않았다. 짧은 파일은 오감지될 수 있고(섞인 파일은 `windows-1252`로 감지됨) 이를 사용자가 선택 화면으로 고치는 흐름이다.
  - 저장 시 대체 문자(U+FFFD)가 남아 있으면 원래 바이트를 잃는다는 경고는 아직 없다(4번 계획의 "검토" 항목).
  - **"저장 인코딩만 바꾸기"(내용은 두고 인코딩만 바꿔 저장)는 이번에 만들지 않았다.** 요구사항 밖이라 보류하며, 필요하면 사용자 확인 후 추가한다.
  - 원격(SSH/WSL) 프로젝트의 인코딩 표시·다시 불러오기, macOS/Linux 화면, 설정 화면의 `active_encoding_button` 설명 갱신, MCP로 `ReloadWithEncoding`을 실제 실행하는 확인은 남았다.
- **자동 검증:** `cargo check -p gram` 경고 0, `cargo build -p gram` 성공. `don_charset` 테스트 9개 통과. 새 테스트 4개도 통과했다: `language`의 다시 불러오기 테스트 3개(깨진 글자 복원·저장하지 않은 변경 버림·실행 취소 기록 비움·변경 없음 상태 / 해독 오류 알림·BOM 기록 / 로컬 파일 필요)와 `worktree`의 저장 거절 테스트 1개(이모지가 든 내용을 EUC-KR로 저장하면 오류이고 디스크는 그대로, 한글만 있으면 CRLF 포함 정확한 EUC-KR 바이트로 저장). 기존 `worktree` 인코딩 저장 시험(Shift_JIS·UTF-8/16 BOM)도 그대로 통과해 회귀가 없다. i18n 카탈로그 테스트 **11개 통과**(신규 키의 소스 사용·영한 일치 포함), 변경 Rust 파일 `rustfmt` 적용. 테스트 작성 중 실수 두 가지(가져오기 누락, Windows 경로 `"C:\root"`의 `\r` 이스케이프)를 고쳤고 코드 쪽 결함은 아니었다.

## 2026-11-04 Markdown 미리보기/편집 모드, 파일 연결(더블클릭) 열기 확인

- **사용자 요청:** ① md 파일을 열면 미리보기 모드부터 보여 주고, 현재 단락 제목 줄(탭 바로 아래 툴바) 오른쪽에 미리보기/편집 모드를 바꾸는 토글형 버튼을 둔다. ② 파일 확장자를 기본 프로그램으로 연결해 더블클릭했을 때 DonEditor가 바로 그 파일을 불러와 보여 주는지 확인하고, md 파일이면 열리자마자 미리보기 모드여야 한다.
- **구현 방식:** 미리보기를 별도 탭으로 여는 대신, **같은 편집기 탭이 본문 대신 미리보기를 그리게** 했다. 탭은 그대로 원래 편집기 항목이므로 저장(미리보기 상태의 Ctrl+S 포함)·변경 표시·닫기 확인·세션 복원·MCP/Agent 도구가 그대로 동작한다.
  - `editor`: `Editor`에 대체 화면 슬롯(`alternate_view`)과 `set_alternate_view`/`alternate_view`/`take_alternate_view_initialization` API, `EditorEvent::AlternateViewChanged`(→ 툴바 갱신)를 추가했다. 분할 복제본은 자동 결정 완료로 표시한다. `ScrollManager::has_pending_autoscroll`을 추가했다.
  - `markdown_preview`: `MarkdownPreviewMode::Embedded`를 추가했다. 내장 미리보기는 **편집기의 포커스 핸들을 함께 써서** 미리보기 키 바인딩(PageUp/Down 등)이 동작하고 모드를 바꿔도 초점이 유지된다. 편집기→미리보기→편집기 순환 참조를 막으려고 미리보기는 편집기를 약한 참조로 갖고, 별도 탭 미리보기만 기존처럼 강한 참조를 유지한다.
  - 신규 `markdown_preview/src/markdown_mode.rs`: 전환 함수, `markdown::TogglePreviewMode` 액션(명령 팔레트·MCP `dispatch_action`으로 실행 가능, 기본 단축키 없음), 작업 공간 `ItemAdded` 시 "열 때 미리보기로 시작", 툴바 항목 `MarkdownModeToggle`(`미리보기 | 편집` 토글 버튼, Markdown 편집기 탭에서만 표시, 툴바 오른쪽 끝). `gram.rs`의 툴바 초기화에 등록했다.
  - 문자열 4개(영·한)를 추가해 카탈로그는 **1,779키**다. 자체 도움말 `docs/index.md`(미리보기/편집 모드)와 `docs/installation.md`(Windows 연결 프로그램 지정 방법: 배포 폴더 최상위 `doneditor.exe`)를 갱신했다.
- **동작 규칙:**
  - 디스크 파일에서 연 Markdown(파일 경로·사용자 `file_types` 설정 기준, 언어가 이미 로드됐으면 언어 기준)은 미리보기로 시작한다. 파일 열기·명령줄·파일 연결·폴더 다시 열 때의 탭 복원이 모두 해당하며, 종료 직전 편집 모드였어도 다시 열면 미리보기로 시작한다.
  - 내용이 없는 파일은 편집 모드로 연다. 탭을 다른 분할로 옮기면 모드를 유지하고, 분할 복제본은 편집 모드로 연다(미리보기 옆에서 바로 편집). 제목 없는 새 버퍼는 자동 전환하지 않는다.
  - 모드를 바꿀 때 보던 위치를 잇는다(미리보기 맨 위 블록 ↔ 편집 화면 맨 윗줄, "특정 줄로 열기"처럼 대기 중인 자동 스크롤이 있으면 커서 기준). 미리보기에서 블록을 두 번 누르면 편집 모드로 바뀌고 그 위치로 커서가 간다.
  - 기존의 별도 탭/옆 분할 미리보기(`markdown::OpenPreview`·`OpenPreviewToTheSide`, 눈 아이콘)는 그대로 유지했다(R8).
- **파일 연결(더블클릭) 확인 — 코드 경로:** Windows 탐색기는 연결된 명령(`"doneditor.exe" "%1"`)으로 새 프로세스를 띄운다. `main.rs`가 경로 인자를 `file://` 요청으로 바꿔 연다. Stable 채널이면 `windows_only_instance.rs`가 `DonEditor-Stable` 뮤텍스로 첫 인스턴스인지 보고, 이미 실행 중이면 명명 파이프+IPC로 경로를 넘긴 뒤 종료한다. 받은 쪽은 `workspace::open_paths`로 기존 창에 열고 창을 활성화한다. Dev 채널(`GRAM_RELEASE_CHANNEL=dev`)은 단일 인스턴스 처리를 하지 않는다. 디버그 빌드도 기본은 Stable 채널이다.
- **실제 확인 (Windows, 디버그 빌드, 격리 데이터 폴더 `target/gui-test-mdpreview-0e4d6269/`):** 사용자 확인을 받아 사용자의 `C:\Tools\DonEditor` 인스턴스를 닫은 뒤 Stable 채널 그대로 테스트했다.
  - **실제 Shell 연결:** 사용자 기본값을 건드리지 않도록 테스트 전용 확장자 `.donmdtest`를 HKCU에 임시 연결하고(테스트 설정 `file_types`에서 Markdown으로 지정), Shell 기본 "열기" 동사(더블클릭과 같음)로 한글·공백 경로 파일을 열었다. **실행 중이 아닐 때 약 7초 안에 새 창이 그 파일을 미리보기 모드로 열었다.** 테스트 후 연결 키와 Shell이 남긴 `FileExts\.donmdtest` 기록을 모두 지웠고, 기존 `.md` 연결(UserChoice 없음)은 변경하지 않았다.
  - **실행 중일 때:** 연결 명령과 같은 명령줄로 `.md`를 열자 두 번째 프로세스가 0.7초 만에 종료 코드 0으로 끝나고, **기존 창에 새 탭으로 미리보기 모드로 열렸다.** `.txt`는 토글 없이 편집 화면으로 열렸다.
  - **토글·편집:** `편집` 클릭 → 원본 편집 화면(문법 강조, 한국어 툴팁), 초점이 편집기로 가서 바로 입력됨, 탭 변경 표시. `미리보기` 클릭 → 변경 반영. **미리보기 상태에서 Ctrl+S로 디스크 저장** 확인. 미리보기 PageDown 스크롤과 양방향 위치 잇기, 블록 두 번 누르기 → 편집(68:1로 이동), 명령 팔레트 `markdown: toggle preview mode`, 빈 `.md`는 편집 모드, 폴더 다시 열기 시 복원 탭이 미리보기, 오른쪽 분할 복제본은 편집 모드이고 그쪽 입력이 왼쪽 미리보기에 즉시 반영됨을 확인했다.
  - **관찰(이번 변경과 무관):** 상태 표시줄의 `Failed to load environment variables`(기존 메시지), 이전 프로세스가 종료되는 중 새로 시작하면 로그 파일 열기 실패 알림이 뜸, 커서가 제목 줄 0열이면 단락 제목 줄이 이전 단락을 표시(개요 범위의 기존 동작), 디버그 빌드는 콘솔 창이 함께 뜸. 기존 창이 스스로 앞으로 오는지는 캡처 스크립트가 창을 앞으로 가져오므로 따로 확인하지 못했다.
- **자동 검증:** Windows 개발자 셸에서 `cargo check -p gram --offline` 경고 0, `cargo build -p gram --offline` 성공(기존 Wasmtime `LNK4217` 링커 경고만), 변경 Rust 파일 `rustfmt --check` 통과. i18n 카탈로그 테스트 **11개 통과**(신규 키 4개의 소스 사용·영한 일치 포함). 내부 143개 crate 라이선스 고지 검사기(`script/check_don_licenses.py`) 통과. 모드 전환 위치 계산 단위 테스트 2개를 `markdown_preview_view.rs`에 추가했고, `cargo test -p markdown_preview --lib`가 **40개 모두 통과**했다(기존 파서·렌더러 테스트 38개 포함).
  - 참고: 검증 도중 PowerShell 도우미 스크립트가 `-p <crate>` 인자를 PowerShell 매개변수로 먹어 cargo가 기본 멤버(`gram`) 전체를 대상으로 돈 일이 있었다. `check`/`build`는 기본 멤버가 원래 `gram`이라 결과가 같았고, 잘못 시작된 i18n 테스트 실행은 중지한 뒤 인자를 문자열 하나로 넘기도록 고쳐 다시 실행했다.
- **남은 것:** 릴리스 빌드·배포 zip 재생성과 압축 해제본에서의 같은 확인, 탐색기 UI에서 실제 `.md`를 "연결 프로그램"으로 지정하는 수동 확인, macOS(Apple 이벤트로 파일 열기)·Linux(.desktop 연결) 확인, MCP `dispatch_action`으로 `markdown::TogglePreviewMode`를 실제 실행하는 확인.

## 2026-11-04 Don Editor 제품 버전 독립

- 사용자 요청으로 Don Editor 워크스페이스 버전을 GramEditor 기반 버전 `3.4.0`에서 독립 제품 버전 `0.1.0`으로 변경했다. Cargo.lock의 내부 워크스페이스 패키지 139개를 갱신했고 외부 의존성 및 독립 `zed_extension_api` 버전 `0.7.0`은 유지했다.
- 정보 창·시작 로그·CLI는 원본 Git 태그보다 제품 버전을 표시한다. MCP 서버 정보와 Windows/Unix 패키지 버전도 워크스페이스 값을 사용한다. README·자체 도움말·패키징 문서와 패키징 테스트 예시를 갱신했다. 원본 GramEditor 문서, 원본 기준 버전 및 과거 `3.4.0` 배포·분석 이력은 보존한다.
- Windows에서 `cargo check -p gram -p cli -p mcp_server --offline`, `cargo metadata --locked --offline`, 변경 Rust 파일의 `rustfmt --check`, Windows ZIP 테스트가 통과했다. `cargo build -p cli --bin cli --offline` 후 실제 CLI `--version`이 `Don Editor 0.1.0`을 출력했다. Linux/macOS 실행 테스트는 이전 사용자 지시에 따라 생략했다.
- 기존 `3.4.0` 배포 ZIP과 GUI 실행 파일은 변경하지 않았다. 이번에는 새 GUI 실행 파일이나 `0.1.0` 배포본을 빌드하지 않았으므로, 그 파일을 실제로 쓰려면 별도 빌드·패키징이 필요하다.

## 2026-11-04 Windows 실행 파일 기능별 용량 조사

- 사용자 요청으로 배포 EXE와 일치하는 릴리스 PDB의 섹션 기여 정보를 Microsoft DIA로 분석해 [Windows 기능별 용량 보고서](WINDOWS-SIZE-REPORT.md)를 작성했다. 내장 문법 분석기 객체 46.12 MiB와 SQL 파서 10.61 MiB 등을 확인했으며, 직접 기여량과 기능 제거 시 실제 절감량의 차이·필수 언어 제약을 문서에 명시했다.
- 분석 도구와 원본 기여 목록은 `target/dia-report-tool/`에 두었다. 프로그램 소스와 기존 배포본은 변경하지 않았고, 기능을 제거한 실험 빌드는 수행하지 않았다.

## 2026-11-04 Windows 릴리스 배포본 생성

- 사용자 요청으로 Windows x86_64 릴리스 배포본을 별도 폴더에 생성하고 그대로 보존했다. `script/package-zip.ps1 -Yes -OutDir dist/windows-20261104-172637` 실행 시 빌드 도구 점검 및 `cargo build --locked --package gram --package cli --package mcp_server --release`가 성공했다. 기존 Wasmtime 관련 `LNK4217` 링커 경고가 나왔다.
- 배포 파일: `dist/windows-20261104-172637/DonEditor-3.4.0-windows-x86_64.zip` (77,238,393바이트). SHA-256: `d0238cf31cfd19ca4dc14fc7206cd678d6c9c3a02a85728ce2fa55a534fa6079`. 동봉 `.sha256`과 별도 계산값이 일치한다.
- `dist/windows-20261104-172637/extracted/`에 압축을 풀어 GUI·CLI·MCP 실행 파일, GPL 라이선스, 소스 ZIP 존재를 확인했다. 격리된 `smoke-data`로 압축 해제된 GUI를 실행해 테스트 Rust 파일 창 생성과 응답 상태를 확인하고 창을 닫았다. 동봉된 MCP 브리지의 `--help` 실행도 확인했다. ZIP·해시·압축 해제본·테스트 데이터는 모두 보존한다.
- 이 확인은 실행 기동 스모크 테스트이며 전체 기능 회귀, 깨끗한 Windows 설치 환경이나 ARM64 검증을 의미하지 않는다. Linux/macOS 직접 테스트와 실제 외부 API 호출은 이전 사용자 지시에 따라 생략한다.

## 2026-11-04 Agent 사이드 패널 수정 (사용자 요청)

- **사용자 요청:** ① 프로그램 실행 시 Agent 패널은 기본적으로 숨김, ② 제공자 이름 클릭 토글을 콤보박스로 변경(이후 Google·OpenCode·LM Studio 추가 예정), ③ API 키는 패널이 아닌 Don Editor 기본 설정에서 제공자별로 사전 입력하고 선택 사항인 API 서버 URL(공란이면 기본값)도 입력, ④ 패널의 키 입력란은 숨기되 해당 제공자 키가 없으면 프롬프트 입력란 자리에 키 입력란을 보여 입력 즉시 기본 설정에 반영, ⑤ AI 요청에 현재 파일·폴더 경로가 컨텍스트로 들어가는지 확인.
- **① 시작 시 숨김:** 기존 `starts_open()`이 `true`라 매 실행마다 도크 복원 후 강제로 열렸다. `false`로 바꾸고, `workspace::Panel`에 기본값 `false`인 `hidden_on_startup()`을 추가해 Agent 패널만 `true`로 지정했다. 도크 복원(`Dock::restore_state`)은 저장된 활성 패널이 이 패널이면 도크를 닫고, 작업 공간 저장(`build_serialized_docks`)은 이 패널이 보이는 채로 종료해도 닫힌 상태로 기록한다. 따라서 이전 실행에서 열어 둔 사용자의 DB 상태도 다음 실행에서 숨김으로 시작한다. 다른 패널의 복원 동작은 그대로다.
- **② 제공자 콤보박스:** 패널 상단의 제공자 버튼을 `DropdownMenu` + `ContextMenu`(현재 선택 체크 표시)로 바꿨다. 항목은 `Provider::ALL`에서 만들고 실행 중에는 비활성화한다. 선택은 기존 `Configure` 경로를 그대로 써서 MCP 액션과 동작이 같다.
- **③ 기본 설정의 API 키·API 서버 URL:** 설정 화면에 **에이전트** 페이지를 추가해 OpenAI·Anthropic 절마다 **API 키**와 **API 서버 URL**을 둔다(사용자 설정에서만 표시).
  - API 키는 settings.json에 쓰지 않고 기존처럼 OS 자격 증명 저장소(`DonEditor.Agent`)에 저장한다. 평문 settings.json에 두면 MCP·Agent의 `get_settings` 도구 결과로 키가 노출되기 때문이다. 설정 UI에는 값 없는 표식 타입(`AgentApiKeyField`)과 전용 렌더러를 등록해 마스킹 입력·Enter/키 저장·키 삭제·현재 출처(저장됨/환경 변수/없음/확인 중)를 표시한다. 저장·삭제 시 세대 번호를 올려 입력란을 새로 만들어 실행 취소 기록에도 키가 남지 않게 했다.
  - 키 등록 상태는 `agent_panel::credentials`의 전역 상태로 관리하고 앱 시작 시 백그라운드에서 조회한다. 설정 창과 Agent 패널이 같은 상태를 보므로 어느 쪽에서 저장해도 다른 쪽에 즉시 반영된다.
  - API 서버 URL은 `SettingsContent.agent.{openai,anthropic}.api_url`(신규 `AgentSettingsContent`)에 저장한다. 공란이면 OpenAI `https://api.openai.com/v1`, Anthropic `https://api.anthropic.com`을 쓰고 요청 경로를 자동으로 붙인다. 키 유출을 막기 위해 `http://`는 localhost·루프백에만 허용하고 계정 정보·쿼리·조각이 든 URL은 거절한다. 전역(사용자) 설정만 읽어 프로젝트 폴더 설정이 키 전송 주소를 바꿀 수 없게 했고, `update_settings` 도구의 `agent` 항목 변경은 승인 창 전에 거절한다.
- **④ 패널의 키 입력:** 패널의 상시 키 입력란·키 저장/삭제 버튼을 없앴다. 선택한 제공자의 키가 저장소와 환경 변수 어디에도 없을 때만 프롬프트 입력란과 보내기/첨부 버튼 자리에 안내 문구·마스킹 키 입력란·**키 저장**·**설정 열기** 버튼을 보여 준다. Enter·`Ctrl+Enter`·키 저장 버튼으로 저장하면 즉시 기본 설정(OS 저장소)에 반영되어 프롬프트 입력란이 다시 나타나고 초점도 옮겨진다. 조회가 끝나기 전에는 프롬프트 입력란을 그대로 보여 준다.
- **⑤ 컨텍스트 확인 결과와 보완:** 수정 전에는 **현재 컨텍스트 첨부** 버튼을 눌렀을 때만 현재 파일 제목·경로·전체 내용·선택 영역·열린 탭이 메시지에 들어갔고, 누르지 않으면 파일·폴더 정보가 전혀 전달되지 않았다. **열린 폴더(워크트리) 경로는 첨부 버튼을 눌러도 포함되지 않았다.** 이를 보완해 `don_tools::editor_state()`를 추가하고, 매 요청(도구 호출 후 이어지는 요청 포함)마다 열린 프로젝트 폴더 경로(파일 하나만 연 경우 그 파일의 상위 폴더, 중복 제거)·현재 활성 파일의 제목/경로/저장 안 된 변경 여부·열린 탭(최대 50개) 제목/경로를 시스템 안내문 뒤에 자동으로 붙인다. 대화 기록에는 저장하지 않고 매번 새로 만든다. 파일 내용은 자동으로 보내지 않으며 모델이 도구로 읽거나 사용자가 첨부 버튼으로 넣는다.
- **문자열:** 신규 13개 키(설정 페이지·키 상태·키 필요 안내·설정 열기·잘못된 URL 오류)를 영·한에 추가하고 키 저장 안내 등 3개 문구를 고쳤다. 카탈로그는 **1,775키**다.
- **기본 설정 파일:** `assets/settings/default.jsonc`에 `agent.openai.api_url`·`agent.anthropic.api_url`의 빈 기본값을 추가했다(없으면 디버그 빌드 설정 화면에 `NO DEFAULT`가 표시됨). 설정 화면의 API 키 항목은 JSON에 쓰지 않지만 json_path를 제공자 경로(`agent.openai` 등)로 지정해, 패널의 **설정 열기**(`agent.<제공자>.api_url` 링크)에서 키·URL 항목이 함께 보이게 했다.
- **변경 파일:** `crates/agent_panel/src/{agent_panel,credentials,provider,lib}.rs`, `crates/agent_panel/Cargo.toml`·`README.md`, `crates/settings/src/{settings_content,vscode_import}.rs`, `crates/settings_ui/src/{settings_ui,page_data,components}.rs`, 신규 `crates/settings_ui/src/components/agent_api_key.rs`, `crates/settings_ui/Cargo.toml`, `crates/workspace/src/{dock,workspace}.rs`, `crates/don_tools/src/don_tools.rs`, `assets/locales/{en,ko}.json`, `assets/settings/default.jsonc`, `FEATURE-INVENTORY.md`, `Cargo.lock`.
- **자동 검증:** GUI 없는 공급자 계약 테스트 **13개 통과**(기존 9개 + API 서버 URL 기본/사용자 지정·위험 URL 거절·사용자 URL 실제 요청·편집기 상태 첨부 4개). i18n 카탈로그 테스트 **11개 통과**. Windows 개발자 셸에서 `cargo check -p gram -p settings_ui -p agent_panel -p don_tools` **경고 없이 통과**, `cargo build -p gram` 성공(기존 Wasmtime `LNK4217` 링커 경고만 있음). `cargo clippy -p agent_panel -p settings_ui -p don_tools`는 `agent_panel`·`settings_ui` 경고 0건이며, 기존 `prettier` 1건과 이번 변경 전부터 있던 `don_tools` 1건(`dispatch_action` 포커스 `if` 병합 제안)만 남았다. 변경 Rust 파일의 `rustfmt --check` 통과.
- **실제 GUI 검증 (Windows, 디버그 빌드, 격리 데이터 폴더 `--user-data-dir`):** 화면 캡처는 이번에는 .NET `CopyFromScreen` 방식으로 성공했다.
  - 새 데이터 폴더로 실행 시 Agent 패널이 **숨겨진 채 시작**하고 우측 아래 반짝임 아이콘이 남는다. 패널을 열어 둔 채 종료한 뒤 다시 실행해도 숨김으로 시작했고, 이전 빌드로 쓰던 테스트 데이터 폴더(`gui-test-feaeda…`)로 실행해도 숨김으로 시작했다. 아이콘 클릭과 `Ctrl+Alt+A`로 다시 열린다.
  - 제공자 **콤보박스**가 열려 OpenAI(체크)·Anthropic 항목이 보이고, Anthropic 선택 시 모델이 `claude-sonnet-5-5`로 바뀐다. 마지막 선택 제공자는 다음 실행에서 복원된다.
  - 키가 없을 때 프롬프트 입력란 자리에 안내 문구·마스킹 키 입력란(초점 이동)·설정 열기·키 저장이 표시된다. 키 입력 후 Enter를 누르면 Windows 자격 증명 관리자에 저장되고 즉시 프롬프트 입력란과 보내기/첨부 버튼이 돌아오며 초점도 옮겨진다. 열려 있던 설정 창에도 즉시 "저장되어 있습니다"가 표시됐다. 설정 화면의 **키 삭제**로 지우면 자격 증명이 삭제되고 패널이 즉시 키 입력란으로 돌아가며, 이전 "저장했습니다" 문구도 지워진다(검증 중 발견해 수정).
  - 설정 화면 **에이전트** 페이지에서 제공자별 API 키(마스킹·키 저장·키 삭제·출처 표시)와 API 서버 URL(공식 주소 placeholder)이 보이고, `API` 검색으로도 찾을 수 있다. 처음에는 버튼·상태 문구가 오른쪽으로 넘쳐 잘려 입력란·버튼·상태를 세로로 쌓는 고정 너비 배치로 고쳤다. 환경 변수 `ANTHROPIC_API_KEY`로 실행하면 "환경 변수 ANTHROPIC_API_KEY의 키를 사용 중"으로 표시되고 키 삭제는 비활성이다.
  - 설정 화면에서 URL을 입력하면 사용자 `settings.jsonc`의 `agent.anthropic.api_url`에 저장된다. 원격 `http://example.com`은 네트워크 요청 없이 "API 서버 URL이 올바르지 않습니다" 오류로 거절됐다. 루프백 캡처 서버(`http://127.0.0.1:18765`, 401 응답)로 바꾸면 실제 요청이 `/v1/messages`로 도착했고 `x-api-key` 헤더와 도구 33개, 시스템 안내문의 편집기 상태(열린 폴더 `proj`, 단일 파일의 상위 폴더, 활성 파일 경로, 탭 목록)를 확인했다. 외부 API 서버로는 어떤 요청도 보내지 않았다.
  - **자격 증명 정리:** 저장·삭제 확인을 위해 처음에 `DonEditor.Agent` 항목이 없음을 `cmdkey`로 확인한 뒤 가짜 값만 사용했다. 테스트 입력이 키 입력란에 들어가 OpenAI 가짜 키가 한 번 더 저장된 일이 있어 즉시 `cmdkey /delete`로 지웠고, 종료 시 `DonEditor.Agent` 항목이 하나도 남지 않았음을 확인했다. 실제 사용자 키는 읽거나 저장하지 않았다.
- **남은 확인:** 유효한 키로 실제 OpenAI/Claude 호출(이전 사용자 지시에 따라 생략), macOS/Linux의 키 저장소·화면, 릴리스 빌드에서의 같은 흐름 확인이 남았다. 테스트 데이터는 `target/gui-test-agent-802d0ebc/`에 남겨 두었다. 이 변경은 사용자가 커밋 `a2ad462`(에이전트 패널 관련)로 반영했다.

## 2026-11-04 재개 후 일시 중지 체크포인트

- **사용자 지시:** 진행 중인 Windows ZIP 오류 수정과 이미 시작한 정적 검사를 마친 뒤 문서·커밋을 정리하고 목표를 일시 정지한다. 새 작업은 시작하지 않는다.
- 실제 디버그 앱 빌드와 MCP 조회는 성공했고, ZIP 해시 오류를 수정해 실제 디버그 ZIP을 검증했다. 릴리스 ZIP과 압축 해제 실행 검증은 아직 하지 않았다.
- `cargo clippy -p don_tools -p agent_panel -p mcp_server --offline`은 종료 코드 0으로 끝났으나 경고가 남았다. 신규 코드에서는 `don_tools` 1건, `mcp_server` 5건, `agent_panel` 3건이다. 특히 `mcp_server`의 Windows ACL 처리에서 동기 `Command::output` 3건은 UI 스레드 차단 가능성을 재개 시 확인한다. 기존 `prettier` 경고 1건도 보고됐다.
- 격리 앱은 파일 창 생성과 실제 MCP 조회 당시 응답했다. Windows 화면 캡처는 재시도해도 `window capture timed out`으로 실패했고, 이후 확인 시 테스트 프로세스가 이미 종료되어 있었다. 격리 디렉터리의 `hang_traces`에 기록이 있으며 종료 원인은 확인하지 못했다. 재개 시 이 현상과 GUI 승인·거절·취소 및 패널 표시/숨김을 확인한다. 테스트 설정·파일은 `target/gui-test-feaeda1dec5a4796a547cc4ebb27460e/`에 남겨 두었다.
- 한국어 설정 경로·Git 출력·선택 단위 GUI crate 회귀 테스트, 기존 기능 인벤토리의 미확인 항목, 실제 Claude Desktop/Code 연결과 릴리스 패키지 실행 검증이 남았다. 사용자가 실제 OpenAI/Claude API 호출 검증은 생략하도록 요청했으며, Linux/macOS에서 직접 테스트해야 하는 항목도 이번 목표에서 건너뛴다.

## 2026-11-04 재개 후 Windows 검증

- 사용자 요청으로 작업을 재개했다. Linux/macOS에서 직접 테스트해야 하는 항목은 이번 작업에서 건너뛴다.
- Windows 통합 검사에서 VS Code 설정 가져오기에 새 `mcp_server` 필드 초기화가 빠진 컴파일 오류를 발견해 수정했으며, 재검사에서 `settings` crate를 통과했다. 전체 앱 검사는 계속 진행 중이다.
- 기본 셸에서는 다른 MSVC가 선택되어 Spectre 라이브러리 검사가 실패했다. `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`의 개발자 셸을 `amd64`로 불러온 뒤 검사를 재개해 해당 단계를 통과했다.
- MCP 공용 도구의 선택 영역 조회를 불변 버퍼 스냅샷과 앵커를 사용하도록 수정하고, 실행 취소 액션·전역 설정 접근의 공개 API 및 불필요한 가져오기를 바로잡았다. Windows `cargo check`에서 `don_tools`가 경고 없이 통과했다. GUI에서의 동작 검증은 아직 남았다.
- 단일 파일 편집 승인 후에도 활성 편집기가 미리보기 대상과 같은지 및 원본 파일 버퍼인지 다시 검사하도록 보강했다. Windows 실행 파일 빌드에서 `don_tools` 컴파일은 통과했으며, 실제 승인 대기 중 탭 변경 시나리오는 GUI 실행 후 검증해야 한다.
- Windows 빌드 도구 설치 스크립트 검증(15개 메뉴 시나리오)과 배포 zip PowerShell 검증이 다시 통과했다. 두 검사는 모의 설치·스테이징을 포함하며 실제 최종 앱 빌드를 대신하지 않는다.
- 실제 Windows 디버그 바이너리로 ZIP을 만들다가 `Get-FileHash` 명령을 찾지 못하는 실패가 발생했다. PowerShell 모듈 자동 로딩에 의존하지 않고 .NET SHA-256으로 계산하도록 고쳤다. 알려진 `abc` 해시와 `Get-FileHash`를 쓸 수 없는 상황을 포함한 회귀 테스트가 통과했으며, 실제 디버그 ZIP 재생성 후 해시·압축 무결성·GUI/CLI/MCP 실행 파일 3개·라이선스·수정 소스 동봉을 검증했다. 릴리스 ZIP 빌드 및 압축 해제 후 실행은 아직 남았다.
- Agent 대화 기록 선택·삭제와 패널 좌우 이동에 MCP 범용 액션을 추가했다. Windows 통합 `cargo check`에서 해당 crate가 통과했으며 실제 창과 MCP 클라이언트의 동작 검증은 남았다.
- MCP에서 Agent 대화 제목·번호를 조회하는 `list_agent_conversations` 도구를 추가했다. 변경 후 Windows `cargo check -p gram -p mcp_server`, MCP 계약 테스트 23개 및 공식 Python MCP SDK의 HTTP/stdio 자동 협상·도구 33개 조회가 통과했다. 실제 편집기 GUI와의 통합 테스트는 남았다.
- 2026-11-04 Windows 개발자 셸에서 `cargo check -p gram -p mcp_server`가 경고 없이 통과했다. 설정 가져오기·MCP 공개 API, Vim 상태 문자열·키맵 편집기·온보딩 테마·구문 트리의 번역 문자열 컴파일 오류를 수정했다. 실제 실행 파일 빌드·실행과 GUI 테스트는 별도 진행한다.
- Windows에서 `cargo build -p gram -p cli -p mcp_server --bins`가 성공해 GUI·CLI·MCP 실행 파일을 만들었다(링커의 기존 Wasmtime `LNK4217` 경고 1건). 격리 데이터 폴더와 테스트 Rust 파일로 GUI가 실행돼 파일 창이 열리고 응답함을 확인했다. 공식 MCP Python SDK로 실제 앱에 stdio 연결해 도구 33개, 활성 버퍼, Agent 기록, 패널 이동 액션 조회를 검증했다.
- Windows 화면 캡처는 창을 새로 선택해 재시도한 뒤에도 `window capture timed out`으로 실패했다. 따라서 실제 화면의 패널 조작 및 MCP 편집 승인·거절·취소 검증은 아직 완료하지 못했다. 테스트 앱·파일은 사용자 환경과 분리했다.
- 현재 프로세스 환경에서 OpenAI/Anthropic API 키 환경 변수와 Claude Code 명령은 감지되지 않았다. 실제 계정 연결은 별도 검증 대상이다.
- 사용자 지시에 따라 실제 OpenAI·Claude API 호출 검증은 이번 작업에서 생략한다. 공급자 프로토콜의 가짜 응답 계약 테스트 결과와 실제 계정 연결 여부를 구분한다.
- Windows 자격 증명 저장소는 별도 고유 테스트 항목과 가짜 값으로 저장·읽기·삭제를 검증했다. 이 검사는 keyring 백엔드 수준이며, 실제 API 키나 Agent 화면을 통한 조작은 검증하지 않았다.

## 2026-11-04 작업 체크포인트 — 사용자 요청으로 일시 중지

- **사용자 지시:** 진행 중인 작업을 정리하고 문서 반영·Git 커밋을 마친 뒤 목표를 일시 중지한다. 새 작업을 시작하지 않고 이 체크포인트에서 멈춘다.
- **전체 목표는 미완료다.** 아래는 소스 구현과 실제로 통과한 검증을 구분한 최신 상태다. 과거 단계별 기록의 미구현·대기 표시는 당시 상황이며, 재개할 때는 이 절의 남은 작업을 먼저 확인한다.

### 이번 작업과 커밋

| 커밋 | 작업 단위 |
|------|-----------|
| `f1ba957` | Linux/macOS tar.gz 배포, Windows zip의 MCP 실행 파일 및 실제 수정 소스 동봉 |
| `5fa69a6` | 한국어 자체 도움말 6개, 원본 도움말 출처·검색 통합, 신규 의존성 고지 |
| `2d7df76` | 내부 워크스페이스 SPDX·원본 GPL·원저작 고지 검증기 |
| `7a48048` | 설정 본문 704개 문구와 키맵·Git·디버거·REPL·확장·원격 등 후속 UI 한국어화 |
| `e098e1f` | 기능 인벤토리의 구현·자동 검사·실제 환경 미검증 구분 |
| `10efaee` | stdio 브릿지 병렬 응답·요청 취소·EOF 세션 취소 |
| `2b5390a` | 우측 Agent 초기 구현, OpenAI/Claude SSE·도구·서명 블록 보존·기록·OS 키 저장 경로 |
| `8de5f9e` | 공용 도구 32개, GPUI 승인 경로, 세션별 MCP 취소·인증·전송·앱 통합 및 회귀 하네스 |

- MCP는 루프백·Bearer·Host/Origin 검사, 세션별 요청 ID와 취소, TCP 종료·DELETE·서버 중지 취소, 같은 포트 재시작, Windows 기존 명시적 ACL 초기화를 구현했다. 전송은 16 MiB, 중복 인코딩하는 도구 결과는 4 MiB로 제한한다.
- 편집 승인은 대상 파일의 전체 변경안을 표시하며, 승인 대기 중 변경된 파일·활성 탭·읽기 전용 상태·발췌 편집기를 검사한다. MCP 취소 토큰은 승인 직후에도 재검사한다. 이 **GPUI 경로는 아직 전체 컴파일·실행 검증 전**이다.
- Agent는 기본 우측·숨김·항상 남는 도크 아이콘, 키 마스킹/OS 자격 증명 저장/환경 변수 폴백, Markdown 스트리밍·중단·컨텍스트·최근 20개 대화와 MCP 공개 액션 8개를 구현했다. 자세한 체크포인트는 `crates/agent_panel/README.md`에 기록했다.
- macOS 번들/창 식별자와 원본 인스턴스 탐색을 분리했다. 프로젝트 `.gram/` 호환 설정, D2Coding UI 글꼴, 임시 D 아이콘은 유지한다. 별도 기여 정책·행동 강령은 새로 제정하지 않았다. 소스 공개 위치 미정 문제는 배포 스크립트의 수정 소스 동봉으로 대응한다.

### 완료한 검증

- **i18n:** 영·한 **1,762키**, GUI 없는 카탈로그 테스트 **11개 통과**. 키·자리표시자·실제 소스 사용 여부·언어 격리 검사 포함. 마지막 MCP 미리보기 표제 2개를 추가한 뒤 재검사했다.
- **MCP 실제 소스 계약:** `don_protocol_tests`의 기본/세션/입력/UTF-8/HTTP **14개**, stdio 병렬·취소·EOF **4개**, 실제 TCP 취소·종료·DELETE·재시작·크기 제한 **5개**가 모두 통과했다. 기본 실행에서 제외하는 Windows ACL **1개**도 명시적으로 실행해 Everyone 명시적 ACE 제거를 확인했다(총 24개). GUI 대신 고정 테스트 실행기를 사용한다.
- **외부 클라이언트:** 공식 Python MCP SDK **2.3.0**으로 HTTP·stdio 각각 legacy 초기화와 자동 협상, 도구 **32개**·리소스 **3개** 조회/호출을 검증했다. `tests/test_mcp_sdk.py`가 재현 스크립트이며 실제 에디터 GUI 연결이나 Claude Desktop/Code 실행을 대신하지 않는다.
- **Agent:** 실제 `provider.rs`를 GUI 없이 재사용한 공급자 계약 테스트 **9개 통과**. SSE·한글·분할 도구 인자·실패/잘린 응답·중단 재개·Claude thinking/signature 보존을 확인했다. 실제 키·유료 API 호출은 사용하지 않았다.
- **배포:** Unix 패키징 **14개**, 기존 설치 스크립트 **19개**, Windows zip PowerShell 검사 통과. 모의 빌드/설치와 실제 tar·해시·소스 스테이징을 구분했다. 신규 앱의 최종 실행 파일을 만든 검증은 아니다.
- **문서/라이선스:** 자체 도움말 링크 **76개**·JSON 예시·원본문서 불변 검사, 내부 **143개 crate**의 SPDX/GPL/원저작 고지 검사, 검사기 회귀 **10개** 통과. 신규 외부 의존성은 원본 crate와 Cargo.lock SHA256을 확인해 개별 고지했다.
- 변경 Rust 파일의 rustfmt 및 Git diff 공백 검사가 통과했다. 로그와 Python SDK용 가상 환경은 Git에서 제외하는 `target/`에 보관한다.

### 남은 작업과 재개 순서

1. **전체 GUI 컴파일부터 재개한다.** `cargo check -p gram -p mcp_server`는 공용 의존성 컴파일 중 사용자 중지 요청에 맞춰 중단했다. 종료 코드 1은 강제 중단 결과이며 빌드 성공을 의미하지 않는다. 신규 GUI 바이너리·릴리스 zip/tar.gz는 아직 만들지 않았다. 이후 바뀐 기본 feature(i18n/Agent)와 매니페스트를 다시 읽도록 새 Cargo 명령으로 시작한다.
2. `don_tools`·MCP의 GPUI 연결과 Agent UI의 타입/빌드 오류를 해결하고 변경 GUI crate 테스트·clippy를 실행한다. 추가한 설정 JSON 경로·한국어 Git 처리·커서 선택 단위 회귀 테스트도 실행 전이다.
3. 실제 앱에서 편집 승인/거절·취소·오래된 변경안 거절, 설정·언어 전환, Agent 기본 위치·숨김·아이콘 복귀·기록 복원, 원본/자체 도움말을 확인한다. MCP의 모든 기능 제어 완전성은 검증 전이며 특정 대화 선택·기록 삭제·패널 이동의 JSON 액션도 보강 대상이다.
4. OS 키 저장/삭제와 실제 OpenAI 또는 Claude 호출, Claude Desktop/Code 실제 앱 연결을 검증한다. 사용자 API 키는 이번 작업에서 읽거나 등록하지 않았다.
5. 기능 인벤토리의 기존 실행 확인 **22개/미확인 28개**(2026-11-05 갱신)를 기준으로 회귀 확인한다. 신규 소스의 전체 기능·시각 검증이 완료됐다고 표시하지 않는다.
6. 신규 코드로 Windows 최종 zip과 각 OS의 Linux/macOS tar.gz 생성·압축 해제·실행, 깨끗한 환경 설치 흐름·ARM64를 확인한다. WSL은 후반에 `CreateVm/ERROR_TIMEOUT`으로 시작에 실패해 추가 Linux 실행 검증을 하지 못했다. macOS 환경도 사용하지 않았다.

빌드 환경 참고: 지정 1.99.1 툴체인의 자동 동기화 중 Windows 디렉터리 이름 변경/롤백 오류가 있었다. 설치된 `stable`의 rustc/cargo도 **1.99.1**임을 확인했다. 검사 시 `RUSTUP_TOOLCHAIN=stable`, `CARGO_BUILD_JOBS=1` 또는 `2`로 메모리를 제한했다. 컴파일 캐시는 보존한다.

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
$env:CARGO_BUILD_JOBS = '2'
cargo check -p gram -p mcp_server
# 아래는 GUI 없이 재현 가능한 검사다.
cargo test -p i18n --no-default-features --lib --offline --target-dir target/i18n-check
cargo test -p agent_panel --no-default-features --lib --offline --target-dir target/contracts
cargo test -p don_protocol_tests --offline --target-dir target/i18n-check
```

### 이전 단계별 기록 (아래의 대기·미구현 표시는 당시 기준이며, 위 진행 기록으로 순차 갱신)

- 최종 갱신: 2026-11-04 (체크 항목 최신화)
- **2026-11-04 현재 상태 요약:** Phase 0~5는 완료 또는 1차 완료, Phase 6(MCP)·Phase 7(Agent 패널)은 구현과 Windows 실제 실행 검증을 대부분 마쳤다(각 절 체크 항목 참고). 남은 것은 실제 Claude Desktop/Code 연결, 유효한 키를 쓰는 실제 API 호출(사용자 지시로 생략), Phase 8 통합 회귀·릴리스 배포본 검증, Linux/macOS 실제 OS 검증, 사용자 결정 사항이다. 아래 '현재 단계' 문구는 2026-10-28 당시 기록이다.
- (2026-10-28 기록) 현재 단계: **Phase 5(한국어 UI) 1차 구현 완료 — 기반(`crates/i18n`·`ui_language` 설정)과 주요 UI(메뉴·패널·탭·컨텍스트 메뉴·검색·상태바·대화상자·명령 팔레트·설정 화면의 언어 선택)를 한글화, 자동 테스트 통과. 설정 화면 본문·키맵 편집기·Git 패널·디버거 등은 아직 영어(Phase 5 "남은 것" 참고). 디버그 빌드를 실제로 실행해 창을 캡처해 한국어 메뉴·우클릭 메뉴가 정상 표시됨을 확인했다(확인 범위는 Phase 5 "실제 화면 확인" 참고)**. Phase 4·3·2 의 사용자 확인·결정 요청도 그대로 남아 있음 → 다음 후보는 Phase 5 2차(남은 화면), Phase 6 (MCP), Phase 3 후속(Linux/macOS tar.gz 배포)
- Phase 1 완료 ✅ (디버그·릴리스 빌드와 실행, GUI 확인, 기능 인벤토리, 기준선 태그 `gram-baseline-3.4.0`)
- Phase 2 구현·검증 완료 ✅, 사용자 시각 확인 대기 (식별자·경로·실행 파일명·표시 이름·글꼴·라이선스 고지 적용, 원본 빌드와 동시 실행 검증)
- Phase 3·4 구현·검증 완료 ✅ (2026-10-27), Phase 5 1차 구현 완료 (2026-10-28). **현재 저장소에는 Phase 5 소스(`crates/i18n/`·`assets/locales/` 등)가 커밋 `f061f14` 에 포함되어 있다. 이전의 "Phase 5 미커밋" 안내를 정정한다.** 아래 과거 작업 기록의 커밋 번호·검증 결과는 당시 환경 기준으로 보존한다.
- 2026-11-03 소스·빌드 환경 점검: Cargo workspace 에 선언된 139개 구성 요소의 디렉터리와 매니페스트가 모두 존재하고 핵심 소스·자산도 확인했다. 단, `AGENTS.md` 의 원본 경로 `D:\Workspace\git\GramEditor` 는 이 PC 에 없어 원본과의 전체 대조는 하지 못했다. 배포 스크립트 `-CheckOnly` 결과 Git·CMake·Windows SDK 10.0.26100 은 준비됨, `rustup`/Rust 1.99.1 및 MSVC Spectre 완화 라이브러리는 없음으로 빌드 환경 미완비.
- 2026-11-04 **빌드 도구 번호 선택 설치 구현 완료:** Windows `install-build-tools.bat` + `script/install-build-tools.ps1`, Linux/macOS `install-build-tools.sh` 추가. 처음에는 전부 미선택이고, 번호로 항목을 고른 후 `I`를 입력해야 설치한다. 조회·모의 실행·취소 지원. 셸 테스트 19개와 Windows 메뉴 15개 시나리오·Rust 구성 요소 검사 통과. 실제 패키지 설치는 수행하지 않았다. 자세한 내용은 아래 절 참고.
- 2026-11-04 **AGENTS.md 변경 문서 반영 완료:** R4에 패널 숨김·별도 아이콘 클릭으로 다시 표시를 명시하고, R5는 Windows zip으로 구분했다. Linux/macOS tar.gz 배포는 **R9**로 추가해 설계·Phase 3 후속·Phase 8·[기능 인벤토리](FEATURE-INVENTORY.md)에 반영했다. R4와 R9는 미구현이며, `install-build-tools.sh`만으로 R9를 충족하지 않는다. 현재 PC(HJOW-Athlon)의 원본 소스 경로는 `H:\Workspace\git\GramEditor`이며 디렉터리 존재만 확인했다(전체 대조 미수행). 이번 작업은 두 문서만 수정했으며 앱 빌드·실행 테스트는 하지 않았다.

### 빌드 도구 선택 설치 (2026-11-04)

- **사용자 요청:** 빌드 프로그램을 자동 설치하는 Windows bat 및 Linux/macOS sh를 별도로 준비하되, 실행 즉시 설치하지 않고 프로그램 목록과 설치 여부를 번호로 선택하게 한다.
- **추가 파일:** 루트 `install-build-tools.bat`(Windows PowerShell 5.1 본체 호출), `script/install-build-tools.ps1`(한국어 메뉴·설치), 루트 `install-build-tools.sh`(Bash 3.2 호환), `tests/test_install_build_tools.ps1`, `tests/test_install_build_tools.py`. README에 실행법·선택 방법·지원 환경·한계 설명 추가. GPL-3.0 계열 라이선스와 기존 원저작 고지 유지.
- **공통 조작:** `1 3`/`1,3`으로 선택 전환, 같은 번호 재입력으로 해제, `A`로 미준비 항목 전체 선택, `N`으로 전체 해제, `I`로 설치 시작, `0`/빈 입력/EOF로 취소. 준비된 도구는 건너뛴다. 잘못된 번호가 섞이면 선택 상태를 바꾸지 않는다. 선행 도구가 미선택이면 함께 선택하도록 안내한다. 패키지 관리자가 설치하는 각 도구의 필수 하위 의존성은 목록 항목과 함께 설치될 수 있다.
- **Windows:** Git·CMake·MSVC/Spectre·Windows SDK·rustup·프로젝트 툴체인 6항목. WinGet 및 기존 Visual Studio Installer를 사용하며, MSVC/SDK는 개별 선택하되 동시에 선택하면 한 번에 설치한다. VS 설치 경로의 공백 인용, ARM64용 MSVC 구성 요소, SDK 헤더/라이브러리/rc/fxc 파일 점검, 표준 Rust/Git/CMake 및 VS 내장 CMake 경로 탐색 포함. WinGet 미설치 시 설치 주소 안내. rustup은 공식 exe와 SHA256을 받아 검증하고 MSVC 자동 설치 제안을 막는다. bat는 ASCII/CRLF, ps1은 UTF-8 BOM/CRLF로 저장.
- **Linux:** apt-get, dnf, pacman, zypper 지원. curl/인증서·Git·CMake·C/C++/Clang/libclang/pkg-config 묶음·GUI/시스템 라이브러리 묶음·rustup·프로젝트 툴체인 7항목. 묶음의 패키지 이름을 모두 표시하며 설치 상태는 로컬 패키지 DB로 확인한다. sudo는 시스템 패키지 설치 시에만 호출하고, apt 목록 갱신도 설치 확정 뒤 한 번만 수행한다. 배포판별 추가 저장소를 자동 활성화하지 않으므로 일부 버전에서는 수동 준비가 필요할 수 있다.
- **macOS:** Xcode/SDK/Metal·Homebrew·Git·CMake·rustup·프로젝트 툴체인 6항목. 선택 전 Git/Clang 스텁을 실행해 개발 도구 설치 창이 뜨는 것을 방지한다. **Xcode 본체 설치는 Apple 인증·초기 실행을 사용자가 완료해야 한다.** 선택 시 App Store를 열고 미완료 종료 코드 2를 반환한다. 설치된 Xcode는 개발 도구 경로·초기 구성을 설정하고 필요한 Metal 도구를 다운로드한다. Homebrew 설치 전에 Xcode/명령줄 도구가 있는지도 다시 검사한다.
- **Rust:** `rust-toolchain.toml`의 버전·프로필·모든 구성 요소(rustfmt/clippy/rust-analyzer)·추가 타깃(wasm32-wasip2)을 읽어 설치 및 재검사한다. 조회 중 cargo/rustc를 호출하지 않으며, rustup 조회도 설치된 툴체인에만 수행해 암묵적 다운로드를 피한다. rustup 자체는 `--default-toolchain none`으로 별도 설치하고 프로젝트 설치 시 전역 기본 Rust 버전을 바꾸지 않는다.
- **검증:** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File tests\test_install_build_tools.ps1` 통과(Rust 설치 인자·누락 구성 요소·누락 타깃·안전한 rustup 상태 조회·메뉴 15개 시나리오). WSL에서 `python3 -m unittest discover -s tests -p test_install_build_tools.py -v` **19개 통과**. 선택/해제/중복/잘못된 입력/EOF, 선택 항목만 설치, 준비된 도구 제외, 의존성 미선택 차단, 조회/모의 실행, 설치 실패, 배포판 4종의 패키지 명령, macOS 설치 전 대화상자 방지·외부 설치 미완료·Metal·CMake 경로를 가짜 명령으로 검증했다. Bash 구문 검사 및 PowerShell 파서 검사 통과.
- **구현 중 보완:** [rustup 소스](https://github.com/rust-lang/rustup/blob/main/src/cli/rustup_mode.rs)의 버전 표시가 활성 툴체인 확인 함수를 호출함을 확인했다. 일부 버전의 암묵 설치 가능성을 피하려고 상태 조회는 `rustup --version` 대신 `rustup --help`를 사용하며, 이 경로의 회귀 테스트도 추가했다.
- **실제 환경 조회:** Windows에서 bat의 `-Help`, `-CheckOnly`, Rust 관리자/툴체인 `-DryRun` 실행 확인. 이 시점에는 Git·CMake·MSVC/Spectre·SDK가 준비됐고 rustup/프로젝트 툴체인은 없음. WSL에서도 `--check-only`로 Linux 목록이 정상 표시됨을 확인했다. **실제 설치·다운로드·관리자 권한 상승은 하지 않았으며, 깨끗한 Windows/macOS/Linux 및 ARM64에서의 설치/빌드는 미검증이다.** 에디터 소스 변경이 없어 Cargo 빌드/테스트는 실행하지 않았다.
- 기존 `package-zip.bat`의 배포 흐름은 그대로다. 이 스크립트는 도구 준비만 담당한다. AGENTS.md에 추가된 Linux/macOS tar.gz 배포 및 Agent 패널 숨김/아이콘 요구사항은 별도 후속 구현 대상이다.

### 사용자 결정·확인 요청 (Phase 2)

1. **시각 확인:** 새 빌드(`target\debug\doneditor.exe`)에서 ① 글꼴이 D2Coding 인지(한글 포함), ② 메뉴 이름·정보 대화상자가 "Don Editor"인지, ③ 작업 표시줄·창 아이콘(임시 "D")이 보이는지
   - **아이콘 색 주의 (Phase 3 에서 발견):** `crates/gram/build.rs` 는 빌드 시 환경 변수 `RELEASE_CHANNEL` 이 없으면 **dev 아이콘(주황)** 을 exe 에 넣는다. Phase 1·2 의 수동 `cargo build` 는 이 변수 없이 빌드했으므로 그때의 exe 는 주황 "D" 였을 것이다(코드로 추론, 당시 exe 는 이미 덮어써져 직접 확인 불가). 배포 스크립트는 `crates/gram/RELEASE_CHANNEL`(stable) 값을 `RELEASE_CHANNEL` 로 지정해 빌드하며, 그 결과물 아이콘이 파랑(stable)임을 픽셀 색으로 확인했다. 수동으로 빌드해 아이콘을 보려면 `$env:RELEASE_CHANNEL='stable'` 을 설정한 뒤 `cargo build`
2. **UI 글꼴:** 메뉴·패널 같은 UI 글자도 D2Coding(고정폭)으로 했다. 어색하면 `assets/settings/default.jsonc` 의 `ui_font_family` 만 되돌리면 된다 (3.8 참고)
3. **프로젝트 폴더 `.gram/` 유지 여부:** 프로젝트별 설정 폴더를 원본과 공유하도록 그대로 두었다. `.doneditor/` 로 분리할지 결정 필요
4. **아이콘:** 임시 "D" 아이콘/로고를 실제 디자인으로 교체할지
5. **소스 공개 위치:** GPL 의무를 위해 배포 시 Don Editor 소스를 어디에 공개할지 (`assets/licenses.md` 에 적을 내용)

---

## 1. 프로젝트 개요

GramEditor(Zed 하드포크, Rust/GPUI 기반)를 복제·커스텀하여 아래 요구사항을 만족하는 텍스트 편집기 **Don Editor**를 개발한다.

| # | 요구사항 | 요약 |
|---|----------|------|
| R1 | 한국어 UI | 메뉴·패널·대화상자·설정 화면 등 UI 문자열 한국어화, 언어 전환 지원 |
| R2 | 문법 강조 | **26개 언어** — java, c, cpp, sql, c#, rust, javascript(mjs, cjs 포함), typescript, python, ruby, json, json5, yaml(yml), properties(java.util.Properties 호환), xml, html, xhtml, markdown(md), swift, scala, r, bat(Windows Batch), powershell, sh(Unix Shell), go, dart — 파일 확장자로 언어 감지 (2026-10-27 AGENTS.md 에서 10개 → 26개로 확대) |
| R3 | 로컬 MCP | 프로그램의 모든 기능을 외부 MCP 클라이언트에서 제어 |
| R4 | Agent 사이드 패널 | OpenAI 또는 Claude API Key 연동, 기본 우측 배치. 패널을 숨길 수 있고, 숨긴 상태에서도 화면 내 별도 아이콘을 클릭해 다시 표시 가능 (2026-11-04 AGENTS.md 변경) |
| R5 | Windows 배포 zip 스크립트 | 별도 bat/ps1 파일. 호출 시 빌드 도구 존재 여부 검사 및 미설치 시 설치 여부 질의 (2026-11-04 Windows 대상으로 명확화) |
| R6 | 기본 글꼴 D2Coding | `assets/fonts/D2Coding-Ver1.4.0-20261003-all.ttc` 사용 (2026-10-27 AGENTS.md 추가) |
| R7 | 라이선스 GPL-3.0 | 원본(GramEditor) 라이선스에 따라 GPL-3.0 유지, 원본 라이선스·저작권 명기 (2026-10-27 AGENTS.md 추가) |
| R8 | 기존 기능 보존 | "그 외 기존에 존재하던 기능은 최대한 포함한다" — 원본 GramEditor 의 기능을 삭제·비활성화하지 않음 (2026-10-27 AGENTS.md 추가, 3.10 참고) |
| R9 | Linux/macOS 배포 tar.gz 스크립트 | 별도 sh 파일. 호출 시 빌드 도구 존재 여부 검사 및 미설치 시 설치 여부 질의 (2026-11-04 AGENTS.md 추가, 미구현) |

> **프로젝트 방향 (2026-10-27 사용자 확인):** 원본 GramEditor 는 AI 를 사용하지 않는 것을 핵심 방침으로 삼는 편집기이지만, Don Editor 는 **AI 기능을 적극 도입**한다.
> 이것이 원본과 달라지는 **가장 큰 차이점**이며, R3(MCP)·R4(Agent 패널)가 그 중심이다. AI 관련 기능은 이후 추가 확장될 수 있다.
> 따라서 (1) 모든 문서(README 등)에 이 차이를 가장 먼저 밝히고, (2) 원본에서 가져온 "AI 금지·인간 기여" 정책 문서는 이 프로젝트의 정책이 아니므로 정리하며, (3) AI 기능은 이후 확장이 쉽도록 공용 구조(`don_tools` 등)로 설계한다.

---

## 2. 기반 프로젝트(GramEditor) 분석 결과

현재 PC(HJOW-Athlon)의 원본 소스 경로: `H:\Workspace\git\GramEditor` ([원본 저장소](https://codeberg.org/GramEditor/gram)). AGENTS.md의 링크 표기에는 이전 경로 `D:\Workspace\git\GramEditor`도 남아 있다. 아래 분석과 파일 복사 방식의 이관 기록은 이전 환경 기준이며, 현재 H: 경로의 소스 전체를 다시 대조한 결과는 아니다.

| 항목 | 내용 |
|------|------|
| 언어/프레임워크 | Rust (toolchain `1.99.1`, `rust-toolchain.toml`), UI 프레임워크 GPUI |
| 규모 | `crates/` 하위 135개 crate, 워크스페이스 버전 `3.4.0` |
| 라이선스 | **GPL-3.0** → 파생물도 GPL-3.0 유지, 배포 시 소스 제공·저작권 고지 필요 |
| 진입점 | `crates/gram/src/main.rs` (각 crate의 `init()` 호출 순서 정의) |
| 메뉴 | `crates/gram/src/gram/app_menus.rs` — 영어 문자열 하드코딩 |
| 다국어(i18n) | **프레임워크 없음** (워크스페이스에 `sys-locale` 의존성만 존재) |
| 문법 강조 | `crates/languages/src/<언어>/` (config.toml + *.scm 쿼리), `languages::init()`의 `register_native_grammars`로 tree-sitter 문법 등록 |
| 패널 | `workspace::Panel` 트레이트 — `project_panel`, `outline_panel`, `git_panel`, `terminal_panel`, `debugger_panel` 구현 참고 가능, `DockPosition::Right` 지원 |
| AI 기능 | Zed의 AI/Agent 관련 crate 전부 제거된 상태 → Agent 패널은 **신규 구현** |
| HTTP | `http_client`, `reqwest_client` crate 존재 → LLM API 호출에 재사용 |
| 자격 증명 | GPUI에 credentials API 없음 → API Key 저장용 `keyring` crate 신규 도입 필요 |
| 데이터 경로 | `crates/paths` — Windows `%APPDATA%\Gram`, `%LOCALAPPDATA%\Gram` |
| 기본 설정/키맵 | `assets/settings/default.jsonc`, `assets/keymaps/default-windows.jsonc` |
| 폰트 | `assets/fonts` (fira-sans, myna, openmoji) — **한글 글리프 없음**. 기본 글꼴은 별칭 `.GramMono`(→Myna), `.GramSans`(→Fira Sans)로 지정되며 매핑은 `crates/gpui/src/text_system.rs:894`, 로드는 `crates/assets/src/assets.rs:55`, `crates/gram/src/gram.rs:4355`, 기본값은 `assets/settings/default.jsonc`(`buffer_font_family`, `ui_font_family`). D2Coding(`.ttc`)은 사용자가 이미 `assets/fonts`에 추가함 |
| Windows 빌드 | `script/bundle-windows.ps1` (Inno Setup 설치본 + 코드 서명 전제), 원본 [Windows 개발 문서](GramEditor-docs/docs/development/windows.md) — 원문에 "Windows 빌드가 동작하는지 알 수 없음"이라 명시됨 |
| 원본 문서 | 원본의 README·CONTRIBUTING·CODE_OF_CONDUCT·SECURITY·CHANGELOG·`docs/` 는 [GramEditor-docs/](GramEditor-docs/) 에 원본 그대로 보관 (2.2 참고) |
| Windows 빌드 필요 도구 | rustup, Visual Studio(또는 Build Tools) MSVC x64 + Spectre-mitigated libs(**필수** — `languages` 가 쓰는 Microsoft `python-environment-tools`(`pet*` crate)가 `msvc_spectre_libs` 를 `features = ["error"]` 로 의존하므로, 미설치 시 빌드 스크립트가 패닉하여 빌드 실패. Phase 1 빌드로 확인), Windows SDK ≥ 10.0.20348, CMake |
| 빌드 플래그 | `.cargo/config.toml` — Windows는 `-C target-feature=+crt-static`, `--cfg windows_slim_errors`. 기본 빌드 대상은 `default-members = ["crates/gram"]` |

### 2.1 요구 언어(26개)별 현재 지원 현황

조사 기준(2026-10-27): `crates/languages/src/*/config.toml` 과 `languages::init()` 의 `register_native_grammars` 목록. 원본 문서([languages.md](GramEditor-docs/docs/languages.md))에서도 java·c#·ruby·swift·scala·r·powershell·dart 는 "내장 아님, 확장 기능으로 제공"으로 분류되어 있다.

**결과: 원본 내장 15개(확인·보강만) + 신규 추가 11개 = 26개**

| # | 언어 | 원본 내장 | 현재 등록 확장자 (config.toml) | 조치 |
|---|------|-----------|-------------------------------|------|
| 1 | c | O | `c` | 유지 |
| 2 | cpp | O | `cc, cpp, cxx, h, hpp, hxx, ...` | 유지 (`.h` 는 C/C++ 공유 — 동작 확인) |
| 3 | sql | O | `sql` | 유지 |
| 4 | rust | O | `rs` | 유지 |
| 5 | javascript | O | `js, jsx, mjs, cjs` | 유지 — **mjs, cjs 는 이미 포함**(문법은 tsx 사용). 동작 확인 |
| 6 | typescript | O | `ts, cts, mts` (+ tsx 별도) | 유지 |
| 7 | python | O | `py, pyi, mpy` | `pyw` 추가 검토 |
| 8 | json | O | `json, geojson, ...` (jsonc 별도) | 유지 |
| 9 | yaml (yml) | O | `yml, yaml, ...` | 유지 |
| 10 | xml | O | `xml` (첫 줄 `<?xml` 감지) | 유지 — `xsd, xsl, xslt, svg` 등 확장자 보강 검토 |
| 11 | html | O | `html, htm, shtml, etlua, xhtml` | 유지 |
| 12 | xhtml | O (html 에 포함) | `xhtml` | 유지 — 동작 확인 |
| 13 | markdown (md) | O | `md, mdx, mdwn, mdc, markdown, MD, gentoo` | 유지 |
| 14 | sh (Unix Shell) | O | `sh, bash, zsh, ...` + 셸뱅 (언어명 "Shell Script", bash 문법) | 유지 |
| 15 | go | O | `go` (+ go.mod, go.work 별도) | 유지 |
| 16 | **java** | X | — | **신규** `java` |
| 17 | **c#** | X | — | **신규** `cs`, `csx` |
| 18 | **ruby** | X (문법 crate 는 워크스페이스에 선언만 됨, `language` crate 테스트에서만 사용) | — | **신규** `rb, rake, gemspec, ru` + `Gemfile`, `Rakefile` |
| 19 | **json5** | X | — | **신규** `json5` (문법 crate 호환성 문제 있음 — 3.4 참고) |
| 20 | **properties** | X (`ini` 언어는 있으나 `properties` 확장자 없음) | — | **신규** `properties` |
| 21 | **swift** | X | — | **신규** `swift` |
| 22 | **scala** | X | — | **신규** `scala, sc, sbt` |
| 23 | **r** | X | — | **신규** `r, R` |
| 24 | **bat (Windows Batch)** | X | — | **신규** `bat, cmd` |
| 25 | **powershell** | X | — | **신규** `ps1, psm1, psd1` |
| 26 | **dart** | X | — | **신규** `dart` |

참고(원본 자체 이슈): `ini`, `kdl` 은 `crates/languages/src/` 에 설정·쿼리 폴더가 있으나 `register_native_grammars` 에 **문법이 등록되어 있지 않다**(워크스페이스 `Cargo.toml` 에도 문법 crate 없음). 실제로 강조가 되는지는 앱 실행 후 확인 필요 (3.10 기능 인벤토리에서 "원본 자체 동작 여부"로 기록).

### 2.2 원본 문서 보관 (GramEditor-docs) — 2026-10-27

원본 GramEditor 의 문서는 Don Editor 의 문서와 섞이지 않도록 [GramEditor-docs/](GramEditor-docs/) 폴더에 **내용 수정 없이 그대로** 옮겼다. 우리 쪽 문서([README.md](README.md), 이 문서)에서 원본을 설명하는 단락에는 이 폴더로 링크를 건다.

| 원래 위치 (GramEditor) | 현재 위치 |
|------------------------|-----------|
| `README.md` | [GramEditor-docs/README.md](GramEditor-docs/README.md) (원래 이름 유지. 루트 `README.md` 는 Don Editor 소개로 새로 작성) |
| `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md` | [GramEditor-docs/](GramEditor-docs/) 바로 아래 |
| `docs/` (131개 `.md`) | [GramEditor-docs/docs/](GramEditor-docs/docs/) |
| `assets/images/docs_logo.png`, `docs_editor.png`, `seal_of_human_authorship.svg` | `GramEditor-docs/assets/images/` (원본 README 에서만 쓰이던 이미지 3개. 코드 참조 없음) |

- **검증:** 이동한 파일 136개는 GramEditor 원본 소스와 바이트 단위로 동일함을 확인했다(`diff -r`, `cmp`). 원본과 같은 상대 구조를 유지해 문서 간 링크는 그대로 동작한다.
- **`LICENSE` 와 `assets/licenses.md` 는 이동하지 않았다.** GPL-3.0 고지는 저장소 루트와 앱 내장 파일로 유지해야 한다.
- **앱 코드 영향 (중요):** 루트 `docs/` 는 앱 바이너리에 내장되어 앱 내 도움말(`gram://docs/...`)로 쓰인다. 그래서 다음을 함께 수정했다. 모두 `[DonEditor]` 표시.
  - `crates/assets/src/assets.rs` — `Docs` 의 `#[folder]` 를 `../../docs` → `../../GramEditor-docs/docs` 로 변경 (1줄)
  - `script/prettier` — `cd docs` → `cd GramEditor-docs/docs`
  - `.gitattributes` — `/docs/**` → `/GramEditor-docs/**`
- **현재 앱 내 도움말은 원본 GramEditor 문서를 그대로 보여준다** (AI 금지 문구 포함). Don Editor 자체 도움말을 추가하고 원본 문서임을 구분 표시하는 것은 이후 과제다 (R8에 따라 원본 도움말 기능은 유지).
- **링크 검사:** 이동 전후를 비교해 이동으로 새로 깨진 링크 15개를 확인했다. 모두 저장소 루트의 자료를 가리킨다 — `CONTRIBUTING.md` 의 `./crates/...` 14개, `SECURITY.md` 의 `./LICENSE` 1개. "내용 수정 없음" 원칙에 따라 그대로 둔다. (이동 전부터 깨져 있던 링크 3개는 별개)

---

## 3. 설계 방침

### 3.1 공통 원칙

- 신규 기능은 가능한 한 **별도 crate**로 분리하여 원본 코드 수정 범위를 최소화한다.
- 원본 코드를 수정하는 위치에는 `// [DonEditor]` 마커 주석을 남겨 변경 지점을 추적한다.
- AGENTS.md 규칙에 따라 **상수, 함수, 반복문에 한국어 주석**을 단다.
- 커밋 전 `cargo fmt`, `cargo clippy` 통과를 기본으로 한다.
- **AI 기능 확장을 전제로 설계한다.** Agent 패널·MCP 외의 AI 기능이 이후 추가될 수 있으므로, LLM 호출(`LlmProvider`)과 도구(`don_tools`)는 특정 기능에 묶이지 않는 공용 구조로 만든다.
- 문서에서 원본과의 차이(AI 적극 도입)를 항상 가장 먼저 밝힌다.

### 3.2 리브랜딩 (Gram → Don Editor)

- 앱 표시 이름, 창 제목, 메뉴("Quit Gram" 등), 아이콘, 실행 파일명 변경.
- `crates/paths` 의 데이터/설정 디렉터리를 `DonEditor`로 변경 → 원본 Gram과 설정·DB 충돌 방지.
- 단일 인스턴스 처리(`windows_only_instance.rs`)의 식별자 변경.
- crate 이름(`gram` 등) 자체의 대규모 리네이밍은 하지 않는다 (병합 비용 대비 이득 적음). 바이너리명·표시명만 변경.
- GPL-3.0 준수: `LICENSE`, `assets/licenses.md` 유지, README에 원저작(GramEditor/Zed) 고지 및 수정 사실 명시.

### 3.3 한국어 UI (R1)

- 신규 crate `crates/i18n`
  - 문자열 리소스: `assets/locales/ko.json`, `assets/locales/en.json` (키 → 문자열)
  - API: `i18n::t("menu.file.open")` 함수 및 `t!()` 매크로 (리소스는 바이너리에 임베드)
  - 키 누락 시 영어 → 키 이름 순으로 폴백
- 설정 항목 추가: `"ui_language": "auto" | "ko" | "en"` (기본 `auto`, `sys-locale`로 OS 언어 판단)
- 적용 범위 (단계적)
  1. 1차: 애플리케이션 메뉴, 타이틀바, 패널 제목/툴팁, 상태바, 주요 알림·확인 대화상자, 신규 기능(Agent 패널, MCP 설정) 전체
  2. 2차: 커맨드 팔레트 액션 표시명(액션 이름 → 한국어 표시명 매핑 테이블), 설정 UI(`settings_ui`), 키맵 편집기
  3. 3차: 온보딩, 기타 잔여 문자열
- 한글 표시/입력 검증
  - 기본 UI·버퍼 폰트에 한글 폴백(예: 맑은 고딕) 지정
  - Windows IME 한글 조합 입력(조합 중 문자 표시, 커서 위치) 동작 확인

### 3.4 문법 강조 (R2)

- 대상은 **26개 언어**(2.1 참고). 원본 내장 15개는 동작 확인과 확장자 보강만 하고, 신규 11개(java, c#, ruby, json5, properties, swift, scala, r, bat, powershell, dart)를 추가한다.
- 신규 언어 디렉터리: `crates/languages/src/{java,csharp,ruby,json5,properties,swift,scala,r,bat,powershell,dart}/`
  - `config.toml`: name, grammar, `path_suffixes`, 주석 기호, 괄호 쌍, 들여쓰기 단위
  - 쿼리: `highlights.scm`(필수), `brackets.scm`, `indents.scm`, `outline.scm`, (가능하면) `injections.scm`
  - 쿼리는 각 tree-sitter 문법 저장소의 기본 쿼리를 바탕으로 Gram 테마 캡처명(`@keyword`, `@function`, `@type` 등)에 맞게 조정. 출처 라이선스(MIT 등) 확인 후 고지
- **문법 crate 후보** (crates.io 조회 2026-10-27. `tree-sitter-language ^0.1` 에 의존하는 crate 는 워크스페이스의 tree-sitter 0.26 및 `register_native_grammars` 의 `LANGUAGE` 방식과 호환)

| 언어 | crate | 최신 버전 | 호환 | 비고 |
|------|-------|-----------|------|------|
| java | `tree-sitter-java` | 0.23.5 | O | |
| c# | `tree-sitter-c-sharp` | 0.23.5 | O | |
| ruby | `tree-sitter-ruby` | 0.23.1 | O | 워크스페이스 `Cargo.toml` 에 이미 선언됨 |
| json5 | `tree-sitter-json5` | 0.1.0 | **X** | `tree-sitter ~0.20.0` 의존 → 0.26 과 비호환. **→ `tree-sitter-json5-orchard` 0.1.0 으로 해결 (Phase 4 참고)** |
| properties | `tree-sitter-properties` | 0.3.0 | O | `java.util.Properties` 호환 확인: `#`·`!` 주석, `=`·`:`·공백 구분자, `\` 줄 이어쓰기, `\uXXXX` 이스케이프 |
| swift | `tree-sitter-swift` | 0.7.4 | O | 원본 문서는 alex-pinkus 문법 사용 |
| scala | `tree-sitter-scala` | 0.26.3 | O | |
| r | `tree-sitter-r` | 1.3.0 | O | |
| bat | `tree-sitter-batch` | 0.11.1 | O | Windows Batch (`.bat`, `.cmd`) |
| powershell | `tree-sitter-powershell` | 0.26.4 | O | |
| dart | `tree-sitter-dart` | 0.3.0 | O | |

  ※ 버전·호환성은 crates.io 메타데이터 기준이며, 실제 빌드·강조 동작은 Phase 4 에서 검증한다. 라이선스는 `script/check-licenses` 로 GPL-3.0 호환성 확인.
- 확장자 매핑은 2.1 의 표를 따른다 (신규 언어의 확장자는 구현 시 확정).
- `Cargo.toml`(워크스페이스) 및 `crates/languages/Cargo.toml`의 `load-grammars` feature에 문법 crate 추가, `register_native_grammars`에 등록
- tree-sitter `0.26` 호환성 확인 (비호환 시 git rev 고정 또는 대안 문법)
- **도입 순서(권장):** ① java, c#, ruby → ② json5, properties, bat, powershell (설정·스크립트류) → ③ swift, scala, r, dart. 한 묶음씩 추가할 때마다 빌드·강조를 확인한다.
- 범위: **문법 강조만** 제공. 신규 언어용 LSP 어댑터는 추가하지 않는다.
- 주의
  - 문법 crate 마다 C 파서를 컴파일하므로 **빌드 시간과 바이너리 크기가 늘어난다** (swift·scala·c# 등 파서가 큰 편). 언어 추가 전후로 크기를 측정해 기록한다.
  - 원본은 swift·dart·scala·r·powershell·java·c#·ruby 를 **확장 기능**으로 제공하므로, 내장 문법과 확장 설치 시의 우선순위·충돌 여부를 확인한다.
- 검증: 26개 언어 샘플 파일(`tests/samples/`)을 열어 언어 자동 감지·강조 확인 (라이트/다크 테마 모두)

### 3.5 공용 도구 레지스트리 + 로컬 MCP 서버 (R3)

"모든 기능 제어"를 위해 GPUI **액션(Action) 시스템**을 활용한다. Gram의 거의 모든 기능은 액션으로 등록되어 있으므로, 범용 액션 실행 도구와 고수준 도구를 함께 제공한다.

- 신규 crate `crates/don_tools` — **공용 도구 레지스트리**
  - 도구 정의(이름, 설명, JSON Schema 입력, 실행 함수)를 한곳에 모음
  - MCP 서버(R3)와 Agent 패널(R4)이 **같은 레지스트리를 공유** → 기능 중복 구현 방지
- 신규 crate `crates/mcp_server`
  - Rust 공식 MCP SDK(`rmcp`) 사용 검토, `gpui_tokio`와 연동
  - 전송 방식: 에디터 내장 **Streamable HTTP** (`127.0.0.1` 바인딩, 실행 시 무작위 토큰 발급)
  - 추가로 stdio 브릿지 실행 파일(`don-mcp.exe`) 제공 → stdio만 지원하는 MCP 클라이언트(Claude Desktop 등)에서 실행 중인 에디터로 중계
  - 설정: `"mcp_server": { "enabled": false, "port": 0 }` (기본 비활성, 메뉴/커맨드로 켜기)
- 도구 목록 (초안)

| 분류 | 도구 |
|------|------|
| 범용 | `list_actions`, `dispatch_action(name, args)` |
| 파일/워크스페이스 | `open_file`, `open_folder`, `save`, `save_all`, `close_tab`, `list_open_tabs`, `list_project_files` |
| 편집 | `get_buffer_text`, `set_buffer_text`, `insert_text`, `replace_range`, `get_selection`, `set_selection`, `undo`, `redo`, `format_buffer` |
| 검색 | `find_in_buffer`, `project_search`, `replace_in_project` |
| 탐색 | `go_to_line`, `get_diagnostics`, `get_outline` |
| UI | `toggle_panel`, `split_pane`, `set_theme`, `set_ui_language`, `set_buffer_language` |
| 설정 | `get_settings`, `update_settings` |
| 터미널 | `run_in_terminal` (사용자 승인 필요) |

- 리소스(MCP resources): 열린 버퍼 내용, 진단 목록, 현재 프로젝트 트리
- 보안: 로컬 루프백 전용, 토큰 인증, 파괴적 동작(파일 삭제, 프로젝트 전체 치환, 터미널 실행)은 에디터 내 승인 대화상자 표시(설정으로 조정 가능)

### 3.6 Agent 사이드 패널 (R4)

- 신규 crate `crates/agent_panel`
  - `workspace::Panel` 구현, **기본 위치 `DockPosition::Right`**, 좌/우 이동 가능
  - 토글 액션 `agent_panel::ToggleFocus` + 기본 단축키(기존 키맵과 충돌 확인 후 지정), 메뉴 "보기 > Agent 패널"
  - **패널 숨김 지원:** 닫기/숨기기 조작으로 패널을 감출 수 있게 한다.
  - **다시 표시하는 별도 아이콘:** 패널을 숨겨도 화면 안에 아이콘을 유지하고, 클릭하면 Agent 패널을 다시 표시한다. 기존 도크 버튼과의 통합 위치를 구현 시 정하고, 한국어 툴팁·표시 상태를 함께 점검한다 (2026-11-04 AGENTS.md 추가).
  - **시작 시 숨김 (2026-11-04 사용자 요청):** 프로그램 실행 시 기본적으로 숨긴 상태로 시작한다. 지난 실행에서 열어 둔 채 종료해도 다시 열지 않는다(`Panel::hidden_on_startup`).
- Provider 추상화 (`LlmProvider` 트레이트)
  - **Anthropic**: Messages API(`/v1/messages`), SSE 스트리밍, tool use
  - **OpenAI**: Responses API(또는 Chat Completions), SSE 스트리밍, function calling
  - 모델은 설정에서 선택/직접 입력 (Claude 기본값은 최신 모델 ID로 지정, 구현 시점에 확인)
  - **제공자 선택은 콤보박스(드롭다운)** 로 한다(2026-11-04 사용자 요청, 이전 토글 버튼 대체). 이후 Google, OpenCode, LM Studio를 추가할 예정이므로 목록은 `Provider::ALL`에서 만든다.
- API Key 관리 (2026-11-04 사용자 요청으로 변경)
  - **기본 설정(설정 > 에이전트)에서 제공자별로 사전 입력**한다. 저장소는 OS 자격 증명 저장소(`keyring` crate)이며 settings 파일에 평문 저장하지 않는다.
  - 제공자별 **API 서버 URL**(선택 사항)도 기본 설정에서 입력받는다. 공란이면 공식 기본 주소를 사용한다. 사용자 `settings.json`의 `agent.<제공자>.api_url`에 저장한다.
  - Agent 패널에는 키 입력란을 두지 않는다. 선택한 제공자의 키가 없을 때만 프롬프트 입력란 자리에 키 입력란을 보여 주고, 입력 즉시 기본 설정에 저장한 뒤 프롬프트 입력란을 다시 표시한다.
  - 환경 변수 `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` 폴백
- 기능
  - 채팅 UI (응답은 기존 `markdown` crate로 렌더링), 스트리밍 표시, 생성 중단 버튼
  - 컨텍스트: 매 요청마다 열린 폴더·현재 파일 경로·열린 탭 경로를 자동으로 포함(2026-11-04), 첨부 버튼으로 현재 파일 내용·선택 영역 추가
  - 도구 호출: `don_tools` 레지스트리 사용 → 편집은 diff 미리보기 후 사용자 승인 시 적용
  - 대화 기록 저장 (`db`/`sqlez` crate 활용), 새 대화/기록 목록
  - 오류(키 누락, 401, 429, 네트워크) 한국어 메시지 표시
- HTTP는 `http_client` + `reqwest_client` 재사용

### 3.7 플랫폼별 배포 스크립트 (R5, R9)

#### Windows zip (R5)

> **Windows zip 구현 완료 (2026-10-27).** 아래는 처음 계획이며, 실제 구현·옵션·검증 결과와 계획에서 달라진 점은 **Phase 3 절**을 기준으로 한다 (예: 옵션 `-CheckOnly`·`-DryRun`·`-SkipBuild`·`-IncludeSource`·`-BuildProfile` 추가, VS Build Tools 패키지 ID 변경, zip 최상위 폴더 구성). Linux/macOS tar.gz(R9)는 별도 후속 구현 대상이다.

- 파일: `script/package-zip.ps1` (본체) + `package-zip.bat` (실행 정책 우회 래퍼, 저장소 루트)
- 처리 순서
  1. **사전 검사**: 각 도구 존재 여부 확인
     - rustup / cargo (`rust-toolchain.toml` 채널 자동 설치 포함)
     - Visual Studio Build Tools — `vswhere.exe`로 MSVC x64 도구 + Spectre libs 확인
     - Windows SDK (≥ 10.0.20348)
     - CMake
     - Git (git 의존성 crate fetch용)
  2. **미설치 시 설치 여부 질의** (`예/아니오`) → 동의 시 `winget`으로 설치
     - 예: `Rustlang.Rustup`, `Kitware.CMake`, `Git.Git`, `Microsoft.VisualStudio.2022.BuildTools`(필요 워크로드/컴포넌트 `--override` 지정)
     - `winget`이 없으면 다운로드 URL 안내 후 중단
  3. VS 개발자 셸 환경 로드 (`Launch-VsDevShell.ps1`)
  4. `cargo build --release` (대상 `x86_64-pc-windows-msvc`, 필요 시 `aarch64`)
  5. 스테이징: 실행 파일, CLI, `don-mcp.exe`, `LICENSE`, `licenses.md`, 한국어 README
  6. `Compress-Archive` → `dist/DonEditor-<버전>-windows-<아키텍처>.zip` + SHA256 해시 파일
- 옵션: `-Arch`, `-OutDir`, `-SkipCheck`, `-Yes`(질의 자동 승인)
- 원본 `bundle-windows.ps1`(Inno Setup + 서명)은 참고만 하고 zip 전용으로 단순화

#### Linux/macOS tar.gz (R9, 2026-11-04 추가 · 미구현)

- 파일: 루트 `package-tar.sh` 예정 (별도 sh 배포 스크립트, Linux/macOS를 감지해 처리).
- 호출 시 해당 OS의 빌드 도구·프로젝트 Rust 툴체인을 검사하고, 없는 도구는 **설치 여부를 물은 뒤 동의한 항목만 설치**한다. 기존 `install-build-tools.sh`의 도구 목록·번호 선택 설치 흐름 재사용을 검토하되, 배포 진입점에서도 검사·설치 질의·설치 후 재검사를 수행해야 한다. 도구가 부족한 상태에서 거절·빈 입력·EOF를 받으면 빌드 전에 중단한다.
- OS별 빌드·실행에 필요한 파일을 스테이징하고, 실행 권한과 상대 경로를 보존해 `dist/DonEditor-<버전>-<linux|macos>-<아키텍처>.tar.gz` 및 SHA256 파일을 생성한다. Windows zip과 동일하게 GPL-3.0 `LICENSE`, GramEditor/Zed 원저작 고지, 제3자·D2Coding 라이선스, 소스 입수 방법을 포함한다.
- Linux와 macOS 각각에서 도구 준비/누락·설치 동의/거절·설치 실패·tar.gz 생성·압축 해제 후 실행을 검증한다. 현재 도구 설치 스크립트 테스트는 이 배포 검증을 대신하지 않는다.

### 3.8 기본 글꼴 D2Coding (R6)

- 글꼴 파일: `assets/fonts/D2Coding-Ver1.4.0-20261003-all.ttc` (TrueType Collection, 약 8.1MB — 여러 face 포함)
- 적용 방법 (**2026-10-27 구현 결과** — 처음 계획과 달라진 점은 ※ 표시)
  1. 글꼴 로드: `crates/assets/src/assets.rs` 의 `Assets::load_fonts` 가 `.ttf` 만 읽어 `.ttc` 가 무시되고 있었음 → `.ttf`, `.ttc` 를 모두 읽도록 수정(`EMBEDDED_FONT_EXTENSIONS`). 단위 테스트(`crates/gram/src/gram.rs` 의 `test_bundled_settings_and_themes`)에도 D2Coding 로드를 추가
  2. ※ **별칭 매핑(`.GramMono`→Myna, `.GramSans`→Fira Sans)은 변경하지 않았다.** 별칭을 바꾸면 테스트가 Myna 의 글자 폭을 전제로 하는 `gpui`·`editor` 의 기존 테스트가 깨질 수 있어(R8 위험), 별칭은 원본 그대로 두고 기본 설정값만 바꿨다. 별칭을 쓰는 곳(미니맵, SVG 렌더러의 모노스페이스 지정)은 계속 Myna 를 쓴다
  3. 기본 설정: `assets/settings/default.jsonc` 의 `buffer_font_family`, `ui_font_family` 를 모두 `"D2Coding"` 으로 변경. 설정 화면에는 `D2Coding`, `D2Coding ligature` 두 패밀리가 보일 것이다
  4. ※ **UI 글꼴도 D2Coding 으로 한 것은 해석에 따른 결정이다.** AGENTS.md 는 "기본 글꼴은 D2Coding"이라고만 적었다. 한글 UI 를 쓰므로 한글 글리프가 없는 Fira Sans 대신 D2Coding 으로 통일했다. 고정폭 글꼴이라 UI 가 다르게 보일 수 있으니, 원하지 않으면 `ui_font_family` 만 `.GramSans`(또는 `.SystemUIFont`)로 되돌리면 된다
- 확인된 사실 (2026-10-27)
  - `.ttc` 파일 구조를 직접 분석: **글꼴 4개** — `D2Coding` Regular/Bold, `D2Coding ligature` Regular/Bold
  - GPUI Windows(DirectWrite) 의 `add_fonts` 는 `AddFontFile` 로 등록하므로 컬렉션 파일의 글꼴을 모두 등록한다 → `.ttf` 로 분리할 필요 없음(코드 확인)
  - 앱이 정상 실행됨(빌드·실행 검증). **실제 화면에서 D2Coding 으로 보이는지, 한글 표시·굵게·줄 높이는 사용자 시각 확인 필요**
  - 라이선스: **글꼴 파일에 내장된 정보로 확인** — Copyright (c) 2015-2016 NAVER Corporation, Font designed by FONTRIX Inc., SIL Open Font License 1.1. `assets/fonts/D2Coding-LICENSE` 추가, `assets/licenses.md` 에 "Bundled fonts" 절 추가

### 3.9 라이선스 GPL-3.0 명기 (R7)

- `LICENSE`(GPL-3.0 전문)는 원본 그대로 유지한다.
- `assets/licenses.md` 상단 고지문(현재 "Copyright (C) 2026 The Gram Project")은 **원본 저작권 표기를 삭제하지 않고**, Don Editor 의 수정 사실과 저작권자 표기를 추가한다.
- README(한국어)에 다음을 명기: Don Editor 는 GramEditor 를 기반으로 한 GPL-3.0 파생물이며, GramEditor 는 다시 Zed 의 하드포크라는 점, 원본 저장소·저작권자, 수정 사실. → 루트 `README.md` 에 반영 완료, 원본 README 는 [GramEditor-docs/README.md](GramEditor-docs/README.md) 로 보존.
- 원본 소스 파일 상단의 라이선스·저작권 헤더는 제거하지 않는다.
- 앱 내 "정보(About)" / 라이선스 화면에도 동일 고지를 유지한다.
- 신규 의존성(문법 crate, MCP/HTTP/keyring 등) 도입 시 `script/check-licenses` 로 GPL-3.0 호환성을 확인하고 `assets/licenses.md` 에 반영한다.
- Windows zip과 Linux/macOS tar.gz에는 `LICENSE`, `licenses.md`, 제3자·글꼴 라이선스, 소스 입수 방법(저장소 위치 또는 소스 동봉)을 포함한다.

### 3.10 기존 기능 보존 (R8)

AGENTS.md 의 "그 외 기존에 존재하던 기능은 최대한 포함한다"에 따라, 새 기능을 얹는 과정에서 원본 GramEditor 의 기능이 사라지거나 동작하지 않게 되지 않도록 한다.

- **원칙:** 원본 기능을 삭제·비활성화하지 않는다. 변경은 "추가"와 "설정" 중심으로 하고, 원본 코드를 고칠 때는 `[DonEditor]` 표시를 남긴다. 기능을 빼야 할 사정이 생기면 사용자와 먼저 협의한다.
- **보존 대상 (예시):** 원본 [README 기능 목록](GramEditor-docs/README.md#features) 기준 — 원격 개발(SSH), 디버거(DAP), Jupyter/REPL, Git 통합, Vim/Helix 모드, Zed Wasm 확장(제한적 지원), LSP, 터미널, 작업(tasks), 스니펫, 테마·아이콘 테마, 마크다운 미리보기, 앱 내 도움말 등. 전체 목록은 원본 [docs](GramEditor-docs/docs/) 목차를 기준으로 한다.
- **방법**
  1. Phase 1 에서 **기능 인벤토리**(체크리스트)를 작성한다 → 작성 완료: [FEATURE-INVENTORY.md](FEATURE-INVENTORY.md). 각 기능마다 "원본 자체 동작 여부(Windows)"를 함께 기록해, 원본에서 원래 안 되던 것과 우리 변경으로 깨진 것을 구분한다. (원본 [Windows 개발 문서](GramEditor-docs/docs/development/windows.md)에도 Windows 동작이 보장되지 않는다고 적혀 있음. `ini`·`kdl` 문법 미등록도 여기서 확인)
  2. 각 Phase 종료 시 인벤토리 중 핵심 항목으로 스모크 테스트를 한다.
  3. Phase 8 회귀 테스트의 기준 목록으로 사용한다.
- **영향 받는 결정**
  - 앱 내 도움말(`gram://docs/...`)은 기능이므로 유지한다. 원본 문서를 삭제·교체하기보다 Don Editor 자체 도움말을 추가하고 원본 문서임을 구분 표시하는 방향으로 검토한다 (Phase 2 항목 참고).
  - 한국어 UI(Phase 5)·MCP(Phase 6)·Agent 패널(Phase 7)은 기존 액션·메뉴 구조를 바꾸지 않고 얹는다.

---

## 4. 목표 디렉터리 구조 (예정)

```
DonEditor/
├─ AGENTS.md / PROGRESS.md / LICENSE
├─ README.md                      # Don Editor 소개 (한국어)
├─ GramEditor-docs/               # 원본 GramEditor 문서 모음 (내용 수정 없음, 2.2 참고)
│  ├─ README.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, CHANGELOG.md
│  ├─ docs/                       # 원본 사용자 문서 (앱 내 도움말로 내장 중)
│  └─ assets/images/              # 원본 README 용 이미지
├─ package-zip.bat                # Windows zip 배포 래퍼 (R5, 구현됨)
├─ script/package-zip.ps1         # Windows zip 배포 본체 (R5, 구현됨)
├─ package-tar.sh                 # Linux/macOS tar.gz 배포 (R9, 예정·미구현)
├─ install-build-tools.bat        # Windows 빌드 도구 선택 설치 래퍼 (구현됨)
├─ script/install-build-tools.ps1 # Windows 빌드 도구 선택 설치 본체 (구현됨)
├─ install-build-tools.sh         # Linux/macOS 빌드 도구 선택 설치 (구현됨, tar.gz 생성 기능 없음)
├─ script/make_placeholder_icon.py  # 임시 아이콘·로고 생성 (Phase 2)
├─ assets/
│  └─ locales/ko.json, en.json    # UI 문자열 (R1)
├─ crates/
│  ├─ (GramEditor 원본 crate 들)
│  ├─ languages/src/java|csharp|ruby|json5|properties|swift|scala|r|bat|powershell|dart/   # 신규 언어 11개 (R2)
│  ├─ i18n/                       # 다국어 (R1)
│  ├─ don_tools/                  # 공용 도구 레지스트리 (R3, R4)
│  ├─ mcp_server/                 # 로컬 MCP 서버 + stdio 브릿지 (R3)
│  └─ agent_panel/                # Agent 사이드 패널 (R4)
└─ tests/samples/                 # 26개 언어 샘플 파일 (R2 검증)
```

---

## 5. 개발 절차 (단계별)

각 Phase 완료 시 이 문서의 체크박스와 "작업 기록"을 갱신한다.

### Phase 0. 계획 수립 ✅
- [x] AGENTS.md 요구사항 분석
- [x] GramEditor 구조 조사 (빌드, 언어, 패널, i18n, 경로, 라이선스)
- [x] 개발 계획 및 절차 작성 (본 문서)

### Phase 1. 소스 이관 및 기준선 빌드 ✅
- [x] GramEditor 소스를 DonEditor로 복사 (`target/`, `.git` 제외, `robocopy /E` 병합 복사 — 2513개 파일 40.7MB. 사용자가 추가한 D2Coding 글꼴·AGENTS.md·PROGRESS.md 는 보존)
- [x] `.gitignore` 정리 (원본에 `**/target` 이미 포함, `/dist/` 추가)
- [x] 빌드 도구 설치 확인 → rustup/cargo, CMake 4.4.4, MSVC 14.51(VS 2026), Windows SDK 10.0.26100 확인. Spectre 라이브러리만 미설치
- [x] **수정 없이** Windows 디버그/릴리스 빌드 성공 및 실행 확인 (최우선 리스크 해소) — **디버그·릴리스 빌드 모두 성공, 실행 확인 완료(2026-10-27).** (1차 디버그 빌드는 Spectre 완화 라이브러리 미설치로 실패(종료 코드 101)했으나 설치 후 성공 — 아래 항목 참고) ※ 컴파일 시작 전(의존성 내려받는 단계)에 원본 문서 이동에 따른 `assets.rs` 경로 1줄 변경이 반영되어 **완전한 무수정 빌드는 아님** (2.2 참고. 빌드에 영향을 주는 변경은 이 경로뿐)
- [x] 빌드 실패 시 원인·해결 방법을 이 문서에 기록 (2026-10-27, 아래 원인·조치·재빌드 결과)
  - **원인 (확인됨):** 약 537개 crate 컴파일까지 진행된 뒤 `msvc_spectre_libs v0.1.3` 의 build script 가 `No spectre-mitigated libs were found` 로 패닉. 다른 컴파일 오류는 없었음(error 1건). 의존 경로: `gram → languages → pet*`(Microsoft python-environment-tools, git 의존성)이 `msvc_spectre_libs` 의 `error` feature 를 켬. 이 crate 는 `cl.exe` 위치 기준 `lib\spectre\x64` 폴더를 찾으며, 사용 중인 VS Community 2026 (18.11, `C:\Program Files\Microsoft Visual Studio\18\Insiders`, MSVC 14.51) 에는 해당 폴더가 없음. 설치된 어떤 VS 인스턴스(2017/2019/2022 BuildTools/2026)에도 Spectre 컴포넌트가 없음
  - **조치 결과 (2026-10-27):** 사용자 승인 후 해결안 A 수행. VS Installer `setup.exe modify --installPath "C:\Program Files\Microsoft Visual Studio\18\Insiders" --add Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre --passive --norestart` 를 관리자 권한(UAC)으로 실행 → 종료 코드 0, 약 5분 30초 소요, `…\MSVC\14.51.36231\lib\spectre\x64` 생성 및 `vswhere -requires` 로 컴포넌트 인식 확인. 1차 시도는 **경로 공백을 따옴표 없이 넘겨(`Start-Process -ArgumentList` 배열) `installPath` 가 `C:\Program` 으로 잘려 종료 코드 1로 실패**했음(시스템 변경 없음). → Phase 3 배포 스크립트에서 설치 명령을 만들 때 경로·인자 따옴표 처리를 주의할 것
  - **재빌드 결과 (2026-10-27):** 설치 후 `cargo build` 재실행 → **성공** (오류 0, 경고 0, 7분 22초, 이어서 321개 crate 컴파일). 산출물 `target/debug/gram.exe` 약 200MB, `target/` 약 14GB. 앱 실행 확인 결과는 아래 항목 참고
  - **실행 확인 (2026-10-27):** `gram.exe --user-data-dir <격리 폴더> sample-open.txt` 로 실행 → 창 생성 2초, 응답 정상(Responding), 창 제목 `sample-open.txt — sample-open.txt` (파일 열림 확인), 메모리 약 154MB. 데이터·설정은 격리 폴더(`config`, `db`, `extensions` 등)에만 생성되었고, 사용자의 `%APPDATA%\Gram`·`%LOCALAPPDATA%\Gram` 은 실행 전 이후 수정되지 않음(수정 시각으로 확인). 확인 후 해당 PID 만 정상 종료. `--user-data-dir` 은 데이터와 설정(`<dir>/config`)을 모두 격리함
  - **GUI 수동 확인 — 확인 완료 ✅ (사용자 직접 확인, 2026-10-27):** 화면 렌더링, GUI 에서의 편집·저장, 한글 IME 입력. (자동 키 입력 전송은 사용자의 다른 창에 입력될 위험이 있어 수행하지 않고 사용자가 직접 확인함. 세부 확인 내용은 기록되지 않음.) 참고: 이 실행 방식(콘솔 없음)에서는 `logs/Gram.log` 가 생성되지 않았음
  - **릴리스 빌드 결과 (2026-10-27):** `cargo build --release` → **성공** (오류 0, 경고 0, 20분 36초, 846개 crate 컴파일). 산출물 `target/release/gram.exe` 약 151MB(+ `gram.pdb` 약 65MB), `target/release` 약 5.1GB, `target/` 합계 약 14GB. 격리 데이터 폴더로 실행해 창 생성 2초·응답 정상·파일 열림·메모리 약 139MB 확인 후 해당 프로세스만 정상 종료. (GUI 편집·저장·한글 입력의 사용자 확인은 디버그 빌드 기준)
  - **해결안 A (권장, 수행함):** Visual Studio Installer 에서 VS 2026 에 `MSVC 최신 버전 - C++ x64/x86 Spectre 완화 라이브러리` (컴포넌트 ID `Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre`) 추가 설치. 원본 문서(`windows.md`)가 요구하는 정식 요건이며, Phase 3 배포 스크립트의 사전 검사·설치 항목에도 이미 포함됨. 설치는 시스템 변경(관리자 권한 필요)이므로 사용자 확인 후 진행
  - **해결안 B (비권장):** `[patch.crates-io]` 로 `msvc_spectre_libs` 를 로컬 스텁으로 대체하여 우회 — 원본 의존성 구성이 달라지고 Spectre 완화 링크가 빠지므로 미설치 환경 대응이 필요할 때만 검토
  - 로그: 세션 임시 폴더의 `build-debug.log` (영구 보관 안 됨). 컴파일된 결과물은 `target/` 에 남아 있어 재빌드 시 이어서 진행됨(약 4.8GB)
- [x] **원본 기능 인벤토리 작성 (R8, 3.10 참고)** — [FEATURE-INVENTORY.md](FEATURE-INVENTORY.md) (2026-10-27). 기능 50개 영역(빌드 포함 여부는 `cargo tree` 로 계산, 원본 문서 링크 포함), Windows 메뉴 항목 97개, 의존 트리 밖 crate 8개, 스모크 테스트 체크리스트 16개. **실행 확인은 2개 영역(편집 기본, 한글 입력)만 완료**이고 나머지는 "아직 모름(⬜)" 상태 — 확인되는 대로 갱신
- [x] 원본 기준선 (이후 변경 diff 추적용) — 사용자가 작업 상태를 이미 `master` 에 커밋(`99a0ebe`, `cf001ce`, `ce1d36b`)했으므로 그 이력은 건드리지 않고, **순수 GramEditor 3.4.0 만 담은 부모 없는 커밋 `961c4dc` 에 태그 `gram-baseline-3.4.0`** 을 달았다. 원본 대비 변경은 `git diff gram-baseline-3.4.0 master` 로 확인 (수정 5·추가 4·이름 변경 138(전부 내용 100% 동일)·삭제 0)
- 완료 기준: 원본 상태 그대로 Windows에서 실행되어 파일 열기/편집/저장 가능 — **디버그 빌드 기준 충족 (2026-10-27, 사용자 확인)**. 기능 인벤토리·기준선 태그·릴리스 빌드 완료 → **Phase 1 완료**

### Phase 2. 리브랜딩
- [x] **표시 이름·메뉴 문구·시작 화면 제목·아이콘 변경** (2026-10-27, 코드·빌드 완료. 메뉴·아이콘의 실제 모습은 사용자 시각 확인 대기)
  - 제품 이름: `release_channel::display_name()` → `Don Editor`(dev 채널은 `Don Editor Dev`). 메뉴(`app_menus.rs`: 메뉴 이름, About, Hide, Quit), 정보 대화상자 제목(`gram.rs`), 시작 화면 제목(`onboarding.rs`), 실행 실패 메시지(`main.rs`), CLI 도움말·버전 문구(`crates/cli`)
  - 실행 파일 속성(`crates/gram/build.rs`): 파일 설명·제품 이름 `Don Editor` — `Get-Item ... VersionInfo` 로 확인함
  - **아이콘·로고는 임시(placeholder)다.** 파란 둥근 사각형 위 흰색 "D"(stable), 주황색(dev). `script/make_placeholder_icon.py` 로 생성(Pillow 필요) — `crates/gram/resources/windows/app-icon.ico`, `app-icon-dev.ico`, `assets/images/logo_dark.svg`, `logo_light.svg`. 실제 디자인이 생기면 이 파일들을 교체하면 된다. macOS·Linux 용 PNG 아이콘과 `gram.iss`(Inno Setup) 등 Windows 이외·설치 프로그램 자료는 원본 그대로 둠
  - 변경하지 않은 것: 메뉴 "Gram Repository"(원본 저장소 URL 을 여는 항목 — Don Editor 저장소 주소가 정해지면 변경), `gram://` URL 스킴(앱 내 도움말에서 사용), HTTP User-Agent, 디버거 프로토콜의 클라이언트 이름
- [x] **데이터/설정 경로 `DonEditor` 로 변경, 단일 인스턴스 식별자 변경** (2026-10-27, **동시 실행 검증 완료**)
  - (변경 전) `crates/release_channel` 의 `app_identifier()` 는 채널(`crates/gram/RELEASE_CHANNEL` = `stable`) 기준 `Gram-Editor-Stable` 이며, Windows 단일 인스턴스 뮤텍스(`<식별자>-Instance-Mutex`)와 명명 파이프에 쓰인다(`windows_only_instance.rs`). 설치된 원본 Gram(stable)과 식별자가 같아 동시에 실행할 수 없었다 → **`DonEditor-Stable` / `DonEditor-Dev` 로 변경**
  - 데이터 경로(`crates/paths`): 설정 `%APPDATA%\DonEditor`, 데이터·캐시 `%LOCALAPPDATA%\DonEditor`, 로그 `DonEditor.log`(macOS·Linux 경로도 `DonEditor`/`doneditor` 로 통일)
  - **의도적으로 바꾸지 않은 것:** 프로젝트 폴더 안의 `.gram/`(`settings.jsonc`, `tasks.jsonc`, `debug.jsonc`)와 원격 서버 디렉터리 `.gram_server`, `.gram_wsl_server`. 같은 프로젝트를 원본 Gram 과 함께 열 때 프로젝트별 설정을 공유하는 편이 자연스럽고 R8(기존 기능 보존)에도 맞다. 분리가 필요하면 `crates/paths` 의 `.gram` 상수만 바꾸면 되므로 사용자 결정 사항으로 남김
  - **검증 (2026-10-27):** 이전 빌드(`gram.exe`, `Gram-Editor-Stable`)와 새 빌드(`doneditor.exe`)를 **동시에 실행** → 서로 다른 PID·창, 둘 다 응답 정상, 뮤텍스 `DonEditor-Stable-Instance-Mutex` 와 `Gram-Editor-Stable-Instance-Mutex` 가 각각 존재. 새 빌드는 `%APPDATA%\DonEditor`(`themes`)와 `%LOCALAPPDATA%\DonEditor`(`db`, `extensions`, `languages`, `logs` 등)를 만들었고, **원본 `%APPDATA%\Gram`·`%LOCALAPPDATA%\Gram` 의 수정 시각은 실행 전후 동일**. 확인 후 제가 띄운 두 프로세스만 정상 종료. (이 검증으로 사용자 PC 에 `DonEditor` 폴더가 생성되었음)
- [x] README 정리 (2026-10-27): 원본 `README.md` 는 [GramEditor-docs/README.md](GramEditor-docs/README.md) 로 보존(내용 수정 없음), 루트 `README.md` 에 Don Editor 소개(한국어, 원저작 고지·GPL-3.0 명기) 작성. 기능이 구현될 때마다 "목표 기능" 표의 상태 갱신 필요
- [x] 원본 문서를 [GramEditor-docs/](GramEditor-docs/) 로 분리 보관하고 우리 문서(README.md, PROGRESS.md)의 원본 설명 단락에 링크 연결 (2026-10-27, 2.2 참고)
- [x] README 에 "AI 적극 도입이 원본과의 가장 큰 차이"임을 반영 (2026-10-27): 소개 문단, 기능 표(AI 기능 최상단), "원본과 달라지는 점", 문서 안내
- [x] 원본의 **AI 금지·인간 기여 정책 문서**(`CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `docs/index.md`, `docs/mission.md`, `docs/HUMAN-CONTRIBUTION-POLICY-2-UP.md`, `docs/migrate/*`)는 Don Editor 방향(AI 적극 도입)과 충돌하므로, 수정하지 않고 [GramEditor-docs/](GramEditor-docs/) 로 분리하여 "원본 프로젝트의 정책"임을 README 에 명시 (2026-10-27)
- [x] **앱 내 도움말 처리:** — **구현 완료(`5fa69a6`, 2026-11-04).** 원본 문서와 도움말 기능은 유지하고, 한국어 자체 도움말 `docs/`(index·installation·agent·mcp·security·packaging)를 추가했으며 원본 문서를 열 때 출처 안내를 표시하고 검색에도 통합했다. 2026-11-04 Agent 설정 변경에 맞춰 `agent.md`·`security.md`·`installation.md`를 갱신했다. 이하 원래 계획: 현재 앱 내 도움말(`gram://docs/...`)이 원본 문서(AI 금지 문구 포함)를 그대로 표시한다. R8(기존 기능 보존)에 따라 도움말 기능 자체와 원본 문서는 **유지**하고, Don Editor 자체 도움말(한국어)을 추가하며 원본 문서임을 구분 표시하는 방향으로 검토한다. 구체 방식은 사용자와 협의 (Phase 5 한국어 UI 와 함께 검토)
- [ ] Don Editor 자체 기여 안내·행동 강령이 필요한지 사용자와 협의 (필요 없으면 생략. 프로젝트 정책 결정이므로 임의로 작성하지 않는다) — **사용자 결정 대기.** 2026-11-04까지 새로 제정하지 않았다
- [x] 앱 내부에 "AI 없음"을 알리는 문구가 있는지 점검 (2026-10-27) — Rust·JSONC·JSON·TOML 의 문자열 리터럴과 기본 설정을 검색한 결과 **해당 문구 없음**(검색에 걸린 것은 테스트용 샘플 문자열 1건). 다만 앱 내 도움말이 표시하는 원본 문서(`GramEditor-docs/docs`)에는 있음(위 "앱 내 도움말 처리" 항목)
- [x] **실행 파일명 변경** (2026-10-27): `crates/gram/Cargo.toml` 의 `[[bin]]` 이름 `gram` → **`doneditor`** (`doneditor.exe`). 대문자 `DonEditor` 는 rustc 의 crate 이름 규칙(snake_case) 경고를 일으켜 소문자로 함. 라이브러리 crate 이름은 `gram` 그대로(대규모 리네이밍 지양 원칙)
  - **`autobins = false` 추가가 필수였다:** `[[bin]]` 이름을 바꾸자 Cargo 가 `src/main.rs`(라이브러리 루트)를 `gram` 바이너리로 자동 발견해 경고가 났다(원본은 [[bin]] 이름이 같아 우연히 가려져 있었음). `autobins = false` 로 해결, 경고 0건 확인
  - 함께 수정: `crates/cli/src/main.rs`(명령 이름 `doneditor`, 앱 탐색 위치 `../DonEditor.exe`·`../lib/doneditor/doneditor.exe`·`./doneditor.exe`), `crates/util/src/util.rs`(앱이 CLI 를 찾는 위치 `bin/doneditor.exe`·`./cli.exe`). **배포 zip 폴더 구성(Phase 3) 가정:** 루트에 `doneditor.exe`(GUI), `bin/doneditor.exe`(CLI)
  - 손대지 않은 것(Windows 에서 쓰이지 않거나 설치 프로그램용): `crates/explorer_command_injector`(`Gram.exe` 참조, 탐색기 컨텍스트 메뉴용 별도 crate), `gram.sh`(WSL 용), Linux `deb`/`rpm` 패키징 메타데이터, `script/bundle-*`. 이전 빌드의 `target/debug/gram.exe` 는 잔재(삭제 가능)
- [x] **기본 글꼴 D2Coding 적용** (2026-10-27, 3.8 참고) — 코드 적용·빌드·실행 완료. `.ttc` 로드 경로 수정, 기본 설정 변경, 라이선스 고지 추가. **화면에서 실제로 D2Coding 으로 보이는지는 사용자 시각 확인 대기**
- [x] **라이선스 고지 정리** (2026-10-27, 3.9 참고) — `assets/licenses.md`: Don Editor 가 Gram(→Zed)의 수정본임을 밝히되 **원본 "Copyright (C) 2026 The Gram Project" 표기와 GPL 본문은 그대로 유지**, "Bundled fonts" 절 추가, `assets/fonts/D2Coding-LICENSE` 추가. README 는 앞서 반영 완료
  - **남은 TODO:** `licenses.md` 의 "full source code … from codeberg.org/GramEditor/gram" 문구는 원본 Gram 을 가리킨다. 이를 "Don Editor 의 소스는 배포자에게서 받을 수 있다"로 바꿔 두었으나, **배포 시 Don Editor 소스를 어디에 공개할지(저장소 주소 또는 소스 동봉)** 를 정해 구체적으로 적어야 한다(GPL 의무). 수정본 저작권자 표기도 필요하면 사용자가 결정
- [ ] **'Gram' 문자열 잔여 백로그 (진행형 백로그, 의도적으로 남긴 항목 포함):** Phase 5 에서 화면에 보이는 것 중 설정 화면 설명문 8건·에디터 안내문·시작 화면·시스템 정보 등을 "Don Editor"로 바꿨다. 남은 것은 식별자·호환성 때문에 의도적으로 두었거나(테마 이름 `Gram (Default)`, `gram://` 스킴, 사용자 에이전트, 키맵 컨텍스트 `Gram Keybind Context`, DAP 클라이언트 이름) Linux 전용(`install_cli` 안내), 내부 오류 메시지·개발용 문구다. 재조사 명령: `grep -rIn '"[^"]*\bGram\b[^"]*"' crates --include=*.rs`
- **완료 기준 점검:** ① 원본 Gram 과 동시 실행해도 설정이 섞이지 않음 — **충족(검증함)** ② 기본 글꼴이 D2Coding 으로 표시됨 — **사용자 시각 확인 필요**. 나머지 협의 항목(앱 내 도움말 처리, 자체 기여 안내 필요 여부)은 Phase 2 를 막지 않는 별도 결정 사항

### Phase 3. Windows 배포 zip 스크립트 (R5, 조기 작성 → 이후 Phase마다 패키징 검증) ✅
- [x] **`script/package-zip.ps1` 사전 검사 + 설치 질의 + winget 설치** (2026-10-27) — 점검 6항목: Git, rustup, Rust 툴체인(`rust-toolchain.toml` 의 채널·targets), MSVC C++ 도구 + Spectre 라이브러리, Windows SDK ≥ 10.0.20348, CMake. 없는 항목마다 `[y/N]` 로 묻고(기본 "아니오", 입력이 닫혀 있어도 "아니오"), 동의하면 `winget`(VS 는 기존 VS 에 `setup.exe modify`, 없으면 `Microsoft.VisualStudio.BuildTools`)으로 설치. 거절하면 직접 설치 주소를 안내하고 종료 코드 1
- [x] **빌드 → 스테이징 → zip + SHA256** — `cargo build --package gram --package cli [--release]` 후 `dist\DonEditor-<버전>-windows-<아키텍처>.zip` + `.sha256`(sha256sum 호환 형식). 끝나면 `dist\staging` 삭제
- [x] **`package-zip.bat` 래퍼** — `-ExecutionPolicy Bypass` 로 ps1 호출, 인자 전달, 종료 코드 전파. **파일을 ASCII 로만 작성**(아래 "발견한 문제" 참고)
- [x] **`cli` 포함 결정:** 기본 `cargo build` 는 `gram` 만 만들므로 스크립트가 `--package cli` 를 함께 빌드해 zip 의 `bin\doneditor.exe` 로 넣는다 (Phase 2 에서 정한 배치)
- [ ] **깨끗한 환경(도구 미설치 VM 또는 사용자 계정)에서 검사·설치 흐름 확인 — 미수행.** 아래 "검증 범위" 참고
- 완료 기준: 스크립트 1회 실행으로 `dist/` 에 실행 가능한 zip 생성, 압축 해제 후 실행 확인 — **충족 (릴리스 빌드 기준)**

**zip 구성** (최상위 폴더 `DonEditor-<버전>-windows-<아키텍처>\` 안): `doneditor.exe`(GUI), `bin\doneditor.exe`(CLI), `bin\don-mcp.exe`(MCP stdio 브릿지, `f1ba957`에서 추가), `LICENSE`, `licenses.md`, `licenses\`(APACHE·MIT·OFL·CC-A-SA-4.0·D2Coding-LICENSE), `README.md`, `SOURCE.txt`, `source\DonEditor-<버전>-source.zip`(작업 트리 스냅샷, 항상 동봉). Phase 2 메모의 "zip 루트에 doneditor.exe" 는 이 최상위 폴더 안을 뜻하도록 정리했다 (압축을 풀 때 파일이 흩어지지 않게). `SOURCE.txt` 는 제품·버전·커밋·미커밋 변경 여부·소스 저장소 주소(`git remote origin`, 없으면 "배포자에게 요청")·GPL 안내를 담는다. 옵션 `-IncludeSource` 는 기본 동봉 소스 zip을 결과물 폴더에도 별도로 복사한다(커밋된 내용만이 아니라 미커밋 수정·새 파일도 포함, gitignore·`target`·`dist` 제외)

**검증 결과 (2026-10-27)**
- **릴리스:** 한 번의 실행으로 **9분 10초**(이전 릴리스 빌드의 캐시 일부 재사용), zip **53.4MB**, 종료 코드 0. 압축 해제 후 `bin\doneditor.exe --version` 이 같은 폴더의 GUI 를 찾아 출력, GUI 를 격리 데이터 폴더(`--user-data-dir`)로 실행 → 응답 정상·`sample-open.txt` 열림·메모리 약 144MB, 해시 파일 일치, 항목 이름 전부 `/`, exe 아이콘 파랑(stable), 확인 후 제가 띄운 PID 만 종료
- **디버그 프로파일(`-BuildProfile Debug`, 스크립트 점검용):** 동일하게 통과 (2분 29초 + 재패키징 11초)
- **점검·설치 흐름(시스템 변경 없이):** `-CheckOnly` 전 항목 OK 출력. PATH 에서 git·cargo 를 숨겨 "미설치"를 흉내 낸 뒤 ① `-DryRun -Yes` → winget·rustup 명령이 올바르게 출력, ② `n` 입력 → 거절 처리·종료 코드 1·직접 설치 안내, ③ 입력이 닫힌 경우 → "아니오" 처리. Spectre 컴포넌트 ID 를 가짜로 바꾼 사본으로 `-DryRun` → `setup.exe modify --installPath "C:\Program Files\Microsoft Visual Studio\18\Insiders" --add … --passive --norestart` 로 **경로 따옴표가 올바르게** 만들어짐(Phase 1 에서 겪은 실패 원인)

**검증하지 못한 것 (솔직한 한계)**
- **실제 설치:** winget 설치, 관리자 권한(UAC)을 거치는 `setup.exe modify`, 설치 직후 PATH 갱신(`Update-SessionPath`)은 이 PC 에 이미 도구가 있어 **실행해 보지 않았다**(출력만 확인). 사용자 PC 를 바꾸는 작업이라 임의로 하지 않았다
- `-Arch aarch64`(x64 PC 에서 교차 빌드) 미검증. 더블클릭 실행 시 `pause` 동작 미확인(실제 더블클릭은 하지 않음). 이 PC 에 LLVM(clang) 이 설치되어 있는지 확인하지 않았으므로, 필요 여부는 깨끗한 환경에서만 알 수 있다
- 빌드 로그에 **링커 경고 LNK4217 1건**(wasmtime 심볼)이 나왔다. Phase 1 릴리스 빌드는 "경고 0"이었다. 빌드는 성공했고 동작에 영향은 없어 보이나 원인(스크립트가 불러온 VS 개발자 환경 때문인지 등)은 조사하지 않았다

**발견한 문제와 조치 (구현 중)**
- `vswhere -latest` 는 이 PC 에서 C++ 도구가 없는 VS 2017 Express 를 고를 수 있다(원본 `bundle-windows.ps1` 이 쓰는 방식) → 필요한 컴포넌트를 `-requires` 로 지정해 인스턴스를 고른다
- 빌드 시 `RELEASE_CHANNEL` 환경 변수가 없으면 dev(주황) 아이콘이 들어간다 → 스크립트가 지정 (위 "사용자 결정·확인 요청" 참고)
- Windows PowerShell 5.1 의 `ZipFile.CreateFromDirectory` 는 zip 항목 이름에 `\` 를 쓴다(다른 OS 의 압축 해제 도구와 비호환) → 항목 이름을 `/` 로 직접 지정해 한 파일씩 압축
- 한 줄짜리 배열 반환이 문자열로 풀려 `$out[0]` 이 첫 글자가 되는 문제, 점검 블록이 지역 변수를 못 보는 문제, 설치 명령 출력이 반환값을 오염시키는 문제 → 각각 수정
- `.bat`: `chcp 65001` 이후의 UTF-8 한글 줄을 cmd 가 잘못 읽어 엉뚱한 명령 오류가 났다 → `.bat` 은 ASCII 로만 쓰고 한글 메시지는 ps1 이 출력(실제 콘솔에서는 PowerShell 이 유니코드로 직접 쓰므로 `chcp` 불필요). PATH 에 Git 의 유닉스 `find` 가 앞서는 환경을 위해 `System32\find.exe` 를 명시. `.gitattributes` 에 `*.bat crlf` 추가(`*` 규칙이 LF 로 정규화하면 cmd 가 오동작할 수 있음)
- `.ps1` 은 **UTF-8 BOM + CRLF** 로 저장해야 Windows PowerShell 5.1 이 한글을 올바르게 읽는다(BOM 없으면 깨짐). 이 파일을 편집한 뒤에는 BOM 이 유지되는지 확인할 것
- PROGRESS 3.7 계획과 달라진 점: VS Build Tools 패키지 ID 를 `Microsoft.VisualStudio.BuildTools`(VS 2026, winget 조회로 확인)로 했다(계획은 `2022.BuildTools`). 이 PC 의 호스트 아키텍처와 같으면 `--target` 을 주지 않아 기존 `target\release` 캐시를 재사용한다(원본 스크립트는 항상 `--target` → 전체 재빌드)

### Phase 3 후속. Linux/macOS 배포 tar.gz 스크립트 (R9, 스크립트 구현·모의 검증 완료 — 실제 OS 빌드·실행 미검증)

- [x] 별도 sh 배포 스크립트(`package-tar.sh`) 작성, Linux/macOS 감지·OS별 빌드 도구 검사 — 구현됨(`f1ba957`). Linux(`apt/dnf/pacman/zypper`)·macOS(`brew`)·x64/ARM64 호스트 감지, 교차 빌드 없음(네이티브 타깃만)
- [x] 미설치 도구의 설치 여부 질의, 선택한 도구 설치 후 재검사, 거절·EOF·설치 실패 시 빌드 전 중단 — 구현됨. `install-build-tools.sh` 번호 선택 메뉴(`1 3`/`A`/`N`/`I`/`0`/EOF) 재사용, 설치 후 `--check-only` 재검사, 미준비 시 종료 코드 1(macOS 외부 설치 미완료는 2)
- [x] OS별 릴리스 빌드·스테이징·tar.gz/SHA256 생성, GPL-3.0·원저작·제3자·글꼴 고지 및 소스 입수 방법 포함 — 구현됨. `gram`·`cli`·`mcp_server`·`remote_server` 함께 빌드, Linux(`bin/doneditor`·`libexec/`·`.desktop`)·macOS(`Don Editor.app`·`Info.plist`) 배치, `LICENSE`·`licenses.md`·`licenses/`·`D2Coding-LICENSE`·작업 트리 소스 스냅샷·`SOURCE.txt` 동봉
- [ ] Linux와 macOS 각각에서 도구 준비/누락 및 설치 동의/거절 흐름 검증, 압축 해제 후 실행 확인 — **모의 검증만 완료, 실제 OS 검증 미수행.** `tests/test_package_tar.py` 14개(가짜 cargo·패키지 관리자, 실제 tar/git/SHA256) 통과. 실제 Linux/macOS 네이티브 릴리스 빌드·압축 해제·GUI 실행, 깨끗한 환경 설치, ARM64, macOS 서명/공증은 미검증
- 완료 기준: 각 OS에서 스크립트 호출 시 도구 검사·설치 질의를 거쳐 tar.gz를 만들고, 압축을 푼 배포본이 실행됨. **스크립트·모의 검증은 충족, 실제 OS 실행 확인은 남음.** `install-build-tools.sh` 테스트 19개 통과는 배포 압축 검증을 대신하지 않는다.

**Linux 검증 결과 (2026-11-04, Debian x86_64, `target/` 없음, 메모리 3.8GiB)**
- `tests/test_package_tar.py` **14개 통과**, `tests/test_install_build_tools.py` **19개 통과**(가짜 cargo·패키지 관리자, 실제 tar/git/SHA256)
- `bash -n` 구문 검사 통과(`package-tar.sh`·`install-build-tools.sh`), `shellcheck`는 미설치로 생략. `package-zip.bat` ASCII·CRLF, `script/package-zip.ps1` UTF-8 BOM·CRLF 유지 확인(Linux에서 실행 검증은 불가)
- `bash package-tar.sh --check-only` → 미준비 항목(C/C++·Clang·GUI 개발 라이브러리) 감지, 종료 코드 1. Rust 1.99.1은 `rustc --version` 조회 시 툴체인 동기화 후 준비됨으로 바뀜(재검사 동작 확인)
- `printf '' | bash package-tar.sh`(EOF)·`printf '0\n' | ...` → 설치 메뉴에서 취소 처리, `필수 도구가 준비되지 않아 빌드 전에 중단합니다`, 종료 코드 1, `dist/` 미생성. R9의 "미설치 시 설치를 묻고 거절하면 빌드 전 중단" 요구 충족
- **실제 릴리스 빌드·tar.gz 생성·압축 해제·실행은 수행하지 않았다.** C/C++·GUI 개발 라이브러리 미설치 상태이며, 시스템 패키지 설치(`sudo apt-get`)와 전체 컴파일(캐시 없음·저메모리)은 이번 작업에서 하지 않았다. macOS·ARM64·깨끗한 환경 설치·서명/공증도 미검증

### Phase 4. 문법 강조 (26개 언어: 원본 내장 15개 확인 + 신규 11개 추가) ✅ (2026-10-27)
- [x] **원본 내장 15개 언어 동작 확인 및 확장자 보강** — 샘플 18개(`mjs`·`cjs`·`yaml`·`xhtml` 포함)를 실제 쿼리·테마 키 규칙으로 강조해 대표 토큰을 검증. 요구 확장자 중 빠진 것은 Python `pyw` 뿐이라 추가, XML 은 `xsd xsl xslt wsdl` 을 보강. mjs·cjs(JavaScript)·yml/yaml·xhtml(HTML) 은 원래 포함되어 있었다
- [x] **신규 11개 추가** (java, c#, ruby, json5, properties, swift, scala, r, bat, powershell, dart) — 묶음별 단계 도입 대신 한 번에 넣었다. 빌드가 오래 걸려서, 먼저 스크래치 crate(tree-sitter 0.26.13 + 문법 crate 11개)로 문법 로드·파싱·쿼리 컴파일을 빠르게 확인한 뒤 본 저장소에 반영했다
- [x] **json5 대안 결정:** `tree-sitter-json5` 0.1.0(tree-sitter ~0.20 의존, 비호환) 대신 **`tree-sitter-json5-orchard` 0.1.0** (MIT, `tree-sitter-language ^0.1` 의존 → 0.26 호환)을 사용. 샘플의 주석·따옴표 없는 키·작은따옴표·후행 쉼표·16진수·`Infinity`·줄 이음을 **오류 없이 파싱**한다. 한계: 2026-09-08 에 공개된 인지도 낮은 fork(다운로드 약 2천 회)라 유지보수가 끊길 수 있다 → 문제가 생기면 `jsonc` 문법 재사용이나 문법 소스 포함을 검토
- [x] **빌드 크기 측정** (릴리스, `package-zip.bat`): `doneditor.exe` **151,408,128 → 170,434,048 바이트 (+19,025,920, 약 +12.6%)**, zip 53.4 → 55.1MB. 문법별 크기는 따로 재지 않았다. 빌드 시간은 19분 40초였으나 이전 9분 1초와는 캐시 재사용 조건이 달라 **직접 비교할 수 없다**
- [x] **내장 문법과 확장 기능 충돌 (코드로 확인):** `LanguageRegistry::register_language` 는 **같은 이름이면 나중 등록이 내장 언어의 문법·확장자·로더를 덮어쓴다**(확장은 내장 뒤에 등록). 이름이 달라도 같은 확장자이면 일치 길이가 같을 때 나중에 등록된 쪽이 이기고, 사용자 `file_types` 설정이 항상 최우선이다. 즉 사용자가 Swift·Dart 등의 확장을 직접 설치하면 그 확장이 우선하며 내장 지원을 깨뜨리지는 않는다
- [x] **라이선스:** 11개 crate 모두 `Cargo.toml` 메타데이터상 MIT(GPL-3.0 과 호환). 각 언어 디렉터리에 `LICENSE`(crate 에 파일이 있으면 원문, 없는 java·ruby·properties 는 저장소 관례(`sql/LICENSE`)대로 `Cargo.toml` authors 만 근거로 한 고지)를 두고 `assets/licenses.md` 에 "Syntax highlighting grammars added by Don Editor" 절을 추가. **정정:** `script/check-licenses` 는 워크스페이스 crate 의 `LICENSE-GPL` 심볼릭 링크만 검사하며 제3자 의존성의 호환성 검사가 아니다(앞서 계획에 적은 용도는 잘못이었음)
- [x] **샘플과 자동 검증:** `tests/samples/` 에 샘플 29개 파일 작성. 테스트 4개 추가 — `crates/languages/src/language_support_tests.rs`(① 모든 언어 설정 로드 ② 샘플 파싱 오류 0 + 대표 토큰 225개(신규 11개 언어 92개 + 원본 내장 133개)가 기대한 테마 키로 강조되는지, 실제 편집기와 같은 `BufferChunks` 경로 사용 ③ 레지스트리에 모든 설정을 올린 상태에서 확장자 감지·충돌 검사, `Gemfile`·`.cmd`·`.psm1`·`.pyw`·`.csx`·`.sbt`·`.R`·`.xsd` 등 추가 확장자 포함), `crates/gram/src/gram.rs` 의 `test_don_editor_required_languages_are_registered`(실제 `languages::init` 로 29개 파일 이름이 기대 언어로 감지되고 문법까지 로드되는지). 결과: **languages 61개(기존 58 + 신규 3), gram 34개(기존 33 + 신규 1) 모두 통과** — 기존 테스트 회귀 없음. 강조 결과를 사람이 보려면 `DON_DUMP_HIGHLIGHTS=1` 로 실행(테스트 파일 머리말 참고)
- [x] **릴리스 zip 스모크 테스트:** 압축 해제 후 GUI 로 신규 언어 샘플 11개를 한 번에 열어 탭 11개·응답 정상·메모리 약 165MB 확인, 로그에 문법·쿼리 관련 오류 없음
- 완료 기준: 26개 언어 파일을 열면 자동으로 언어가 감지되고 키워드·문자열·주석·타입 등이 강조됨 — **API·테마 키 수준에서 충족** (실제 창의 색상을 눈으로 본 확인은 사용자 몫, 아래 참고)

**신규 언어 구성** — 디렉터리 `crates/languages/src/{java,csharp,ruby,json5,properties,swift,scala,r,bat,powershell,dart}/` 마다 `config.toml`, `highlights.scm`, `LICENSE`, `overrides.scm`(주석·문자열 범위: 주석·문자열 안에서 따옴표가 자동으로 닫히지 않게 함), `brackets.scm`(bat·properties 제외). 언어 서버(LSP) 어댑터는 없고 **문법 강조만** 제공한다. 등록은 `lib.rs` 의 `register_native_grammars`·`built_in_languages`(`[DonEditor]` 표시), 의존성은 워크스페이스 `Cargo.toml` 과 `crates/languages/Cargo.toml`.

| 언어(표시 이름) | 감지 확장자·파일 이름 | 문법 crate |
|---|---|---|
| Java | `java` | tree-sitter-java 0.23.5 |
| C# | `cs` `csx` | tree-sitter-c-sharp 0.23.5 |
| Ruby | `rb` `rake` `gemspec` `ru` `Gemfile` `Rakefile` (+ 셔뱅) | tree-sitter-ruby 0.23.1 |
| JSON5 | `json5` | tree-sitter-json5-orchard 0.1.0 |
| Properties | `properties` | tree-sitter-properties 0.3.0 |
| Swift | `swift` | tree-sitter-swift 0.7.4 |
| Scala | `scala` `sc` `sbt` | tree-sitter-scala 0.26.3 |
| R | `r` `R` (+ 셔뱅) | tree-sitter-r 1.3.0 |
| Batch | `bat` `cmd` | tree-sitter-batch 0.11.1 |
| PowerShell | `ps1` `psm1` `psd1` (+ 셔뱅) | tree-sitter-powershell 0.26.4 |
| Dart | `dart` | tree-sitter-dart 0.3.0 |

**쿼리를 그대로 쓰지 못한 이유 (중요한 발견):** Gram 은 같은 범위에 캡처가 여러 개면 **나중 캡처가 이기고**, 테마에 어떤 키로도 매칭되지 않는 이름은 "스타일 없음"이 되어 앞 캡처를 **지워 버린다**(`crates/language/src/buffer.rs` 의 하이라이트 스택). 매칭 규칙은 "테마 키의 모든 `.` 구성 요소가 캡처 이름에 있어야 하고 가장 긴 키 선택"(`highlight_map.rs`)이다. 그래서 crate 의 기본 쿼리 중 다음을 고쳤다 (각 `highlights.scm` 머리말에 변경 내용을 적었다).
- Swift: 한 줄 주석이 `@comment @spell` 이라 **주석 스타일이 지워짐** → `@spell` 제거
- Scala·R: nvim 방식 이름 `@conditional`·`@repeat`·`@include`·`@exception`·`@method`·`@storageclass` 는 매칭되는 키가 없어 `if`·`for`·`import`·메서드 이름이 무색 → `keyword.conditional`·`keyword.repeat`·`keyword.import`·`keyword.exception`·`function.method`·`keyword.modifier` 로 변경. 스타일을 지우는 `@none`·`@spell`·`(ERROR) @error` 는 제거(Scala 의 문자열 보간 `@none` 은 `@variable` 로 대체)
- PowerShell: 테마에 없는 `@array`·`@assignvalue` 패턴 제거, `@delimiter` → `@punctuation.delimiter`
- JSON5: 숫자는 `number`, 불리언은 `boolean`, 객체 키는 `property` 로 구분하도록 쿼리를 확장
- Java·C#·Ruby·Properties·Bat·Dart 는 수정 없이 사용

**테마 확인:** 번들 테마 67개(light 22, dark 45) 중 신규 언어가 쓰는 기본 키(comment·string·keyword·number·function·type·variable·constant·property·operator·boolean·punctuation·attribute·label·tag)가 모두 정의된 테마는 **42개**이며 기본 테마 One Dark·One Light 도 여기에 든다. 나머지 25개(Alabaster 계열·Everforest·Kanagawa·Sumi Light 등)는 원본 테마가 일부 키를 정의하지 않은 것으로 **기존 언어에도 동일하게 적용**된다. One Dark/Light 는 `keyword.import`·`keyword.conditional`·`number.float` 키가 없지만 최장 접두 규칙으로 `keyword`·`number` 색이 적용된다. (키 수준 확인이며 라이트/다크 각 테마의 실제 색상값을 화면에서 비교하지는 않았다)

**알려진 한계 / 결정이 필요한 것**
- **XHTML — 사용자 결정 필요:** `xhtml` 은 원본대로 HTML 문법을 쓴다. HTML 문법이 `<?xml ... ?>` 선언을 처리하지 못해 **그 한 줄만 파싱 오류(강조 없음)** 이고, 태그·속성·문자열·DOCTYPE 은 정상 강조된다(테스트는 이 파일을 알려진 예외로 표시). XML 문법으로 바꾸면 선언까지 강조되지만 `<script>`·`<style>` 안의 코드 강조와 HTML 전용 기능을 잃는다. R8(원본 동작 보존)을 우선해 현 상태로 두었다
- Batch 문법은 `@echo off`·`goto done`·`call :show "인자"` 처럼 **명령 한 줄 전체를 하나의 keyword 노드**로 만들어 줄 전체가 keyword 색이다(문법 구조의 한계)
- Markdown 샘플은 블록 수준(제목·목록·인용·코드 펜스)만 테스트로 검증했다. 인라인 강조와 코드 펜스 안의 언어 강조는 injection 이라 이 테스트 경로에 포함되지 않는다
- 신규 언어는 들여쓰기(`indents.scm`)·아웃라인·텍스트 객체·실행 가능 항목 쿼리가 없다(요구사항이 "기본적인 문법 강조"라 범위에서 제외). Swift·Scala 는 crate 에 해당 쿼리가 있으나 가져오지 않았다
- **눈으로 본 확인은 하지 않았다:** 하이라이트 파이프라인과 테마 키 수준은 자동 테스트로, GUI 는 "열림·응답·로그 오류 없음"까지만 확인했다. 실제 창에서 색이 어떻게 보이는지는 사용자 확인이 필요하다
- 릴리스 로그의 `conpty.dll` 경고(터미널이 제한 기능으로 동작)는 원본 설치 프로그램도 이 파일을 포함하지 않는 **기존 상태**로 보이며 이번 변경과 무관하다(코드 `check_for_conpty_dll`). 로그의 prettier 설치 실패(Node 없음)·셸 환경 읽기 실패 등 다른 ERROR 줄은 기준선과 비교하지 않았다

### Phase 5. 한국어 UI — 1차 완료 (2026-10-28)
- [x] **`crates/i18n` 구현:** 문자열 리소스 `assets/locales/en.json`·`ko.json`(키 383개, 바이너리에 내장), `i18n::t!("키")`(`&'static str`) / `t!("키", 이름 = 값)`(`String`), 키가 없으면 현재 언어 → 영어 → 키 이름 순으로 폴백, `{이름}` 자리 표시자는 한 번만 훑어 치환(값 안의 `{다른이름}` 은 다시 치환하지 않음 — 파일 이름 등이 들어가는 문장 보호). 언어 상태는 원자적 전역값이며 기본값은 영어라서 `init` 을 부르지 않는 기존 테스트는 영어 그대로 동작한다. 테스트·도구용 `with_language`(현재 스레드에서만 언어를 바꿈)도 제공
- [x] **설정 `ui_language`:** `"auto"`(기본, OS 언어를 따름 — 한국어이면 한국어, 그 외는 영어) / `"ko"` / `"en"`. `SettingsContent` 필드, `default.jsonc`, `UiLanguageSetting`, **설정 화면의 "일반" 맨 위에 드롭다운**을 추가했다. 시작할 때 설정을 읽은 직후 `i18n::init`(메뉴보다 먼저), 설정이 바뀌면 언어를 다시 적용하고 창을 새로 그리며 **메뉴는 다시 만든다**
- **기본값 주의:** `auto` 이므로 **OS 가 한국어인 PC 에서는 이제 별도 설정 없이 한국어로 시작한다.** 영어를 원하면 설정 파일에 `"ui_language": "en"`
- [x] **1차 적용 범위 (키 383개, `i18n::t` 호출 약 500곳):**
  - **애플리케이션 메뉴 전체**(112항목): 파일·편집·선택·보기·이동·실행·창·도움말과 앱 메뉴(설정 하위 메뉴 포함)
  - **탭·창:** 탭 컨텍스트 메뉴, 탭 바 버튼 툴팁, 새로 만들기·분할 메뉴, 저장 확인·덮어쓰기·삭제 충돌 대화상자(`pane.rs`), 종료·다시 시작 확인, 알림 툴팁, 도크 닫기/이동, 시작 화면(`welcome.rs`)
  - **패널:** 프로젝트·개요·Git·터미널·디버그 패널 툴팁(`persistent_name` 은 저장된 상태의 식별자라 **바꾸지 않음**), 프로젝트 패널 컨텍스트 메뉴와 **삭제·휴지통·덮어쓰기·되돌리기 확인 대화상자**(문장 조각 조합을 키 단위로 재구성), 개요 패널 메뉴, 터미널 패널·탭 메뉴
  - **에디터:** 우클릭 메뉴, 변경 덩어리(hunk) 툴팁, 시그니처 도움말, 빠른 동작 막대(선택·편집기 컨트롤 드롭다운)
  - **검색:** 버퍼 검색·프로젝트 검색·빠른 검색의 플레이스홀더·툴팁·버튼·옵션 이름
  - **상태바·타이틀바:** 커서 위치·언어·인코딩·줄 끝·진단·기호 개요 툴팁, 원격 연결 상태, 최근 프로젝트·브랜치 툴팁, 창 탭 메뉴
  - **선택기·팔레트:** 명령 팔레트·파일 찾기·기호 이동·줄 이동·언어/테마/아이콘 테마 선택기, 최근 프로젝트, 경로 입력 프롬프트
  - **기타:** 온보딩 일부, 설정 창 제목, `Gram` → `Don Editor` 표기 정리(설정 화면 설명 8건, 에디터 안내문, 시스템 정보 등)
- [x] **명령 팔레트(2차 일부):** 한국어일 때 메뉴에 있는 액션은 `"파일 열기 (workspace: open)"` 처럼 **한국어 이름과 영어 이름을 함께** 보여 줘 두 언어로 모두 검색된다. 한국어 이름은 메뉴 정의에서 자동으로 모은다(별도 번역 표 없음). 사용 기록·정렬 기준은 영어 이름을 그대로 쓰므로 언어를 바꿔도 기록이 유지된다. **메뉴에 없는 액션(약 1000개 중 대부분)은 영어 이름 그대로**
- [x] **자동 검증:** `cargo test -p i18n` 11개 — 영·한 키 일치, 자리 표시자 일치, 빈 값 없음, **소스가 쓰는 키가 모두 `en.json` 에 있고 `en.json` 에 죽은 키가 없음**(소스 전체를 훑는 검사), 폴백, 치환, 로케일 이름 매핑(`ko`·`ko-KR`·`ko_KR.UTF-8`), `with_language` 격리 등. `gram` 의 `test_app_menus_follow_the_display_language` — 실제 `app_menus` 를 영어·한국어로 만들어 영어 메뉴 이름이 원본과 같은지, 한국어 최상위 메뉴가 `파일·편집·선택·보기·이동·실행·창·도움말` 인지, 키 이름이나 `{자리표시자}` 가 노출되지 않는지, 번역되지 않은 항목이 제품 이름뿐인지 확인
- [x] **회귀 확인(영어 동작 보존, R8):** 변경한 crate 의 기존 테스트가 모두 통과 — **editor 475**(5개 무시), workspace 132, project_panel 69, file_finder 53, terminal_view 43, search 22, settings 23, command_palette 15, diagnostics 13, outline_panel 10, go_to_line 7, settings_ui 6, gram 35(기존 34 + 메뉴 테스트 1), i18n 11 (실패 0). `cargo fmt --all` 통과, 변경 crate clippy 경고 0건
- [x] **실제 화면 확인 (2026-10-28, 디버그 빌드를 직접 실행해 창을 캡처):** 이 PC(OS 언어 한국어)에서 격리된 `--user-data-dir` 로 실행하고, 창 메시지로만 클릭을 보내(마우스 커서는 움직이지 않음) 창 영역을 캡처했다.
  - **`"ui_language": "ko"`:** 메뉴 막대 `Don Editor 파일 편집 선택 보기 이동 실행 창 도움말`, 앱 메뉴(`Don Editor 정보`, `설정 열기`, `프로젝트 설정 열기`, `언어 서버`, `키맵 열기`, `테마 선택...`, `확장 기능`, `Don Editor 종료`), **파일 메뉴 전체**(`새 파일`, `새 창`, `파일 열기...`, `폴더 열기...`, `최근 항목 열기...`, `프로젝트에 폴더 추가…`, `저장`, `다른 이름으로 저장…`, `모두 저장`, `편집기 닫기`, `프로젝트 닫기`, `창 닫기`), **에디터 우클릭 메뉴**(`정의로 이동`, `선언으로 이동`, `구현으로 이동`, `모든 참조 찾기`, `기호 이름 바꾸기`, `버퍼 서식 지정`, `코드 작업 보기`, `잘라내기`, `복사`, `붙여넣기`, `파일 탐색기에서 보기`, `터미널에서 열기` 등), **탭 우클릭 메뉴**(`변경 없는 탭 닫기`, `모두 닫기`, `경로 복사`, `탭 고정`, `터미널에서 열기`)가 모두 한국어로 나오고 단축키 표시도 정상이다. **D2Coding 이 한글을 정상으로 그린다**(글자 깨짐·잘림 없음, 메뉴 폭도 자동으로 맞춰짐)
  - **`"ui_language": "en"`:** 앱 메뉴가 원본과 같은 영어(`About Don Editor`, `Open Settings` …)로 나온다
  - **`auto`(설정 없음):** OS 가 한국어이므로 한국어로 나온다. 의도치 않게 설정 파일이 잘못되었을 때의 오류 알림도 `사용자 설정 파일이 올바르지 않습니다` / `설정 파일 열기` 로 한국어였다
  - 보너스: 상태바에 `C#`, 코드에 문법 색이 보여 Phase 4 의 언어 감지·강조도 실제 창에서 확인되었다
  - **확인하지 못한 것:** 툴팁, 저장 확인·삭제 확인 같은 **대화상자**, 검색 바, 프로젝트 패널 메뉴, **명령 팔레트의 한·영 이중 이름**(키보드 입력이 필요해 보내지 않음), 설정 화면의 언어 드롭다운, **실행 중 언어 전환**(메뉴 재생성 포함), 잘못된 값(`"fr"`)의 동작. 한국어 문구가 어색하거나 긴 문장이 잘리는 곳이 더 있을 수 있다
  - 접근성 트리(UI Automation)는 GPUI 가 노출하지 않아(항목 0개) 글자를 코드로 읽을 수 없었고, 캡처 이미지를 눈으로 확인했다. 상태바의 `Failed to load environment variables` 는 프로젝트 환경 로드 오류 메시지로 번역 대상이 아니다
- 완료 기준: OS 언어가 한국어이면 주요 UI가 한국어로 표시되고, 설정으로 영어 전환 가능 — **주요 UI 는 충족(자동 테스트 수준), 전체는 아님**

**남은 것 (Phase 5 2차·3차, 정직한 범위)**
측정 지표(소스에서 UI 호출에 영어 리터럴이 그대로 들어간 곳을 센 것이며 정확한 총계는 아님): **남은 후보 약 390곳 / 적용된 호출 약 500곳.** 후보가 많은 곳은 `ui`(약 90, 대부분 컴포넌트 미리보기라 사용자 노출 아님), `git_ui`(약 44), `debugger_ui`(약 42), `language_tools`(약 35), `keymap_editor`(약 21), `repl`(약 21), `recent_projects` 의 원격 서버 화면(약 20)이다.
- **설정 화면 본문(`settings_ui/src/page_data.rs`):** 제목·설명이 681줄이 정적 표에 있다. 이 방식은 호출 지점 치환이 비효율적이라 필드에 키를 두고 그릴 때 번역하는 구조 변경이 필요하다. 지금은 "UI Language" 항목만 영·한 병기로 추가했다
- **키맵 편집기, Git 패널(`git_ui`), 디버거 UI, REPL, 언어 서버 도구, 원격 서버 관리 화면, 확장 기능 화면, Vim 표시, 진단 뷰, 활동 표시기의 상태 문구(언어 서버 진행 상황 등), 에디터 안에서 생성되는 안내·오류 메시지:** 영어 그대로
- **명령 팔레트의 메뉴에 없는 액션 이름**, 커서 위치 표시의 복수형 문구(`"3 selections"` 등, 영어 문법에 묶인 조합), 외부(LSP·확장)가 보내는 메시지는 번역 대상이 아니거나 구조상 어렵다
- 원본 도움말 문서(`GramEditor-docs/docs`)와 Phase 2 에서 정한 "앱 내 도움말 처리"는 별도 결정 사항

**설계 결정과 이유**
- **키 기반 호출(`t!("menu.file.open")`)을 택하고 "영어 원문을 키로 쓰는 일괄 자동 번역"은 쓰지 않았다.** `Label` 등에서 영어 문자열을 자동으로 번역하면 사용자 데이터(파일 이름, 브랜치 이름, 심볼 이름)도 같은 경로로 그려져 `Save` 라는 파일이 `저장` 으로 둔갑한다. 호출 지점마다 바꾸는 편이 느리지만 안전하다
- 번역 문자열을 `&'static str` 로 돌려주도록 해 `Panel::icon_tooltip` 처럼 정적 문자열을 요구하는 곳에도 쓴다. 영어 리소스는 원본 UI 문자열과 같아서(예외는 아래) 영어 동작이 바뀌지 않는다
- 영어 문구가 원본과 달라진 곳(의도적): 도움말 메뉴 "Gram Repository" → "Gram Repository (original)"(원본 저장소를 여는 항목이라서), 설정 화면 설명·에디터 안내문·시스템 정보·WSL 경로 오류 메시지의 "Gram" → "Don Editor"

**알아 둘 점 / 발견한 문제**
- 개발 중 `t_with` 의 치환 결함을 발견·수정했다(값에 `{warning}` 같은 글자가 있으면 다음 치환에서 다시 바뀜). 회귀 테스트를 추가했다
- `t!("` 로 끝나는 `format!("`·`assert!("` 를 키로 오인하는 소스 검사 오류도 고쳤다(앞 글자가 식별자 문자이면 제외)
- 번역 작업에는 "정확히 예상한 횟수만 일치할 때만 적용"하는 일회성 패치 도구를 썼다(저장소에는 남기지 않음). 일치 횟수가 어긋나면 파일을 전혀 바꾸지 않아, 같은 문자열이 다른 문맥에 또 있는 경우(`Filter Options` 툴팁+머리글, `Go to Line/Column` 두 분기 등)를 걸러냈다
- Windows 에서 `cmd`/`bash` 를 거치는 스크립트는 백슬래시가 깨질 수 있어 도구 스크립트는 파일로 저장해 실행했다

### Phase 6. 공용 도구 레지스트리 + 로컬 MCP 서버
- [x] `crates/don_tools` 도구 정의 구조 및 1차 도구(파일/편집/검색/범용 액션) 구현 — `8de5f9e`. 현재 도구 33개(`list_agent_conversations` 포함)
- [x] `crates/mcp_server` Streamable HTTP 서버 구현 (루프백, 토큰) — 루프백·Bearer·Host/Origin 검사, 세션·취소, 크기 제한. 실제 TCP 계약 테스트 통과
- [x] `don-mcp.exe` stdio 브릿지 구현 — `10efaee`(병렬 중계·요청/EOF 취소). Windows 배포 zip의 `bin\don-mcp.exe`로 동봉
- [x] 승인 대화상자(파괴적 동작) 및 MCP 켜기/끄기 메뉴·설정 추가 — 보기 메뉴의 MCP 켜기/끄기·연결 정보, 설정 `mcp_server.enabled/port`(기본 비활성). 승인·거절·취소의 실제 화면 조작 검증은 남음
- [x] 나머지 도구(UI, 설정, 터미널, 진단, 리소스) 구현 — 설정 조회/변경(2026-11-04부터 `agent` 항목 변경 거절), 터미널 실행(승인), 진단·개요, 패널 토글, 리소스 3개
- [ ] Claude Desktop / Claude Code 등 MCP 클라이언트 연결 설정 예시 문서화 및 실제 연결 테스트 — **문서화 완료(`docs/mcp.md`), 공식 Python MCP SDK로 실제 앱 HTTP·stdio 연결 검증 완료. Claude Desktop/Code 실제 앱 연결은 미검증**
- 완료 기준: 외부 MCP 클라이언트에서 파일 열기·편집·저장·검색·패널 토글·임의 액션 실행 가능 — **SDK 기준 도구 조회·호출은 확인, GUI 승인을 거치는 편집·저장의 실제 화면 확인과 Claude 앱 연결이 남음**

### Phase 7. Agent 사이드 패널
- [x] `crates/agent_panel` 패널 골격 (우측 기본 배치, 토글 액션, 메뉴) — `2b5390a`. 2026-11-04 실제 화면 확인(우측 표시, `Ctrl+Alt+A`)
- [x] 패널 숨김 및 숨긴 상태에서도 접근 가능한 별도 아이콘 구현, 아이콘 클릭으로 다시 표시 — 2026-11-04 실제 화면 확인
- [x] **(2026-11-04 사용자 요청)** 프로그램 실행 시 항상 숨긴 상태로 시작(이전에 열어 둔 상태 포함) — 실제 화면 확인
- [x] **(2026-11-04 사용자 요청)** 제공자 선택을 콤보박스로 변경(`Provider::ALL`, 이후 Google·OpenCode·LM Studio 추가 예정) — 실제 화면 확인
- [x] `LlmProvider` 트레이트 + Anthropic / OpenAI 구현 (스트리밍) — 계약 테스트 13개 통과, 루프백 캡처 서버로 실제 요청 형식 확인. 유효한 키의 실제 API 호출은 사용자 지시로 생략
- [x] API Key 입력·저장(자격 증명 관리자)·환경 변수 폴백 — **2026-11-04 사용자 요청으로 변경:** 기본 설정(설정 > 에이전트)에서 제공자별 키·API 서버 URL(공란이면 기본값) 입력, 키 없을 때만 패널에 인라인 키 입력. 저장·삭제·즉시 반영·환경 변수 표시를 실제 화면에서 확인
- [x] 채팅 UI, 컨텍스트 첨부, 중단, 오류 처리 — 2026-11-04부터 요청마다 열린 폴더·현재 파일·탭 경로 자동 첨부. 메시지 표시·URL 오류 표시는 실제 화면 확인, 중단 버튼은 실제 응답 스트림이 없어 화면 확인 못 함
- [x] `don_tools` 연동 도구 호출 + 편집 diff 미리보기/승인 — 구현·컴파일 완료, 요청에 도구 33개 포함 확인. 실제 모델이 도구를 호출해 승인창을 거치는 흐름은 미검증(실제 API 호출 생략)
- [x] 대화 기록 저장/불러오기 — 워크스페이스별 최근 20개, MCP `list_agent_conversations`·`SelectConversation`·`ClearHistory`. 재실행 시 제공자 선택 복원은 확인, 기록 목록의 화면 조작은 미확인
- 완료 기준: Claude/OpenAI 키 중 하나만 등록해도 기본 우측 패널에서 대화하고, 현재 파일을 수정하도록 지시하면 승인 후 반영됨. 패널을 숨긴 뒤에도 화면 내 별도 아이콘을 클릭해 다시 표시할 수 있음 — **아이콘 복귀·키 등록 흐름은 충족. 실제 모델과의 대화·승인 후 편집 반영은 실제 API 호출 생략으로 미확인**

### Phase 8. 통합 테스트 · 문서화 · 릴리스
- [ ] 전체 기능 회귀 테스트 ([기능 인벤토리](FEATURE-INVENTORY.md) 기준 기존 기능 R8 + 신규 요구사항 R1~R7·R9, Agent 패널 숨김·아이콘으로 다시 표시 포함)
- [ ] `cargo test` (변경 crate 위주), `cargo clippy` 경고 정리 — **대부분 완료(2026-11-05):** 변경 crate 테스트 text 24·language 99·worktree 26·editor 475·gram 36·markdown_preview 40·don_charset 9·don_typst 17·i18n 11·MCP 전송 25개 통과, `don_tools`·`mcp_server`·`agent_panel`·`settings_ui`·새 crate 4개 clippy 경고 0건(이전에 남았던 `don_tools` 1건·`mcp_server` 5건은 정리함). **남은 것:** 원본에 있던 `prettier` clippy 1건(손대지 않음), `settings_ui`·`workspace` 등 나머지 GUI crate 테스트 전체 실행
- [x] 사용자 문서(한국어): 설치, Agent 설정, MCP 연결 방법 — `docs/installation.md`·`agent.md`·`mcp.md`·`security.md`·`packaging.md`(`5fa69a6`), 2026-11-04 Agent 설정 변경 반영
- [x] Windows 배포 스크립트로 릴리스 zip 생성 및 압축 해제 실행 확인 — **2026-11-04 x86_64 릴리스 ZIP·SHA-256·구성 확인 및 격리 데이터로 GUI 기동 확인. 전체 기능 회귀와 깨끗한 설치 환경·ARM64는 별도 미검증**
- [ ] Linux/macOS 배포 스크립트로 각 OS의 최종 tar.gz 생성 및 압축 해제 실행 확인, 빌드 도구 검사·설치 질의 흐름 점검 — **모의 검증만 완료, 실제 OS 검증은 이번 목표에서 건너뜀(사용자 지시)**
- 완료 기준: Windows zip과 Linux/macOS tar.gz를 각 대상 OS에서 압축 해제 후 실행 가능, R1~R9의 모든 요구사항 동작 확인

---

## 6. 리스크 및 검토 사항

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 원본 Windows 빌드 동작 여부 불확실 | 전체 일정 | Phase 1에서 최우선 검증, 실패 원인 기록 |
| 빌드 시간·디스크 사용량 큼 | 개발 속도 | 측정(2026-10-27): 디버그 재빌드 7분 22초(537개 crate 는 재사용), 릴리스 20분 36초, `target/` 합계 약 14GB(릴리스 5.1GB). 증분 빌드 활용, 디버그 빌드 위주 개발. 문법 11개 추가 시 증가 예상 |
| i18n 대상 문자열이 135개 crate에 분산 | R1 범위 | 단계적 적용(1차~3차), 신규 기능은 처음부터 i18n 적용. **1차 완료, 남은 규모는 Phase 5 "남은 것" 참고.** 새 UI 문자열을 추가할 때는 `i18n::t!` 와 `assets/locales/{en,ko}.json` 에 함께 넣는다(키 정합성은 `cargo test -p i18n` 이 검사) |
| 설정 화면·키맵 편집기 등 정적 표에 든 문자열 | R1 | `settings_ui/src/page_data.rs` 에 제목·설명이 681줄 있는 식이라 호출 지점 치환 방식으로는 비효율적. 구조(필드에 키를 두고 그릴 때 번역)를 바꾸는 별도 작업이 필요 |
| tree-sitter 문법 crate ABI 비호환 | R2 | 호환 버전/git rev 고정 |
| 한글 폰트 폴백·IME 조합 입력 문제 | R1 사용성 | Phase 5에서 별도 검증 항목으로 관리 |
| 로컬 MCP 포트 노출 | 보안 | 루프백 전용, 토큰 인증, 기본 비활성, 파괴적 동작 승인 |
| API Key 유출 | 보안 | 자격 증명 관리자 저장, 로그·설정 파일에 기록 금지 |
| 원본의 LSP 자동 다운로드 등 외부 설치 동작 | 사용자 경험 | 동작 방식 확인 후 필요 시 기본값 조정 |
| GPL-3.0 의무 | 법적 | LICENSE 유지, 원저작 고지, 배포 시 소스 제공 방법 명시 (3.9 참고) |
| D2Coding `.ttc` 로드 호환성 | R6 | **대부분 해소(2026-10-27):** 로드를 막던 `.ttf` 전용 필터를 수정했고, DirectWrite 가 컬렉션을 지원함을 코드로 확인, 4개 글꼴(D2Coding·D2Coding ligature × Regular/Bold) 확인. 화면 표시는 사용자 시각 확인 대기. 문제 시 `.ttf` 분리 |
| 빌드 시 `RELEASE_CHANNEL` 환경 변수 누락 | 브랜딩 | 없으면 exe 에 dev(주황) 아이콘이 들어감. 배포 스크립트가 채널 파일 값을 지정하므로 **배포용 빌드는 `package-zip.bat` 로 할 것**(수동 `cargo build` 는 `$env:RELEASE_CHANNEL='stable'` 필요) |
| 이름 변경 시 Cargo 자동 발견 | 빌드 | 바이너리 이름을 바꾸면 `src/main.rs` 가 별도 바이너리로 자동 발견됨 → `autobins = false` 로 해결(적용 완료). Phase 3 에서 바이너리를 추가할 때도 `[[bin]]` 을 명시할 것 |
| 'Gram' 문자열 잔여 | R1·브랜딩 | 화면에 보이는 것은 Phase 5 에서 대부분 정리. 남은 것은 호환성 때문에 둔 식별자와 내부 메시지 (위 백로그 항목 참고) |
| 신규 문법 11개 추가로 빌드 시간·바이너리 크기 증가 | R2 | 묶음별로 추가하며 전후 크기·시간 측정 기록 (3.4 참고) |
| `tree-sitter-json5` 0.1.0 이 tree-sitter 0.20 의존 → 0.26 과 비호환 | R2 (json5) | **해소:** 호환되는 fork `tree-sitter-json5-orchard` 0.1.0 사용. 인지도 낮은 fork 라 유지보수가 끊기면 `jsonc` 문법 재사용이나 문법 소스 포함을 검토 |
| 번들 쿼리의 캡처 이름이 테마와 안 맞음 | R2 | 매칭되지 않는 캡처가 뒤에 오면 앞 캡처의 색을 지운다(Swift 주석 사례). 새 언어를 추가할 때 `language_support_tests.rs` 에 대표 토큰 기대값을 넣고 `DON_DUMP_HIGHLIGHTS=1` 로 확인할 것 |
| `.xhtml` 의 `<?xml ... ?>` 선언 줄이 HTML 문법에서 오류 | R2 (xhtml) | 그 줄만 강조 없음, 나머지 정상. XML 문법으로 바꿀지 사용자 결정 필요 (Phase 4 참고) |
| 내장 문법과 원본의 확장 기능(Swift·Dart·Scala·R·PowerShell·Java·C#·Ruby) 중복 | R2 | 확장 설치 시 우선순위·충돌 확인 |
| 신규 기능 추가로 기존 기능 회귀 | R8 | Phase 1 기능 인벤토리 작성, 각 Phase 스모크 테스트, Phase 8 회귀 테스트 (3.10 참고) |
| 원본 `ini`·`kdl` 언어는 문법이 등록되어 있지 않음(원본 자체 이슈 가능성) | R8 | 앱 실행 후 강조 여부 확인, 인벤토리에 "원본 자체 동작 여부"로 기록 |
| Spectre 완화 라이브러리 필수 (VS 2026 Community 18.x / MSVC 14.51 환경에서 미설치로 1차 빌드 실패) | 빌드 | Phase 1에서 확인됨. VS Installer 로 설치 필요. Phase 3 배포 스크립트의 사전 검사·설치 질의에 반드시 포함 (`vswhere -requires Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre`) |
| 원본의 AI 금지·인간 기여 정책 (README, CONTRIBUTING, CODE_OF_CONDUCT, docs/index.md 등) | 정책·문서 일관성 | 업스트림 기여자 대상 정책이며 Don Editor 방향(AI 적극 도입)과 충돌. 업스트림에는 기여(PR 등)하지 않고, 원본 문서는 [GramEditor-docs/](GramEditor-docs/) 로 분리 보관. 단, 앱 내 도움말이 아직 원본 문서를 표시함(Phase 2/5에서 교체 검토) |
| 원본 문서 이동에 따른 앱 코드 영향 | 빌드 | 루트 `docs/` 는 `assets.rs` 가 앱에 내장함 → 경로 수정 필수였음(반영 완료). 향후 원본 업데이트를 병합할 때 `docs/` 경로 충돌 주의 |

---

## 7. 작업 기록

| 날짜 | 내용 |
|------|------|
| 2026-10-27 | AGENTS.md 분석, GramEditor 소스 구조 조사, 개발 계획·절차 수립 (PROGRESS.md 작성) |
| 2026-10-27 | AGENTS.md 에 R6(기본 글꼴 D2Coding), R7(GPL-3.0 명기) 추가된 것을 계획에 반영 (3.8, 3.9 절, Phase 2 항목) |
| 2026-10-27 | 사용자 요청: 원본 `README.md` 를 이름 변경하고, 루트 `README.md` 를 Don Editor 설명(원본 README 링크 포함)으로 새로 작성 |
| 2026-10-27 | 사용자 지시: 원본 프로그램 문서를 별도 디렉터리 [GramEditor-docs/](GramEditor-docs/) 에 원본 그대로 이동(136개 파일, 원본과 바이트 동일 확인)하고 우리 문서의 원본 설명 단락에 링크 연결. 루트 `docs/` 가 앱에 내장되어 있어 `assets.rs`·`script/prettier`·`.gitattributes` 경로 수정. 링크 검사 결과 이동으로 깨진 링크 15개(소스 디렉터리·LICENSE 대상)는 원본 보존을 위해 그대로 둠 |
| 2026-10-27 | 사용자 지시: AI 적극 도입이 원본과의 가장 큰 차이점임을 문서에 반영. README.md 수정, PROGRESS.md 에 프로젝트 방향·설계 원칙·Phase 2 문서 정리 항목·리스크 추가 |
| 2026-10-27 | Phase 1: GramEditor 소스 복사 완료, `.gitignore` 보완, 빌드 도구 점검 완료. `rust-toolchain.toml` 에 지정된 Rust 1.99.1 툴체인이 rustup 으로 자동 설치됨. 원본 디버그 빌드 시작 (로그는 세션 스크래치패드 `build-debug.log`) |
| 2026-10-27 | Phase 1: 기준선 디버그 빌드 **실패** (537개 crate 컴파일 후 `msvc_spectre_libs` 패닉 — Spectre 완화 라이브러리 미설치). 앞서 "빌드 설정에서 참조되지 않으므로 필요 없을 수 있음"이라 한 판단은 틀렸음(의존 crate `pet*` 가 필수로 요구). 설치 여부를 사용자에게 확인하기 위해 중단 |
| 2026-10-27 | Phase 1: 사용자 승인 후 Spectre 완화 라이브러리를 VS Installer 로 설치(종료 코드 0, 1차 시도는 경로 따옴표 누락으로 실패). 기준선 디버그 빌드 재실행 시작 (로그 `build-debug-2.log`) |
| 2026-10-27 | **AGENTS.md 변경 반영:** (1) 문법 강조 언어 10개 → 26개로 확대, (2) R8 "기존 기능 최대한 포함" 추가. 원본 `crates/languages` 및 crates.io 조사 결과를 2.1 표·3.4 설계·3.10 절·Phase 4·리스크에 반영 (원본 내장 15개 + 신규 11개, json5 문법 crate 비호환 확인). 이 시점에 코드 변경 없음 — 계획 문서만 갱신 |
| 2026-10-27 | Phase 1: 기준선 디버그 빌드 **성공** (오류 0·경고 0, 7분 22초, `gram.exe` 약 200MB). 사용자 요청으로 README.md 에도 26개 언어·R8·Spectre 필수 조건·빌드 성공 사실 반영 |
| 2026-10-27 | Phase 1: 기준선 앱 실행 확인 — 격리 데이터 폴더(`--user-data-dir`)로 실행, 창 2초 내 생성·응답 정상·파일 열림 확인 후 해당 프로세스만 정상 종료. 원본 Gram 의 설정 폴더는 변경되지 않음. 원본과 앱 식별자(`Gram-Editor-Stable`)가 같아 동시 실행 불가함을 확인 → Phase 2 과제로 기록. GUI 편집·저장·한글 입력은 에이전트가 확인하지 못함(사용자 수동 확인 요청) |
| 2026-10-27 | Phase 1: 사용자가 화면 렌더링·GUI 편집·저장·한글 입력(IME)을 직접 확인함 → **확인 완료로 표시**. 디버그 빌드 기준 Phase 1 완료 기준(실행, 파일 열기/편집/저장) 충족. 남은 것: 릴리스 빌드, 원본 기능 인벤토리, 기준선 커밋 |
| 2026-10-27 | Phase 1: 사용자 허락으로 기준선 작성. 사용자가 이미 `master` 에 3개 커밋을 만들어 둔 상태라 이력은 그대로 두고, 순수 GramEditor 3.4.0 트리만 담은 부모 없는 커밋 `961c4dc` 를 만들어 태그 `gram-baseline-3.4.0` 부여(`master` 불변 확인). 원본 대비 diff 가 의도한 변경(수정 5·추가 4·이름 변경 138·삭제 0)뿐임을 검증. 릴리스 빌드(`cargo build --release`) 백그라운드 시작 |
| 2026-10-27 | Phase 1: 릴리스 빌드 **성공** (오류 0·경고 0, 20분 36초, `gram.exe` 약 151MB). 릴리스 바이너리도 격리 데이터 폴더로 실행해 창·응답·파일 열기 확인. **Phase 1 완료** |
| 2026-10-27 | Phase 2 시작 (사용자 요청). 사전 조사: "Gram" 사용처(식별자·경로·리소스·UI 문자열 312건 중 화면 노출 약 109건), `.ttc` 로드 경로(`load_fonts` 의 `.ttf` 전용 필터 발견), DirectWrite 의 컬렉션 지원, `.ttc` 내부 글꼴 4개·라이선스 정보 분석 |
| 2026-10-27 | Phase 2 구현: 앱 식별자(`DonEditor-Stable/Dev`)·표시 이름·데이터/설정/로그 경로·실행 파일명(`doneditor`)·Windows 리소스 문자열·CLI·메뉴·시작 화면·정보 대화상자 변경, `.ttc` 로드 수정, 기본 글꼴 D2Coding, D2Coding 라이선스 고지·`licenses.md` 갱신, 임시 아이콘·로고(`script/make_placeholder_icon.py`). 별칭 `.GramMono`/`.GramSans` 매핑은 테스트 영향 때문에 변경하지 않음. 변경한 원본 코드에는 `[DonEditor]` 주석 표시 |
| 2026-10-27 | Phase 2 빌드 중 문제: 바이너리 이름 변경 후 Cargo 가 `src/main.rs` 를 두 번째 바이너리(`gram`)로 자동 발견(매니페스트 경고) → 빌드를 중단하고 `autobins = false` 추가 후 재시작. 디버그 빌드 **성공** (오류 0·경고 0, 10분 10초, 이전 빌드 결과 재사용으로 일부 crate 만 컴파일) |
| 2026-10-27 | Phase 2 검증: 이전 빌드와 새 빌드 **동시 실행** 성공(별개 PID·창, 뮤텍스 2종 존재), 새 데이터 폴더 `DonEditor` 생성, 원본 Gram 폴더 수정 시각 불변, 실행 파일 속성 "Don Editor" 확인. 화면 표시(글꼴·메뉴·아이콘)는 사용자 시각 확인 대기. 소스 공개 위치 등 5건의 결정 요청을 문서 상단에 정리 |
| 2026-10-27 | Phase 1: 원본 기능 인벤토리 [FEATURE-INVENTORY.md](FEATURE-INVENTORY.md) 작성 (R8). 부수 발견: 기본 `cargo build` 는 `cli.exe` 를 만들지 않음 → Phase 3 항목에 추가 |
| 2026-10-27 | **Phase 3 착수·구현:** `script/package-zip.ps1`(점검 6항목·설치 질의·winget/VS Installer 설치·VS 개발자 환경·빌드·스테이징·zip·SHA256·SOURCE.txt·선택적 소스 zip)과 `package-zip.bat` 작성. 조사 중 `vswhere -latest` 가 C++ 도구 없는 VS 2017 을 고를 수 있음, `RELEASE_CHANNEL` 없으면 dev 아이콘이 들어감을 발견 |
| 2026-10-27 | Phase 3 검증: `-CheckOnly`/`-DryRun`/거절/EOF 경로, VS 설치 명령 따옴표를 시스템 변경 없이 확인(PATH 숨김·가짜 컴포넌트 ID 사용). 디버그 프로파일로 전체 흐름 통과 → 압축 항목 이름 `\` 문제 발견·수정. `.bat` 의 `chcp 65001` 이후 UTF-8 한글 파싱 오류 발견 → ASCII 전용으로 재작성. **릴리스 실행:** 9분 10초, zip 53.4MB, 압축 해제 후 CLI·GUI 실행·아이콘(파랑) 확인. (진행 중 실수: 완료 감시를 걸 때 감시 프로세스가 자기 자신을 매칭하는 조건을 써서 30분을 헛되이 기다림 — 결과는 로그로 확인) |
| 2026-10-27 | **Phase 4 착수·구현:** crates.io 조회로 문법 crate 11개의 버전·`tree-sitter-language ^0.1` 의존(0.26 호환) 확인, json5 는 비호환이라 `tree-sitter-json5-orchard` 로 대체. 스크래치 crate 로 문법 로드·파싱·쿼리 컴파일을 먼저 검증. `tests/samples/` 29개 작성, 11개 언어 디렉터리(config·highlights·brackets·overrides·LICENSE) 작성, Cargo·`lib.rs` 등록, Python `pyw`·XML 확장자 보강, `assets/licenses.md` 고지 추가 |
| 2026-10-27 | Phase 4 검증: 쿼리가 Gram 규칙(마지막 캡처 우선, 테마 키 최장 매칭)에서 일부 무색이 되는 것을 발견(Swift 주석 등)해 Scala·R·Swift·PowerShell·JSON5 쿼리를 조정. 검증 테스트 추가 — languages 61개·gram 34개 통과. XHTML 의 XML 선언 파싱 오류 발견(알려진 한계로 기록). 릴리스 빌드 19분 40초, exe +19MB(+12.6%), zip 55.1MB. 릴리스 zip 에서 신규 언어 샘플 11개를 GUI 로 열어 응답 확인. (실수 1건: 처음 만든 Python 보조 스크립트의 백슬래시가 셸을 거치며 깨져 파일로 저장해 실행하는 방식으로 바꿈) |
| 2026-10-28 | **Phase 5 착수·1차 구현:** `crates/i18n`(리소스·`t!`·폴백·치환·`with_language`·설정 `ui_language`·팔레트 이중 이름)과 `assets/locales` 를 만들고, 메뉴 112항목을 시작으로 탭·패널·컨텍스트 메뉴·검색·상태바·대화상자·선택기·온보딩 일부를 키 단위로 치환(키 383개). 설정 화면에 언어 드롭다운 추가. 도구 스크립트로 일치 횟수를 검증하며 일괄 치환 |
| 2026-10-28 | Phase 5 검증: i18n 11개·gram 메뉴 테스트 통과, 변경 crate 기존 테스트 회귀 없음, fmt·clippy 통과. 발견·수정: `t_with` 재치환 결함, 소스 스캔 오탐, 설정 필드 누락(`vscode_import`), 병렬 테스트의 전역 언어 경쟁(→ `with_language` 스레드 격리). 한계 기록: 설정 화면 본문·키맵 편집기·Git 패널 등 미적용 |
| 2026-10-28 | Phase 5 실제 화면 확인: 디버그 빌드(5분 53초)를 격리 폴더로 실행해 ko/en/auto 를 창 캡처로 확인 — 한국어 메뉴 막대·앱 메뉴·파일 메뉴·에디터/탭 우클릭 메뉴 정상, 영어는 원본과 동일, D2Coding 한글 렌더링 정상. (실수: 첫 시도에서 설정 JSON 의 따옴표가 자식 PowerShell 인자 전달 중 사라져 ko/en 이 모두 auto 로 동작 — 알림 문구로 알아채고 스크립트가 설정 파일을 직접 만들게 고쳐 재확인. 또 `LP` 라는 함수 이름이 PowerShell 별칭 `Out-Printer` 와 충돌해 클릭이 실패했으나 인쇄는 일어나지 않았고 이름을 바꿔 재실행) |
| 2026-11-04 | **AGENTS.md 변경 문서 반영:** 변경 커밋 `ab97a9a`를 확인해 R4(Agent 패널 숨김·별도 아이콘으로 다시 표시), R5(Windows zip), 신규 R9(Linux/macOS sh 기반 tar.gz·빌드 도구 검사·미설치 시 설치 질의)를 요구사항·설계·목표 구조·Phase 3 후속/7/8·다음 작업에 반영. FEATURE-INVENTORY.md에 R1~R9 상태표 및 신규 요구사항 확인 목록 추가. HJOW-Athlon의 원본 H: 경로와 현재 Phase 5 커밋 상태도 정정. 두 Markdown 문서의 diff·요구사항 정합성을 검토했으며, 앱 소스 변경·빌드·설치는 수행하지 않음 |
| 2026-11-04 | **Phase 3(R5·R9 배포 스크립트) Linux 재검증:** `package-tar.sh`·`install-build-tools.sh` 구문 검사, `test_package_tar.py` 14개·`test_install_build_tools.py` 19개 통과, `--check-only`(종료 코드 1)·EOF/`0` 취소(빌드 전 중단·종료 코드 1·`dist/` 미생성) 확인, `package-zip.bat` ASCII·`package-zip.ps1` BOM·CRLF 확인. Phase 3 후속 절을 "미구현"에서 "스크립트 구현·모의 검증 완료, 실제 OS 빌드·실행 미검증"으로 정정. 실제 릴리스 빌드·tar.gz 생성·실행, 시스템 패키지 설치, macOS·ARM64 검증은 수행하지 않음(Phase 3 후속 "Linux 검증 결과" 참고). 앱 소스 변경 없음 |
| 2026-11-04 | **Phase 6·7 구현·검증:** 공용 도구 33개·로컬 MCP(HTTP/stdio, 세션 취소)·Agent 패널 초기 구현(`8de5f9e`·`10efaee`·`2b5390a` 이후 후속 커밋). Windows 전체 빌드·격리 실행, 공식 Python MCP SDK로 실제 앱 연결 검증. 실제 API 호출 검증은 사용자 지시로 생략 (상단 체크포인트 참고) |
| 2026-11-04 | **Agent 패널 수정(사용자 요청):** 시작 시 숨김, 제공자 콤보박스, 설정 > 에이전트의 제공자별 API 키(OS 자격 증명 저장소)·API 서버 URL, 키 없을 때 인라인 키 입력, 요청별 폴더·파일 경로 자동 첨부. Windows 실제 화면 검증 후 커밋 `a2ad462`. 이어서 PROGRESS.md 체크 항목과 한국어 도움말(`docs/agent.md` 등)을 최신화 |

---

## 8. 다음 작업

1. ~~Spectre 완화 라이브러리 설치~~ — 완료 (해결안 A)
2. ~~`cargo build` 재실행~~ ~~격리 폴더로 실행 확인~~ — 완료(창 생성·응답·파일 열기, 그리고 사용자가 GUI 편집·저장·한글 입력(IME)·화면 렌더링까지 직접 확인). 실행 방법: `target\debug\gram.exe --user-data-dir <임의 폴더> <파일>` (원본 Gram 과 식별자가 같으므로 원본 Gram 을 먼저 종료할 것)
3. ~~릴리스 빌드 확인~~ — 완료. 기능 인벤토리·기준선 태그도 완료 (**Phase 1 완료**). 이후 인벤토리의 ⬜ 항목은 앱을 쓰면서 확인되는 대로 갱신
4. ~~Phase 2 (리브랜딩 + D2Coding 기본 글꼴)~~ — 구현·검증 완료 (2026-10-27). **남은 것: 상단 "사용자 결정·확인 요청" 5건**
5. ~~Phase 3 (Windows 배포 zip 스크립트, R5)~~ — 구현·검증 완료 (2026-10-27). **남은 것:** 깨끗한 환경에서의 실제 설치 흐름 확인(Phase 3 "검증하지 못한 것" 참고). **Linux/macOS tar.gz(R9)는 스크립트 구현·모의 검증 완료(`f1ba957`, Phase 3 후속 "Linux 검증 결과" 참고). 남은 것은 실제 OS 릴리스 빌드·tar.gz 생성·압축 해제·실행, 깨끗한 환경 설치, ARM64, macOS 서명/공증이다.**
6. ~~README.md 에 AGENTS.md 변경(26개 언어, R8) 반영~~ — 완료 (2026-10-27)
7. ~~Phase 4 (문법 강조 26개)~~ — 구현·검증 완료 (2026-10-27). **남은 것:** 실제 창에서 색상 눈으로 확인(`tests/samples/` 의 파일을 열어 보면 됨), XHTML 처리 결정(Phase 4 "알려진 한계"). (변경분은 커밋 `78320a1` 에 포함됨) 빌드 크기 기준값(이후 비교용): 릴리스 `doneditor.exe` 170,434,048 바이트, zip 55.1MB
8. ~~Phase 5 1차 (한국어 UI)~~ — 완료 (2026-10-28, 현재 저장소의 `f061f14`에 소스 포함). **남은 것:** 대화상자·툴팁·검색 바·명령 팔레트 이중 이름·언어 전환의 실제 화면 확인(메뉴·우클릭 메뉴는 확인함), Phase 5 2차·3차(설정 화면 본문 구조 변경, 키맵 편집기, Git 패널, 디버거 등 — Phase 5 "남은 것"). **다음 후보는 Phase 5 2차, Phase 6 (공용 도구 + 로컬 MCP 서버), Phase 3 후속(R9)**이다. 병행 가능한 사용자 결정: 상단 "사용자 결정·확인 요청 (Phase 2)" 5건. 특히 5번(소스 공개 위치)은 `SOURCE.txt` 의 "소스 저장소" 항목과 `assets/licenses.md` 에 들어갈 내용이므로 배포 전에 필요
9. ~~**Phase 6·7 (MCP, Agent 패널)** 구현~~ — 구현 및 Windows 실제 실행 검증 대부분 완료 (2026-11-04, Phase 6·7 체크 항목 참고). **남은 것:** Claude Desktop/Code 실제 연결, MCP 편집 승인·거절·취소의 화면 조작, Agent 중단·기록 목록 화면 조작, 실제 API 호출(사용자 지시로 생략)
10. ~~AGENTS.md 최신 요구사항을 PROGRESS.md·FEATURE-INVENTORY.md에 반영~~ — 완료 (2026-11-04). 새 요구사항의 구현 완료를 뜻하지 않으며, R4는 후속 작업으로 남아 있다. R9는 스크립트 구현·모의 검증까지 완료이며 실제 OS 빌드·실행 검증이 남는다.
11. **Phase 8** — 남은 clippy 경고(`don_tools` 1, `mcp_server` 5) 정리, GUI crate 테스트 실행, 신규 코드 릴리스 ZIP 생성·압축 해제·실행, 기능 인벤토리 미확인 항목 회귀 확인
