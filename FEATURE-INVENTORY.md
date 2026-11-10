# 원본 기능 인벤토리 및 추가 요구사항 (R1~R9)

> 작성: 2026-10-27 · 기준: GramEditor 3.4.0 (git 태그 `gram-baseline-3.4.0` = 원본 무수정) · 관련: [PROGRESS.md](PROGRESS.md) 3.10절
> 최종 갱신: 2026-11-04 · 한국어 UI 후속 구현, 로컬 MCP·Agent 패널·플랫폼별 배포·라이선스 검사 결과 반영

AGENTS.md 의 "기존에 존재하던 기능은 최대한 포함한다"(R8)를 지키기 위해, 원본 GramEditor 의 기능을 목록으로 만들고 **우리 변경으로 깨지지 않았는지** 확인하는 기준으로 쓴다.

1~6절은 원본 기능 보존 기준이며, Don Editor의 추가 요구사항과 확인 항목은 7~8절에서 관리한다. 이번에는 실제 구현과 자동 검증 결과를 반영했다. **새 소스 전체의 앱 통합 빌드·GUI 실행 검증은 아직 완료하지 않았다.** 기존 50개 영역의 실행 확인 수는 22개(2026-11-05 스모크 점검 반영)이며, 미확인 28개 영역은 완료 처리하지 않는다.

## 읽는 법

- **빌드 포함**: `cargo tree -p gram --target x86_64-pc-windows-msvc` 의존 트리에 해당 crate 가 들어 있는지 조사한 **정적 결과**다. ✅ 전부 포함 / ⚠ 일부는 별도 바이너리 / ❓ 저장소에 없음.
- **실행 확인**: 앱을 실제로 실행해 동작을 본 결과다. ✅ 확인됨 / ⬜ 아직 확인하지 않음. **⬜ 은 "동작하지 않는다"가 아니라 "아직 모른다"는 뜻이다.**
- 이 표는 Windows 기준이다. 기능별 실행 확인은 사람이 직접 앱을 써 봐야 하는 것이 많아, 확인되는 대로 이 문서를 갱신한다.
- 7절에서는 **구현**, **자동 검증**, **실제 앱·API·OS 검증**을 구분한다. 가짜 명령을 이용한 macOS 테스트를 실제 macOS 빌드·실행으로 간주하지 않으며, 모의 API 응답도 OpenAI/Claude 실사용 검증과 구분한다.

### AGENTS.md에 따른 작업 기준

- 작업 전에 PROGRESS.md를 읽고, 완료·중단 시 진행 상황을 한국어로 기록한다. 코드 변경 시 상수·함수·반복문에 한국어 주석을 단다.
- GPL-3.0과 GramEditor 원본 라이선스·저작권 고지를 유지한다. 배포 형식별 포함 여부는 8절에서 점검한다.
- 현재 PC(HJOW-Athlon)의 원본 소스 위치는 `H:\Workspace\git\GramEditor`이다. AGENTS.md의 `D:\Workspace\git\GramEditor` 링크 표기는 이전 경로이며, 원본 저장소는 [GramEditor/gram](https://codeberg.org/GramEditor/gram)이다. 현재 H: 경로와의 전체 소스 대조는 수행하지 않았다.

### 원본 문서가 밝힌 Windows 관련 사실

- Windows 지원은 "일반적으로 충분히 테스트되지 않았다" ([remote-development.md](GramEditor-docs/docs/remote-development.md)).
- Windows 는 SSH/WSL 원격 개발의 **클라이언트로만** 지원되고, 원격 서버로는 아직 지원되지 않는다 (같은 문서).
- DirectX 11 호환 GPU 가 필요하다 ([windows.md](GramEditor-docs/docs/windows.md)).
- 원본 [개발 문서](GramEditor-docs/docs/development/windows.md) 는 "Windows 빌드가 동작하는지 알 수 없다"고 적었으나, **이 저장소에서는 디버그·릴리스 빌드와 실행이 성공**했다 (PROGRESS.md Phase 1).

## 1. 지금까지 실행으로 확인된 것

| 확인 항목 | 확인자 | 비고 |
|-----------|--------|------|
| 앱 실행, 창 생성(2초 내), 응답 정상 | 에이전트 | 격리 데이터 폴더(`--user-data-dir`)로 실행 |
| 인자로 넘긴 파일 열기 | 에이전트 | 창 제목으로 확인 |
| 릴리스 빌드(`cargo build --release`) 후 앱 실행·창 생성·파일 열기 | 에이전트 | 20분 36초, `gram.exe` 약 151MB |
| Phase 2 빌드(`doneditor.exe`)를 원본 식별자 빌드와 동시 실행, 새 데이터 폴더(`DonEditor`) 생성, 원본 Gram 폴더 불변 | 에이전트 | 2026-10-27. 글꼴·메뉴·아이콘의 실제 모습은 미확인 |
| 화면 렌더링, GUI 편집·저장, 한글 입력(IME) | 사용자 | 2026-10-27 |

## 2. 기능 목록 (50개 영역)

| 영역 | 기능 | 담당 crate | 빌드 포함 | 원본 문서 | 실행 확인 | 비고 |
|------|------|-----------|-----------|-----------|-----------|------|
| 편집 기본 | 텍스트 편집·선택·되돌리기 | `editor`, `text`, `rope` | ✅ | [multibuffers](GramEditor-docs/docs/multibuffers.md) | ✅ | 파일 열기·편집·저장, 화면 렌더링 — 사용자 확인(2026-10-27) |
| 편집 기본 | 한글 입력(IME) | `editor`, `gpui` | ✅ | - | ✅ | 사용자 확인(2026-10-27, 원본 글꼴 기준). **D2Coding 적용(Phase 2) 후 재확인 필요** |
| 편집 기본 | 다중 버퍼(Multibuffers) | `multi_buffer` | ✅ | [multibuffers](GramEditor-docs/docs/multibuffers.md) | ✅ | 2026-11-05 확인: 프로젝트 검색(`Ctrl+Shift+F`) 결과가 여러 파일의 발췌를 하나의 다중 버퍼 탭으로 보여 줌. 진단·참조 찾기 등 다른 진입점은 미확인 |
| 편집 기본 | SuperTab | `editor` | ✅ | [supertab](GramEditor-docs/docs/supertab.md) | ⬜ |  |
| 편집 기본 | 줄 이동 | `go_to_line` | ✅ | - | ✅ | 2026-11-05 확인: 줄 이동 창에 3을 입력하면 "3번째 줄로 이동" 안내가 뜨고 커서가 3줄로 이동 |
| 편집 기본 | 줄 끝(개행)·인코딩 선택 | `line_ending_selector`, `encoding_selector` | ✅ | - | ✅ | 2026-11-05 확인: 인코딩 표시·선택 화면·다시 불러오기·저장 거절, 줄 끝 선택 목록과 CRLF 선택 |
| 편집 기본 | 스니펫 | `snippet`, `snippet_provider`, `snippets_ui` | ✅ | [snippets](GramEditor-docs/docs/snippets.md) | ⬜ | 2026-11-05 `snippet` 단위 테스트 8개 통과(앱 실행 확인은 아직) |
| 탐색 | 탭 전환 | `tab_switcher` | ✅ | [tab-switcher](GramEditor-docs/docs/tab-switcher.md) | ✅ | 2026-11-05 확인: Ctrl+Tab 으로 탭 전환(열린 탭 목록 표시) |
| 탐색 | 명령 팔레트 | `command_palette`, `command_palette_hooks` | ✅ | [command-palette](GramEditor-docs/docs/command-palette.md), [all-actions](GramEditor-docs/docs/all-actions.md) | ✅ | 2026-11-05 확인: 열기·검색·실행(한국어/영어 이름 병기) |
| 탐색 | 파일 찾기 | `file_finder` | ✅ | - | ✅ | 2026-11-05 확인: `lib` 검색 결과·새 파일 만들기 항목 |
| 탐색 | 프로젝트 패널 | `project_panel` | ✅ | - | ✅ | 2026-11-05 확인: 폴더 트리·파일 아이콘·활성 파일 강조 |
| 탐색 | 개요(Outline) 패널·개요 | `outline_panel`, `outline` | ✅ | [outline-panel](GramEditor-docs/docs/outline-panel.md) | ✅ | 2026-11-05 확인: Rust 파일에서 `fn main` 항목 표시, 개요 없는 파일은 "No outlines available" 안내(이 안내 문구는 원본 영어 그대로라 한국어화 백로그 대상) |
| 탐색 | 심볼 검색 | `project_symbols` | ✅ | - | ⬜ |  |
| 탐색 | 검색·치환 (버퍼/프로젝트) | `search` | ✅ | - | ✅ | 2026-11-05 확인: 버퍼 검색(일치 2건·1/2)·프로젝트 검색(멀티버퍼 결과). 치환은 미확인 |
| 탐색 | 최근 프로젝트·파일 | `recent_projects`, `recent_files` | ✅ | - | ✅ | 2026-11-05 확인: Ctrl+R 로 최근 프로젝트 선택 창이 열리고 원격/로컬 폴더 열기 단축키가 표시됨. 격리 데이터 폴더라 기록이 비어 있어 목록에서 항목을 골라 여는 동작은 미확인 |
| 탐색 | Which-key 도움말 | `which_key` | ✅ | - | ⬜ |  |
| 탐색 | 일지(Journal) | `journal` | ✅ | - | ⬜ |  |
| 언어/코드 지능 | 언어 지원·문법 강조(tree-sitter) | `languages`, `language`, `language_extension` | ✅ | [languages](GramEditor-docs/docs/languages.md), [configuring-languages](GramEditor-docs/docs/configuring-languages.md) | ✅ | 원본 내장 15개 언어(2.1) 중 일부는 Phase 4에서 확인. `ini`·`kdl` 은 문법 미등록(원본 자체 이슈 후보); 2026-11-05 확인: Rust·Markdown·Typst 문법 강조(전체 26개 언어는 미확인) |
| 언어/코드 지능 | 언어 선택기·언어 도구 | `language_selector`, `language_tools` | ✅ | - | ✅ | 2026-11-05 확인: 상태 표시줄 언어 이름을 누르면 언어 목록이 열림. 언어 도구(LSP 로그 등)는 미확인 |
| 언어/코드 지능 | LSP(언어 서버) | `lsp` | ✅ | [language-servers](GramEditor-docs/docs/language-servers.md) | ⬜ | 언어 서버 자동 다운로드 동작은 별도 점검 필요(PROGRESS 리스크) |
| 언어/코드 지능 | 자동 완성 | `editor`, `language` | ✅ | [completions](GramEditor-docs/docs/completions.md) | ⬜ |  |
| 언어/코드 지능 | 진단 | `diagnostics` | ✅ | [diagnostics](GramEditor-docs/docs/diagnostics.md) | ⬜ |  |
| 언어/코드 지능 | 툴체인 선택 | `toolchain_selector` | ✅ | [toolchains](GramEditor-docs/docs/toolchains.md) | ⬜ |  |
| 언어/코드 지능 | 코드 포매팅(Prettier) | `prettier` | ✅ | - | ⬜ |  |
| 개발 도구 | Git (패널·diff·blame·그래프) | `git`, `git_ui`, `git_graph`, `git_hosting_providers` | ✅ | [git](GramEditor-docs/docs/git.md) | ⬜ | Git 실행 파일 필요 |
| 개발 도구 | 터미널 | `terminal`, `terminal_view` | ✅ | - | ⬜ | 원본 문서: Windows 에서 셸 프로필 스크립트가 종료되면 활성화 스크립트가 안 돌 수 있음 |
| 개발 도구 | 작업 실행(Tasks) | `task`, `tasks_ui` | ✅ | [tasks](GramEditor-docs/docs/tasks.md) | ⬜ |  |
| 개발 도구 | 디버거(DAP) | `dap`, `dap_adapters`, `debugger_ui`, `debugger_tools`, `debug_adapter_extension` | ✅ | [debugger](GramEditor-docs/docs/debugger.md) | ⬜ |  |
| 개발 도구 | REPL / Jupyter | `repl` | ✅ | [repl](GramEditor-docs/docs/repl.md) | ⬜ |  |
| 개발 도구 | 원격 개발 (SSH / WSL) | `remote`, `remote_server` | ⚠ 일부 별도: remote_server | [remote-development](GramEditor-docs/docs/remote-development.md) | ⬜ | 원본 문서: Windows 는 원격 '클라이언트'로만 지원, 원격 '서버'는 미지원. `remote_server` 는 별도 바이너리(gram 의존 트리 밖) |
| 편집 모드 | Vim 모드 | `vim`, `vim_mode_setting` | ✅ | [vim](GramEditor-docs/docs/vim.md) | ⬜ | 2026-11-05 `vim` 단위 테스트 167개 통과(이번 세션이 바꾼 `editor` 위에서 회귀 없음. 앱 실행 확인은 아직) |
| 편집 모드 | Helix 모드 | `vim` | ✅ | [helix](GramEditor-docs/docs/helix.md) | ⬜ | 같은 `vim` crate의 단위 테스트 167개 통과(앱 실행 확인은 아직) |
| 설정/외형 | 설정 파일·설정 UI·설정 프로필 | `settings`, `settings_ui`, `settings_profile_selector` | ✅ | [configuring-gram](GramEditor-docs/docs/configuring-gram.md), [environment](GramEditor-docs/docs/environment.md) | ✅ | 2026-11-05 확인: 설정 창이 한국어로 열리고 13개 페이지(에이전트 포함) 표시. 프로필은 미확인 |
| 설정/외형 | 키맵·단축키 편집기 | `keymap_editor` | ✅ | [key-bindings](GramEditor-docs/docs/key-bindings.md) | ✅ | 2026-11-05 확인: 메뉴의 "키맵 열기"로 단축키 편집기 탭이 한국어 열 제목(작업·인자·키 입력·컨텍스트·출처)과 함께 열림. 단축키 수정·저장은 미확인 |
| 설정/외형 | 테마 | `theme`, `theme_selector`, `theme_importer`, `theme_extension` | ⚠ 일부 별도: theme_importer | [themes](GramEditor-docs/docs/themes.md) | ✅ | 2026-11-05 확인: 테마 선택 목록 표시 |
| 설정/외형 | 아이콘 테마·파일 아이콘 | `file_icons` | ✅ | [icon-themes](GramEditor-docs/docs/icon-themes.md) | ✅ | 2026-11-05 확인: 프로젝트 패널의 폴더·파일 종류별 아이콘(기본 아이콘 테마). 아이콘 테마 바꾸기는 미확인 |
| 설정/외형 | 시각 사용자화 | `ui` | ✅ | [visual-customization](GramEditor-docs/docs/visual-customization.md) | ⬜ |  |
| 확장/연동 | 확장 기능(Wasm) 설치·관리 | `extension`, `extension_host`, `extension_api`, `extensions_ui` | ⚠ 일부 별도: extension_api | [extensions](GramEditor-docs/docs/extensions.md) | ⬜ | 원본 README: 제한적 지원. 확장이 임의 실행 파일을 내려받을 수 있다는 경고 있음 |
| 확장/연동 | Zed 설정 가져오기(마이그레이션) | `migrator` | ✅ | [zed](GramEditor-docs/docs/migrate/zed.md), [vs-code](GramEditor-docs/docs/migrate/vs-code.md) | ⬜ |  |
| 뷰어 | Markdown 미리보기 | `markdown_preview`, `markdown` | ✅ | - | ✅ | 2026-11-04 Windows 디버그 빌드 확인. Don Editor는 md 파일을 같은 탭의 미리보기 모드로 먼저 열고 툴바 `미리보기 \| 편집` 토글로 전환한다. 기존 별도 탭/옆 분할 미리보기도 유지. 2026-11-10: 미리보기 모드에서도 Ctrl+F(macOS Cmd+F)로 검색 바를 열고, 원본에서 찾은 일치를 미리보기 글자에 강조(자동 통합 테스트로 확인) |
| 뷰어 | SVG 미리보기 | `svg_preview` | ✅ | - | ✅ | 2026-11-05 확인: `.svg` 파일을 열면 편집기와 함께 "Preview pic.svg" 미리보기 탭이 열림 |
| 뷰어 | 이미지 뷰어 | `image_viewer` | ✅ | - | ✅ | 2026-11-05 확인: 160x100 PNG 를 열면 이미지와 크기·확대 정보가 표시됨 |
| 앱 | 명령줄(CLI) | `cli`, `install_cli` | ✅ | [command-line-interface](GramEditor-docs/docs/command-line-interface.md) | ⬜ | `cli` 실행 파일은 기본 빌드(`cargo build`) 대상이 아니라 `target\debug` 에 만들어지지 않음(확인함). 배포 zip 에 넣으려면 `-p cli` 별도 빌드 필요(Phase 3) |
| 앱 | 앱 내 도움말(문서 뷰어) | `docs` | ✅ | [index](GramEditor-docs/docs/index.md) | ✅ | 원본 문서를 `GramEditor-docs/docs` 에서 내장(`assets.rs` 경로 수정함). 표시 확인 필요; 2026-11-05 확인: `gram: open docs`로 한국어 `Don Editor 사용 안내` 표시(원본 도움말 열기·검색은 미확인) |
| 앱 | 온보딩 | `onboarding` | ✅ | - | ⬜ |  |
| 앱 | 제목 표시줄·알림·진행 표시 | `title_bar`, `notifications`, `activity_indicator` | ✅ | - | ⬜ |  |
| 앱 | 충돌 보고 | `crashes` | ✅ | [troubleshooting](GramEditor-docs/docs/troubleshooting.md) | ⬜ |  |
| 앱 | 자격 증명 입력(askpass) | `askpass` | ✅ | - | ⬜ | SSH 접속 시 사용 |
| 개발용 | 인스펙터·프로파일러 | `inspector_ui`, `miniprofiler_ui` | ✅ | - | ⬜ | 개발자용 |
| 개발용 | 컴포넌트 미리보기 | `component` | ✅ | - | ⬜ | 개발자용 |

실행 확인 22개 / 미확인 28개. (2026-11-05 Markdown 미리보기와 스모크 점검 추가)

## 3. 기준선 메뉴 구조 (Windows, 2026-10-27 조사)

`crates/gram/src/gram/app_menus.rs` 를 분석한 당시의 근사값이다 (다른 OS 전용 4개와 Dev 채널 전용 1개 제외, 항목 96개). 이후 추가한 Agent 패널·MCP 메뉴 및 Don Editor 사용 안내는 이 기준선 집계에 포함하지 않았으며 7~8절에서 별도로 관리한다. 모든 메뉴 항목은 기능 점검 시 체크리스트로 쓴다.

| 메뉴 | 항목 수 | 항목 |
|------|---------|------|
| View | 13 | Zoom In, Zoom Out, Reset Zoom, Reset All Zoom, Toggle Left Dock, Toggle Right Dock, Toggle Bottom Dock, Toggle All Docks, Project Panel, Outline Panel, Terminal Panel, Debugger Panel, Diagnostics |
| View ▸ Editor Layout | 4 | Split Up, Split Down, Split Left, Split Right |
| Don Editor | 3 | About Don Editor, Extensions, Quit Don Editor |
| Don Editor ▸ Settings | 11 | Open Settings, Open Settings File, Open Project Settings, Open Project Settings File, Open Default Settings, Language Servers, Open Keymap, Open Keymap File, Open Default Key Bindings, Select Theme..., Select Icon Theme... |
| File | 13 | New, New Window, Open File..., Open Folder..., Open Recent..., Open Remote..., Add Folder to Project…, Save, Save As…, Save All, Close Editor, Close Project, Close Window |
| Edit | 9 | Undo, Redo, Cut, Copy, Copy and Trim, Paste, Find, Find in Project, Toggle Line Comment |
| Selection | 13 | Select All, Expand Selection, Shrink Selection, Select Next Sibling, Select Previous Sibling, Add Cursor Above, Add Cursor Below, Select Next Occurrence, Select Previous Occurrence, Select All Occurrences, Move Line Up, Move Line Down, Duplicate Selection |
| Go | 13 | Back, Forward, Command Palette..., Go to File..., Go to Symbol in Project, Go to Symbol in Editor..., Go to Line/Column..., Go to Definition, Go to Declaration, Go to Type Definition, Find All References, Next Problem, Previous Problem |
| Run | 11 | Spawn Task, Start Debugger, Edit tasks.jsonc..., Edit debug.jsonc..., Continue, Step Over, Step Into, Step Out, Toggle Breakpoint, Edit Breakpoint, Clear All Breakpoints |
| Window | 2 | Minimize, Zoom |
| Help | 4 | License, Show Welcome, Documentation, Gram Repository |

## 4. `gram` 의존 트리에 포함되지 않는 crate

`crates/` 의 135개 중 8개는 `gram` 바이너리에 링크되지 않는다. 별도 바이너리·도구·벤치마크·테스트 지원용이며, 기능 누락이 아니다.

`explorer_command_injector`, `extension_api`, `extension_cli`, `fs_benchmarks`, `project_benchmarks`, `remote_server`, `schema_generator`, `theme_importer`

## 5. 수동 점검 체크리스트 (스모크 테스트)

각 Phase 종료 시 이 목록으로 기존 기능이 유지되는지 확인한다. 확인하면 `[x]` 로 바꾼다.

- [x] 앱 실행·창 표시 (2026-10-27)
- [x] 파일 열기·편집·저장 (2026-10-27, 사용자)
- [x] 한글 입력(IME) (2026-10-27, 사용자)
- [ ] 폴더 열기 → 프로젝트 패널에 트리 표시
- [ ] 파일 찾기, 명령 팔레트 열기
- [ ] 버퍼 내 검색, 프로젝트 전체 검색
- [ ] 문법 강조 (예: `.rs`, `.py`, `.json`, `.md` 파일)
- [ ] 터미널 패널 열기, 명령 실행
- [ ] Git 저장소에서 Git 패널에 변경 파일 표시
- [ ] 설정 파일 열기, 테마 변경
- [ ] 키맵 편집기 열기, Vim 모드 전환
- [ ] 확장 기능 화면 열기
- [ ] Help 메뉴에서 앱 내 도움말 열기 (내장 문서 표시 확인)
- [x] Markdown 미리보기 (2026-11-04, 같은 탭 미리보기/편집 전환·파일 연결로 열기 포함)
- [ ] 디버거 패널 열기
- [x] 원본 Gram 과 동시 실행, 설정·데이터 폴더 분리 (2026-10-27, Phase 2 검증)
- [ ] D2Coding 글꼴 표시(한글 포함), 메뉴 이름, 아이콘 (Phase 2, 사용자 시각 확인 대기)

## 6. 원본 자체 이슈 후보 (우리 변경과 무관)

- `ini`·`kdl` 언어: 설정·쿼리 폴더는 있으나 `register_native_grammars` 에 문법이 등록되어 있지 않다. 실제 강조 여부를 확인해 "원본에서도 안 됨"이면 여기에 기록한다.
- 창을 닫아도 `logs/Gram.log` 가 생기지 않았다 (콘솔 없이 실행한 경우). 원본 동작인지 확인 필요.

## 7. Don Editor 요구사항별 상태 (2026-11-04)

기존 요구사항 번호 R1~R8은 유지하고, Linux/macOS 배포 요구사항을 R9로 추가했다. 다음 상태는 원본 기능 50개 영역의 실행 확인 수와 별개다.

| 요구사항 | 확인할 기능 | 현재 상태·근거 | 남은 확인 |
|----------|-------------|----------------|-----------|
| R1 한국어 UI | 메뉴·패널·대화상자·설정 등의 한국어 표시와 언어 전환 | **후속 구현 및 순수 카탈로그 검증 완료(`7a48048`).** 영/한 1,762키, 설정 제목·설명·섹션 등 704개 문구와 키맵·Git·디버거·REPL·언어 서버·원격 서버·확장·작업·이미지·툴체인 등의 UI 한글화. 실행 중 언어 변경 시 설정 페이지·검색 인덱스 재생성. i18n 테스트 11개 통과 | 신규 UI 통합 빌드·실행, 설정·대화상자·툴팁의 잘림과 실행 중 언어 전환 확인. 추가한 설정 경로·한국어 Git·커서 단위 회귀 테스트는 GUI 의존성 빌드 완료 후 실행 필요. 외부 서버 메시지·사용자 데이터·지속 식별자·개발용 미리보기는 번역 범위에서 제외 |
| R2 문법 강조 | 파일 확장자로 26개 언어 감지·기본 강조. JavaScript의 mjs/cjs, yml/yaml, xhtml, `java.util.Properties` 호환 properties 포함 | **구현·자동 검증 완료.** 전체 언어·확장자·한계는 PROGRESS.md 2.1절 및 Phase 4 참고 | 실제 화면의 색상 확인, XHTML XML 선언 처리 등 알려진 한계 검토 |
| R3 로컬 MCP | 프로그램의 모든 기능 제어 | **구현 및 전송 계약 자동 검증 완료.** `don_tools` 공용 도구·범용 액션 실행과 `mcp_server` 루프백 HTTP·토큰 인증·`don-mcp` stdio 브릿지 추가. 기본·세션·HTTP 14개, stdio 4개, TCP 5개 및 선택 Windows ACL 1개 통과. 공식 SDK 2.3.0의 HTTP/stdio 협상·32개 도구·3개 리소스 검증 완료(고정 테스트 실행기) | 앱 통합 빌드·실행 및 실제 Claude Desktop/Claude Code 연결. 편집 승인·취소·오래된 변경안 거절, 검색·설정·터미널·모든 패널/액션의 GUI 동작 확인 |
| R4 Agent 사이드 패널 | OpenAI 또는 Claude API Key 연동, **기본 우측 배치·패널 숨김·화면 내 별도 아이콘 클릭으로 다시 표시** | **구현, 통합 검증 전.** `agent_panel`에 패널·토글·스트리밍 제공자·OS 자격 증명/환경 변수·컨텍스트 첨부·공용 도구·대화 기록 경로 추가. 동일 공급자 모듈의 모의 계약 테스트 9개 통과(`2b5390a`). 2026-11-04 사용자 요청으로 시작 시 숨김, 제공자 콤보박스, 기본 설정(설정 > 에이전트)의 제공자별 API 키·API 서버 URL, 키 없을 때 인라인 키 입력, 요청별 폴더·현재 파일 경로 자동 첨부로 변경하고 계약 테스트 13개 통과 | 실제 OpenAI/Claude API 호출, 자격 증명 저장·삭제, 채팅·중단·승인·대화 복원, 기본 우측 위치·숨김·아이콘 복귀의 실제 GUI 확인 |
| R5 Windows zip 배포 | 별도 bat/ps1 호출 시 빌드 도구 검사·미설치 도구의 설치 여부 질의 후 zip 생성 | **구현·스크립트 검사 완료.** `package-zip.bat`, `script/package-zip.ps1`에 MCP 실행 파일·소스 동봉 등 후속 배포 요구 반영(`f1ba957`). 기존 Phase 3의 zip 생성·압축 해제 후 실행 기록은 당시 소스 기준 | 이번 신규 기능을 포함한 최종 Windows zip 생성·압축 해제·실행. 깨끗한 Windows에서 실제 도구 설치·설치 후 재검사 및 ARM64 검증 |
| R6 D2Coding 기본 글꼴 | `assets/fonts`의 D2Coding을 기본 편집·UI 글꼴로 사용 | **구현 완료.** Phase 2 설정·TTC 로드 변경, Phase 5의 한글 렌더링 확인 기록 | D2Coding 적용 후 IME 입력 재확인 및 추가 시각 확인 |
| R7 GPL-3.0·원저작 고지 | 원본 라이선스·저작권 고지 유지, 배포본에 라이선스·소스 입수 방법 포함 | **고지·배포 소스 동봉 및 자동 검사 구현.** GPL-3.0·GramEditor/Zed·제3자·D2Coding 고지 유지, 플랫폼별 배포 스크립트에 해당 소스 스냅샷 포함. 내부 매니페스트 143개 검사와 검사기 회귀 테스트 10개 통과(`2d7df76`). 원본 도움말을 보존하고 구분 안내·한국어 Don Editor 도움말 추가(`5fa69a6`) | 최종 실제 zip/tar.gz에서 고지·소스 스냅샷과 실행 파일 버전의 일치 확인. 앱의 라이선스·도움말 화면 확인 |
| R8 기존 기능 보존 | 원본 GramEditor 기능 최대한 유지 | **지속 점검.** 1~6절의 원본 기능 목록·스모크 테스트 및 Phase별 회귀 기록 | 미확인 28개 영역은 확인되는 대로 갱신 |
| R9 Linux/macOS tar.gz 배포 | 별도 sh 호출 시 OS별 빌드 도구 검사·미설치 도구의 설치 여부 질의 후 tar.gz 생성 | **구현·모의 환경 자동 검증 완료(`f1ba957`).** `package-tar.sh`에 OS 감지·도구 선택 설치·재검사·릴리스 빌드·스테이징·소스/고지 동봉·tar.gz/SHA256 생성 추가. 패키징 테스트 14개 및 기존 도구 설치 테스트 19개 통과. Linux/macOS 명령은 가짜 도구로 검증 | 실제 Linux와 macOS 각각에서 필요한 도구 설치·릴리스 빌드·최종 tar.gz 생성·압축 해제·GUI 실행. 모의 스테이징 결과를 실제 OS 실행 확인으로 간주하지 않음 |

## 8. 추가·변경 요구사항 확인 목록

구현·자동 검증과 실제 환경 확인을 나누어 기록한다. `[x]`의 범위는 각 문장에 적힌 구현 또는 테스트에 한정하며, 2절의 미확인 28개 영역을 대신하지 않는다.

### 구현 및 자동 검증

- [x] R1: 설정 화면 본문과 후속 UI 한글화, 설정 페이지·검색 인덱스의 언어 변경 대응 구현(`7a48048`).
- [x] R1: GUI 의존성을 제외한 i18n 테스트 11개 통과. 영/한 1,762키의 키·자리표시자 정합성과 사용/미사용 키 검사 포함.
- [x] R3: 공용 도구 레지스트리, 범용 액션·편집·검색·설정·터미널·패널 제어 경로와 승인 모달 구현.
- [x] R3: MCP 계약 테스트 23개 및 별도 Windows ACL 1개 통과, 공식 SDK HTTP/stdio 초기화·자동 협상 검증 완료. 앱 UI를 대신하는 테스트 실행기를 사용한 전송 검증이다.
- [x] R4: 기본 우측 Agent 패널·숨김·도크 아이콘 복귀·API 제공자·키 저장·공용 도구·대화 기록 경로 구현. 실제 GUI/API 동작을 확인했다는 뜻은 아니다.
- [x] R4: 실제 공급자 모듈의 모의 스트리밍·도구 호출·오류·서명 블록 보존 계약 테스트 9개 통과(`2b5390a`).
- [x] R5: Windows zip 스크립트에 신규 실행 파일·고지·소스 동봉을 반영하고 스크립트 검사 통과(`f1ba957`).
- [x] R9: 별도 `package-tar.sh` 구현 및 Linux/macOS를 모의한 패키징 테스트 14개 통과. 도구 준비/누락·설치 동의/거절·실패·스테이징·압축·해시 흐름 포함.
- [x] R5·R9: 기존 도구 설치 스크립트 테스트 19개 통과. 도구 설치 테스트와 배포 압축 파일 테스트는 별도로 수행했다.
- [x] R7: 내부 매니페스트 143개 라이선스 검사 및 검사기 회귀 테스트 10개 통과(`2d7df76`).
- [x] R1·R7·R8: 한국어 Don Editor 도움말 추가, 원본 도움말 보존과 원본 문서 구분 안내 구현(`5fa69a6`).
- [x] 추가 기능(2026-11-05, 사용자 요청): Markdown 미리보기/편집 전환(같은 탭, 열면 미리보기, `475437a`), 상태 표시줄 인코딩 표시·다른 인코딩으로 다시 불러오기·표현 불가 문자 저장 거절(`c29fe81`), Typst 쪽 미리보기·Markdown→Typst 변환·PDF 내보내기·인쇄(`4421531`). 단위 테스트: markdown_preview 40개, don_charset 9개, language 다시 불러오기 3개, worktree 저장 시험 3개, don_typst 17개, i18n 11개(1,824키) 통과.

### 실제 앱·API·OS 확인

- [x] 추가 기능(2026-11-05) Windows 디버그 빌드 실제 창 확인: Markdown·Typst 파일을 파일 연결(더블클릭과 같은 Shell 열기, 실행 중/아닐 때)로 열면 미리보기로 시작, 미리보기↔편집 전환·저장·분할·세션 복원, 인코딩 표시·다시 불러오기(취소/확인, 실행 취소 기록 비움, 디스크 무변경)·표현 불가 문자 저장 거절·섞인 인코딩 부분 복원, Typst 오류 표시, 변환 저장 창, 중간 파일 없는 PDF 내보내기. 세부는 PROGRESS.md.
- [ ] 추가 기능: 실제 프린터 출력 또는 PDF 뷰어가 `print` 동작을 지원하는 환경에서의 인쇄, Linux/macOS 인쇄(`lp`)·시스템 한글 글꼴·인코딩 선택 화면, 릴리스 빌드에서의 같은 흐름, (MCP `dispatch_action`으로 `typst::ConvertMarkdownToTypst`·`typst::ExportPdf`·`encoding_selector::ReloadWithEncoding` 실행은 2026-11-05 확인 완료. `typst::Print`·`markdown::TogglePreviewMode`·`typst::TogglePreviewMode`의 MCP 실행은 미확인.)

- [ ] R1~R9: 이번 전체 변경 소스의 앱 통합 빌드·실행 및 변경 GUI crate 테스트 완료. 전체 `cargo check`는 사용자 일시 중지 요청에 맞춰 공용 의존성 컴파일 단계에서 중단했다.
- [ ] R1: 설정·키맵·Git·디버거 등 후속 화면의 한국어 표시, 대화상자·툴팁의 잘림 및 실행 중 언어 전환을 실제 창에서 확인한다.
- [ ] R1·R8: 새 Don Editor 도움말과 원본 도움말의 검색·탐색·원본 구분 표시를 실제 창에서 확인한다.
- [ ] R3: 실제 MCP 클라이언트에서 편집 승인/거절과 조회·편집·검색·설정·터미널·범용 액션을 검증한다.

- [ ] R4: OpenAI 또는 Claude API Key 중 하나를 등록해 Agent 기능을 사용할 수 있다.
- [ ] R4: 최초 표시 시 Agent 패널이 기본적으로 우측에 배치된다.
- [ ] R4: 프로그램 실행 시 Agent 패널은 숨긴 상태로 시작한다(이전 실행에서 열어 두었어도 동일).
- [ ] R4: 제공자를 콤보박스로 선택하고, API 키·API 서버 URL을 기본 설정에서 제공자별로 입력한다(URL 공란이면 기본값).
- [ ] R4: 선택한 제공자의 키가 없으면 패널의 프롬프트 입력란 자리에 키 입력란이 나타나고, 입력 즉시 기본 설정에 반영된 뒤 프롬프트 입력란이 다시 나타난다.
- [ ] R4: Agent 패널을 숨길 수 있고, 숨긴 상태에서도 화면 내 별도 아이콘에 접근할 수 있다.
- [ ] R4: 해당 아이콘 클릭으로 패널이 다시 표시되며, 반복해서 숨김·표시할 수 있다.
- [ ] R3·R4·R8: Agent 패널 표시/숨김이 로컬 MCP 제어에 포함되고, 기존 도크·패널 동작도 유지된다.
- [ ] R5: 깨끗한 Windows에서 설치 동의·거절·EOF·설치 실패 경로를 확인하고, 도구 준비 후 zip 생성·압축 해제·실행을 확인한다.
- [ ] R9: Linux sh 배포 스크립트 호출 시 필요한 빌드 도구를 검사하고, 없으면 설치 여부를 묻는다.
- [ ] R9: macOS sh 배포 스크립트 호출 시 필요한 빌드 도구를 검사하고, 없으면 설치 여부를 묻는다.
- [ ] R9: Linux와 macOS 각각에서 도구 설치 동의·거절·EOF·설치 실패 및 설치 후 재검사 흐름을 확인한다.
- [ ] R9: Linux와 macOS 각각에서 tar.gz 생성·압축 해제·실행을 확인한다. 도구 설치 스크립트만 실행한 결과와 구분한다.
- [ ] R7·R5·R9: 최종 실제 Windows zip 및 Linux/macOS tar.gz에서 GPL-3.0 전문, GramEditor/Zed 원저작 고지, 제3자·D2Coding 라이선스와 해당 버전의 소스 스냅샷이 포함되는지 확인한다.
