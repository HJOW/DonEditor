# 원본 기능 인벤토리 및 추가 요구사항 (R1~R9)

> 작성: 2026-10-27 · 기준: GramEditor 3.4.0 (git 태그 `gram-baseline-3.4.0` = 원본 무수정) · 관련: [PROGRESS.md](PROGRESS.md) 3.10절
> 최종 갱신: 2026-11-04 · [AGENTS.md](AGENTS.md) 변경 반영: Agent 패널 숨김·별도 아이콘으로 다시 표시(R4), Windows zip(R5), Linux/macOS tar.gz(R9)

AGENTS.md 의 "기존에 존재하던 기능은 최대한 포함한다"(R8)를 지키기 위해, 원본 GramEditor 의 기능을 목록으로 만들고 **우리 변경으로 깨지지 않았는지** 확인하는 기준으로 쓴다.

1~6절은 원본 기능 보존 기준이며, Don Editor의 추가 요구사항과 확인 항목은 7~8절에서 관리한다. 이번 갱신은 문서 반영 작업으로, 새 빌드·앱 실행이나 배포본 검증을 수행한 결과가 아니다. 기존 50개 영역의 실행 확인 수는 변경하지 않는다.

## 읽는 법

- **빌드 포함**: `cargo tree -p gram --target x86_64-pc-windows-msvc` 의존 트리에 해당 crate 가 들어 있는지 조사한 **정적 결과**다. ✅ 전부 포함 / ⚠ 일부는 별도 바이너리 / ❓ 저장소에 없음.
- **실행 확인**: 앱을 실제로 실행해 동작을 본 결과다. ✅ 확인됨 / ⬜ 아직 확인하지 않음. **⬜ 은 "동작하지 않는다"가 아니라 "아직 모른다"는 뜻이다.**
- 이 표는 Windows 기준이다. 기능별 실행 확인은 사람이 직접 앱을 써 봐야 하는 것이 많아, 확인되는 대로 이 문서를 갱신한다.
- 7절의 구현·검증 상태는 [PROGRESS.md](PROGRESS.md)의 기존 기록과 현재 파일 구성을 기준으로 정리한다. Windows의 빌드·실행 결과를 Linux/macOS에서 확인한 결과로 간주하지 않는다.

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
| 편집 기본 | 다중 버퍼(Multibuffers) | `multi_buffer` | ✅ | [multibuffers](GramEditor-docs/docs/multibuffers.md) | ⬜ |  |
| 편집 기본 | SuperTab | `editor` | ✅ | [supertab](GramEditor-docs/docs/supertab.md) | ⬜ |  |
| 편집 기본 | 줄 이동 | `go_to_line` | ✅ | - | ⬜ |  |
| 편집 기본 | 줄 끝(개행)·인코딩 선택 | `line_ending_selector`, `encoding_selector` | ✅ | - | ⬜ |  |
| 편집 기본 | 스니펫 | `snippet`, `snippet_provider`, `snippets_ui` | ✅ | [snippets](GramEditor-docs/docs/snippets.md) | ⬜ |  |
| 탐색 | 탭 전환 | `tab_switcher` | ✅ | [tab-switcher](GramEditor-docs/docs/tab-switcher.md) | ⬜ |  |
| 탐색 | 명령 팔레트 | `command_palette`, `command_palette_hooks` | ✅ | [command-palette](GramEditor-docs/docs/command-palette.md), [all-actions](GramEditor-docs/docs/all-actions.md) | ⬜ |  |
| 탐색 | 파일 찾기 | `file_finder` | ✅ | - | ⬜ |  |
| 탐색 | 프로젝트 패널 | `project_panel` | ✅ | - | ⬜ |  |
| 탐색 | 개요(Outline) 패널·개요 | `outline_panel`, `outline` | ✅ | [outline-panel](GramEditor-docs/docs/outline-panel.md) | ⬜ |  |
| 탐색 | 심볼 검색 | `project_symbols` | ✅ | - | ⬜ |  |
| 탐색 | 검색·치환 (버퍼/프로젝트) | `search` | ✅ | - | ⬜ |  |
| 탐색 | 최근 프로젝트·파일 | `recent_projects`, `recent_files` | ✅ | - | ⬜ |  |
| 탐색 | Which-key 도움말 | `which_key` | ✅ | - | ⬜ |  |
| 탐색 | 일지(Journal) | `journal` | ✅ | - | ⬜ |  |
| 언어/코드 지능 | 언어 지원·문법 강조(tree-sitter) | `languages`, `language`, `language_extension` | ✅ | [languages](GramEditor-docs/docs/languages.md), [configuring-languages](GramEditor-docs/docs/configuring-languages.md) | ⬜ | 원본 내장 15개 언어(2.1) 중 일부는 Phase 4에서 확인. `ini`·`kdl` 은 문법 미등록(원본 자체 이슈 후보) |
| 언어/코드 지능 | 언어 선택기·언어 도구 | `language_selector`, `language_tools` | ✅ | - | ⬜ |  |
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
| 편집 모드 | Vim 모드 | `vim`, `vim_mode_setting` | ✅ | [vim](GramEditor-docs/docs/vim.md) | ⬜ |  |
| 편집 모드 | Helix 모드 | `vim` | ✅ | [helix](GramEditor-docs/docs/helix.md) | ⬜ |  |
| 설정/외형 | 설정 파일·설정 UI·설정 프로필 | `settings`, `settings_ui`, `settings_profile_selector` | ✅ | [configuring-gram](GramEditor-docs/docs/configuring-gram.md), [environment](GramEditor-docs/docs/environment.md) | ⬜ |  |
| 설정/외형 | 키맵·단축키 편집기 | `keymap_editor` | ✅ | [key-bindings](GramEditor-docs/docs/key-bindings.md) | ⬜ |  |
| 설정/외형 | 테마 | `theme`, `theme_selector`, `theme_importer`, `theme_extension` | ⚠ 일부 별도: theme_importer | [themes](GramEditor-docs/docs/themes.md) | ⬜ |  |
| 설정/외형 | 아이콘 테마·파일 아이콘 | `file_icons` | ✅ | [icon-themes](GramEditor-docs/docs/icon-themes.md) | ⬜ |  |
| 설정/외형 | 시각 사용자화 | `ui` | ✅ | [visual-customization](GramEditor-docs/docs/visual-customization.md) | ⬜ |  |
| 확장/연동 | 확장 기능(Wasm) 설치·관리 | `extension`, `extension_host`, `extension_api`, `extensions_ui` | ⚠ 일부 별도: extension_api | [extensions](GramEditor-docs/docs/extensions.md) | ⬜ | 원본 README: 제한적 지원. 확장이 임의 실행 파일을 내려받을 수 있다는 경고 있음 |
| 확장/연동 | Zed 설정 가져오기(마이그레이션) | `migrator` | ✅ | [zed](GramEditor-docs/docs/migrate/zed.md), [vs-code](GramEditor-docs/docs/migrate/vs-code.md) | ⬜ |  |
| 뷰어 | Markdown 미리보기 | `markdown_preview`, `markdown` | ✅ | - | ⬜ |  |
| 뷰어 | SVG 미리보기 | `svg_preview` | ✅ | - | ⬜ |  |
| 뷰어 | 이미지 뷰어 | `image_viewer` | ✅ | - | ⬜ |  |
| 앱 | 명령줄(CLI) | `cli`, `install_cli` | ✅ | [command-line-interface](GramEditor-docs/docs/command-line-interface.md) | ⬜ | `cli` 실행 파일은 기본 빌드(`cargo build`) 대상이 아니라 `target\debug` 에 만들어지지 않음(확인함). 배포 zip 에 넣으려면 `-p cli` 별도 빌드 필요(Phase 3) |
| 앱 | 앱 내 도움말(문서 뷰어) | `docs` | ✅ | [index](GramEditor-docs/docs/index.md) | ⬜ | 원본 문서를 `GramEditor-docs/docs` 에서 내장(`assets.rs` 경로 수정함). 표시 확인 필요 |
| 앱 | 온보딩 | `onboarding` | ✅ | - | ⬜ |  |
| 앱 | 제목 표시줄·알림·진행 표시 | `title_bar`, `notifications`, `activity_indicator` | ✅ | - | ⬜ |  |
| 앱 | 충돌 보고 | `crashes` | ✅ | [troubleshooting](GramEditor-docs/docs/troubleshooting.md) | ⬜ |  |
| 앱 | 자격 증명 입력(askpass) | `askpass` | ✅ | - | ⬜ | SSH 접속 시 사용 |
| 개발용 | 인스펙터·프로파일러 | `inspector_ui`, `miniprofiler_ui` | ✅ | - | ⬜ | 개발자용 |
| 개발용 | 컴포넌트 미리보기 | `component` | ✅ | - | ⬜ | 개발자용 |

실행 확인 2개 / 미확인 48개.

## 3. 메뉴 구조 (Windows 에서 보이는 항목)

`crates/gram/src/gram/app_menus.rs` 를 분석한 근사값이다 (다른 OS 전용 4개와 Dev 채널 전용 1개 제외, 항목 96개). 모든 메뉴 항목은 기능 점검 시 체크리스트로 쓴다.

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
- [ ] Markdown 미리보기
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
| R1 한국어 UI | 메뉴·패널·대화상자·설정 등의 한국어 표시와 언어 전환 | **1차 구현 완료.** `crates/i18n`, `assets/locales` 및 Phase 5 자동 테스트·일부 실제 화면 확인 기록 | 설정 화면 본문·키맵·Git·디버거 등 추가 번역, 대화상자·실행 중 언어 전환 등의 화면 확인 |
| R2 문법 강조 | 파일 확장자로 26개 언어 감지·기본 강조. JavaScript의 mjs/cjs, yml/yaml, xhtml, `java.util.Properties` 호환 properties 포함 | **구현·자동 검증 완료.** 전체 언어·확장자·한계는 PROGRESS.md 2.1절 및 Phase 4 참고 | 실제 화면의 색상 확인, XHTML XML 선언 처리 등 알려진 한계 검토 |
| R3 로컬 MCP | 프로그램의 모든 기능 제어 | **미구현.** 공용 도구 레지스트리·로컬 MCP 서버·액션 실행 경로는 Phase 6 계획 | 외부 MCP 클라이언트 연결, 편집·검색·설정·터미널·패널 표시/숨김 등 전체 기능 제어 검증 |
| R4 Agent 사이드 패널 | OpenAI 또는 Claude API Key 연동, **기본 우측 배치·패널 숨김·화면 내 별도 아이콘 클릭으로 다시 표시** | **미구현.** Phase 7 계획에 숨김·아이콘 복귀 요구사항 추가 | 키 연동·대화·도구 호출·숨김·아이콘 접근 및 다시 표시 흐름 검증 |
| R5 Windows zip 배포 | 별도 bat/ps1 호출 시 빌드 도구 검사·미설치 도구의 설치 여부 질의 후 zip 생성 | **구현·검증 기록 있음.** `package-zip.bat`, `script/package-zip.ps1`, Phase 3의 zip 생성·압축 해제 후 실행 기록 | 깨끗한 Windows에서 실제 도구 설치·설치 후 재검사 및 ARM64 검증 |
| R6 D2Coding 기본 글꼴 | `assets/fonts`의 D2Coding을 기본 편집·UI 글꼴로 사용 | **구현 완료.** Phase 2 설정·TTC 로드 변경, Phase 5의 한글 렌더링 확인 기록 | D2Coding 적용 후 IME 입력 재확인 및 추가 시각 확인 |
| R7 GPL-3.0·원저작 고지 | 원본 라이선스·저작권 고지 유지, 배포본에 라이선스·소스 입수 방법 포함 | **고지 적용, 배포 전 확인 남음.** `LICENSE`, `assets/licenses.md`, 제3자·D2Coding 고지 및 Windows zip 포함 기록 | Don Editor 소스 제공 위치 확정, Linux/macOS tar.gz에도 동일 고지 포함 |
| R8 기존 기능 보존 | 원본 GramEditor 기능 최대한 유지 | **지속 점검.** 1~6절의 원본 기능 목록·스모크 테스트 및 Phase별 회귀 기록 | 미확인 48개 영역은 확인되는 대로 갱신 |
| R9 Linux/macOS tar.gz 배포 | 별도 sh 호출 시 OS별 빌드 도구 검사·미설치 도구의 설치 여부 질의 후 tar.gz 생성 | **미구현.** `package-tar.sh` 예정, PROGRESS.md의 Phase 3 후속 참고. `install-build-tools.sh`는 도구 설치만 수행 | Linux와 macOS 각각에서 설치 동의/거절·빌드·tar.gz 생성·압축 해제 후 실행 검증 |

## 8. 추가·변경 요구사항 확인 목록

아래 항목은 앞으로 수행할 검증이다. R5에 기존 검증 기록이 있어도, 이번 문서 수정만으로 새 항목을 완료 처리하지 않는다.

- [ ] R4: OpenAI 또는 Claude API Key 중 하나를 등록해 Agent 기능을 사용할 수 있다.
- [ ] R4: 최초 표시 시 Agent 패널이 기본적으로 우측에 배치된다.
- [ ] R4: Agent 패널을 숨길 수 있고, 숨긴 상태에서도 화면 내 별도 아이콘에 접근할 수 있다.
- [ ] R4: 해당 아이콘 클릭으로 패널이 다시 표시되며, 반복해서 숨김·표시할 수 있다.
- [ ] R3·R4·R8: Agent 패널 표시/숨김이 로컬 MCP 제어에 포함되고, 기존 도크·패널 동작도 유지된다.
- [ ] R5: Windows bat/ps1 배포 스크립트 호출 시 필요한 빌드 도구를 검사하고, 없으면 설치 여부를 묻는다.
- [ ] R5: 깨끗한 Windows에서 설치 동의·거절·EOF·설치 실패 경로를 확인하고, 도구 준비 후 zip 생성·압축 해제·실행을 확인한다.
- [ ] R9: Linux sh 배포 스크립트 호출 시 필요한 빌드 도구를 검사하고, 없으면 설치 여부를 묻는다.
- [ ] R9: macOS sh 배포 스크립트 호출 시 필요한 빌드 도구를 검사하고, 없으면 설치 여부를 묻는다.
- [ ] R9: Linux와 macOS 각각에서 도구 설치 동의·거절·EOF·설치 실패 및 설치 후 재검사 흐름을 확인한다.
- [ ] R9: Linux와 macOS 각각에서 tar.gz 생성·압축 해제·실행을 확인한다. 도구 설치 스크립트만 실행한 결과와 구분한다.
- [ ] R7·R5·R9: Windows zip 및 Linux/macOS tar.gz 모두 GPL-3.0 전문, GramEditor/Zed 원저작 고지, 제3자·D2Coding 라이선스와 소스 입수 방법을 포함한다.
