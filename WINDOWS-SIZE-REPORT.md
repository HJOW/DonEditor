# Windows 실행 파일 기능별 용량 점유 보고서

작성일: 2026-11-04  
대상: Don Editor 3.4.0, Windows x86_64 릴리스 배포본  
대상 파일: `dist/windows-20261104-172637/extracted/DonEditor-3.4.0-windows-x86_64/doneditor.exe`  
기준 소스: 배포 빌드 당시 커밋 `2c987bf`  
단위: MiB = 1,048,576바이트

## 결론

실행 파일은 **165.59 MiB**(173,628,416바이트)다. 실제 링크 결과에서 **내장 Tree-sitter 문법 분석기 객체들이 46.12 MiB**를 차지한다. 그중 SQL 분석기(`tree-sitter-sequel`)만 **10.61 MiB**로, 확인된 개별 구성 요소 중 가장 크다. C++ 5.30 MiB, C# 5.07 MiB, Swift 3.94 MiB, Scala 3.83 MiB가 뒤따른다.

다만 이 다섯 언어는 모두 `AGENTS.md`의 필수 지원 대상이다. 기본 배포판에서 해당 문법을 제거하면 요구사항을 충족하지 못한다. **크기가 곧 선택 제거 가능 용량은 아니다.** 필수 언어는 문법 최적화 또는 별도 설치 방식의 가능성을 기능·사용성 검증과 함께 조사해야 한다.

확장 기능은 직접 확인된 `extension_host` 등 3.86 MiB와 Wasmtime·Cranelift·Wasm 관련 이름의 객체 8.94 MiB가 유력한 분석 대상이다. 둘은 이름 기준으로 겹치지 않는 집계지만, 다른 기능이 공유하는 코드와 관련 의존성은 포함하지 않았고 일부 코드가 다른 객체에 합쳐질 수 있으므로 **확장을 제거하면 12.80 MiB가 줄어든다는 뜻은 아니다.**

## 측정 방식과 한계

- 배포 EXE와 `target/release/doneditor.exe`의 SHA-256이 일치함을 확인했다. 같은 디렉터리의 `doneditor.pdb`는 Microsoft DIA의 실행 파일 로딩 경로로 일치 확인 후 사용했다.
- Visual Studio의 DIA2Dump 샘플을 `target/dia-report-tool/`에 복사해 로컬 빌드하고, 시스템 COM 등록 없이 DIA DLL을 직접 불러오게 테스트용 복사본만 조정했다. `dia2dump -c`로 PDB의 섹션 기여 항목 398,349개를 추출했다. 원본 EXE·PDB·배포본과 프로젝트 소스는 변경하지 않았다.
- 각 항목의 16진수 크기를 합산했다. Rust 객체의 crate 이름과 `target/release/build/<패키지>-<해시>/out/` 경로를 이용했다. 동일한 `parser.o` 파일명은 **전체 빌드 경로**로 구분했다. PDB 집계 원본은 `target/dia-report-tool/contributions.txt`에 있다(빌드 캐시는 영구 보관을 보장하지 않는다).
- 기여 항목 합계는 **170,341,756바이트**다. PE 파일 크기와의 차이는 PDB 집계에 없는 섹션·정렬·헤더 및 가상/실제 파일 크기 차이 때문이다. 아래 기능별 값은 선택한 이름과 경로에 귀속된 **직접 기여량**이며 공유 의존성, 인라이닝, Thin LTO, 링커 중복 제거로 인해 기능을 끄고 다시 빌드했을 때의 실제 감소량과 다르다. 일반적으로 기능별 항목은 상호 의존하므로 무조건 더하거나 절감량의 하한·상한으로 쓰지 않는다.
- 각 수치는 해당 빌드에서 측정한 값으로, 다른 빌드·버전·플랫폼에 그대로 적용할 수 없다. `.rlib` 파일 크기는 최종 EXE 기여량으로 사용하지 않았다.

## EXE 영역별 실측

| 영역 | 디스크 크기 | 의미 |
|---|---:|---|
| `.text` | 102.87 MiB | 실행 코드 |
| `.rdata` | 60.52 MiB | 읽기 전용 데이터·상수·압축 내장 리소스 등 |
| `.pdata` | 1.52 MiB | 예외 처리 관련 정보 |
| `.reloc` | 0.46 MiB | 재배치 정보 |
| `.data` | 0.19 MiB | 쓰기 가능한 초기화 데이터 |
| 기타 | 약 0.03 MiB | 리소스·헤더·정렬 등 |

문법 분석기 객체 46.12 MiB 중 약 **43.77 MiB가 `.rdata`**, 약 2.27 MiB가 `.text`에 해당한다. 따라서 `.rdata`의 큰 부분은 단순히 글꼴이나 이미지가 아니라 파서 테이블이다.

## 기능 관련 구성 요소별 직접 기여량

같은 기능의 모든 의존성이 포함된 값은 아니다. 괄호 안의 이름만 합산했으며 각 줄의 값은 실제 기능 제거 시 절감량이 아니다.

| 기능 또는 구성 | 직접 기여량 | 집계 범위 / 해석 |
|---|---:|---|
| 문법 분석기 | 46.12 MiB | 빌드 경로에 `tree-sitter-`가 포함된 생성 파서·스캐너 객체. 아래 언어별 표 참고 |
| 확장 실행 런타임 후보 | 8.94 MiB | 이름이 `wasmtime`, `cranelift`, `wasmparser`, `wasmprinter`로 시작하는 객체. 공유 가능 |
| 프로젝트 기반 | 11.11 MiB | `project`; 여러 기능이 공유하는 핵심 기반 |
| 내장 리소스 | 7.10 MiB | `assets` 객체 전체. 글꼴만의 크기가 아니며 코드·문서·테마 등도 포함 |
| 편집기 | 6.60 MiB | `editor`; 핵심 기능 |
| 설정 및 키맵 | 6.10 MiB | `settings`, `settings_ui`, `keymap_editor`, `settings_profile_selector` |
| 언어 기반 UI·연동 | 4.97 MiB | `language`, `languages`, `language_tools`, `language_selector`; 위 파서 객체와 별도 |
| Git | 4.49 MiB | `git`, `git_ui`, `git_graph`, `git_hosting_providers` |
| 디버거 | 4.03 MiB | `debugger_ui`, `debugger_tools`, `dap`, `dap_adapters`, `debug_adapter_extension` |
| 확장 기능 관리·호스트 | 3.86 MiB | `extension_host`, `extension`, `extensions_ui`, `extension_api`; 위 런타임 후보와 이름상 별도 |
| 검색·파일 찾기 | 2.89 MiB | `search`, `file_finder`, `project_symbols` |
| Vim 모드 | 2.89 MiB | `vim` |
| Markdown 내 Mermaid 렌더러 | 2.18 MiB | `mermaid_rs_renderer`; Markdown 전체 크기가 아님 |
| REPL | 1.95 MiB | `repl` |
| MCP 서버와 공용 도구 | 1.71 MiB | `mcp_server`, `don_tools`; `don_tools`는 Agent와 공유. 별도 `don-mcp.exe`는 이 표에서 제외 |
| 터미널 | 1.58 MiB | `terminal`, `terminal_view`; 쉘·공통 의존성 제외 |
| Agent 패널 | 0.67 MiB | `agent_panel`만. 공용 도구·HTTP·자격 증명 등 공유 의존성 제외 |

추가로 `workspace` 2.47 MiB, `gpui` 2.13 MiB, `ui` 1.10 MiB 등은 여러 기능에서 쓰는 공통 기반이다. 특정 패널을 끈다고 전부 사라지지 않는다.

## 내장 문법별 직접 기여량

크기는 PDB의 해당 빌드 패키지 경로에 귀속된 파서·스캐너 등 기여 항목 합계다. 문법 쿼리, Rust 연결 코드, 공통 Tree-sitter 런타임은 별도 객체에 있을 수 있다. 필수 여부는 `AGENTS.md`의 목록 기준이며, 미표시 항목도 기존 기능 보존 요구사항과 사용자 설정을 검토해야 한다.

| 문법 패키지 | MiB | 요구사항 관련 |
|---|---:|---|
| SQL (`tree-sitter-sequel`) | 10.606 | 필수 |
| C++ | 5.301 | 필수 |
| C# | 5.066 | 필수 |
| Swift | 3.937 | 필수 |
| Scala | 3.827 | 필수 |
| TypeScript / TSX | 2.736 | TypeScript 필수; JavaScript 관련 지원과 공유 여부 확인 필요 |
| Odin | 2.116 | 필수 목록 외 |
| Ruby | 1.996 | 필수 |
| Dart | 1.382 | 필수 |
| Bash | 1.268 | Unix Shell 필수 |
| Rust | 1.057 | 필수 |
| PowerShell | 0.886 | 필수 |
| Typst | 0.722 | 필수 목록 외 |
| Markdown | 0.671 | 필수 |
| Zig | 0.657 | 필수 목록 외 |
| C | 0.580 | 필수 |
| R | 0.459 | 필수 |
| Python | 0.433 | 필수 |
| Erlang | 0.405 | 필수 목록 외 |
| Java | 0.389 | 필수 |
| Gleam | 0.380 | 필수 목록 외 |
| Go | 0.203 | 필수 |
| YAML | 0.176 | 필수 |
| Windows Batch | 0.149 | 필수 |
| CSS | 0.107 | HTML 관련 지원 여부 검토 |
| Scheme | 0.088 | 필수 목록 외 |
| HCL | 0.087 | 필수 목록 외 |
| Nix | 0.070 | 필수 목록 외 |
| Roto | 0.059 | 필수 목록 외 |
| Git commit | 0.053 | Git 관련 사용 여부 검토 |
| Lua | 0.048 | 필수 목록 외 |
| XML | 0.042 | XML·XHTML 필수 |
| TOML | 0.021 | 필수 목록 외 |
| JSDoc | 0.021 | JavaScript 관련 지원 여부 검토 |
| 정규식 | 0.020 | 공통 기능과 공유 여부 검토 |
| Go mod / Go work | 0.018 / 0.010 | Go 관련 지원 여부 검토 |
| HTML | 0.018 | 필수 |
| JSON5 | 0.018 | 필수 |
| diff | 0.018 | Git·diff 관련 사용 여부 검토 |
| properties | 0.009 | 필수 |
| JSON | 0.005 | 필수 |
| Git rebase | 0.004 | Git 관련 사용 여부 검토 |

JavaScript는 이 집계에 독립된 `tree-sitter-javascript` 빌드 객체로 나타나지 않았다. TSX 문법을 이용하는 코드가 있으므로 별도의 JavaScript 크기나 제거 효과를 추정하지 않는다.

## 선택 분리를 위한 우선 검토

1. **확장 런타임:** 직접 기여 이름 기준 12.80 MiB 규모가 보인다. Wasmtime이 다른 기능에서도 사용되는지 의존 관계를 확인하고, 확장 미포함 실험 빌드를 만들어 실제 EXE·ZIP 크기와 기능 차이를 비교한다. 확장 설치·실행이 중단될 수 있다.
2. **필수 목록 밖의 문법:** Odin 2.12 MiB, Typst 0.72 MiB, Zig 0.66 MiB 등이 후보지만 기존 문법 지원은 줄어든다. 온디맨드 문법 제공 또는 별도 배포의 설치·오프라인 사용성을 먼저 정한다. 삭제 전 필요한 언어·공유 파서 여부를 확인한다.
3. **디버거·Git·REPL·Vim·터미널·Mermaid:** 위 표의 직접 기여량은 분리 후보의 비교 출발점이다. 각각 기존 기능을 줄이며 공통 의존성이 많으므로 실제 감소량은 개별 실험 빌드로 확인한다. 필수 MCP의 전 기능 제어 및 Agent 공용 도구에도 영향을 검사한다.
4. **필수 문법의 파서 크기:** SQL·C++·C#·Swift·Scala가 크지만 제거 대신 다른 호환 문법, 생성 옵션, 선택적 다운로드 등을 평가한다. 확장자 감지·문법 강조·오프라인 동작·라이선스 유지가 필요하다.
5. **컴파일 최적화:** 현재 릴리스는 `opt-level = 3`, `lto = "thin"`, `codegen-units = 4`다. 별도 타깃 디렉터리에서 `opt-level = "s"`/`"z"`와 LTO 조합을 비교해 크기·빌드 시간·실행 성능을 측정한다. 기본 배포 설정은 실험 결과와 회귀 테스트 없이 바꾸지 않는다.

내장 글꼴의 원본 파일 합계는 약 11.41 MiB이며 D2Coding 원본은 8.11 MiB다. `rust-embed`의 압축 기능이 켜져 있어 원본 파일 크기를 EXE 내 점유량으로 간주할 수 없다. 글꼴을 외부 파일로 옮겨도 배포 폴더 전체가 줄어드는 것은 아니다. D2Coding 기본 글꼴은 필수 요구사항이다.

## 권장 후속 실험

기존 배포본을 보존한 채 **기준 빌드와 기능을 하나씩 뺀 실험 빌드**를 동일한 Rust·프로필·환경으로 각각 별도 디렉터리에 만든다. EXE와 ZIP 크기, 기능 및 회귀 테스트 결과를 함께 기록한다. `기준 EXE 크기 − 실험 EXE 크기`가 해당 조합에서 관찰된 실제 절감량이며, 복수 기능을 함께 뺀 결과는 공유 코드 때문에 단순 합과 다를 수 있다. 이번 보고서 작성에서는 실험 빌드나 프로그램 수정은 수행하지 않았다.
