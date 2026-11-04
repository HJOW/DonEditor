# DonEditor

DonEditor 는 [GramEditor](https://codeberg.org/GramEditor/gram) 를 기반으로 만드는 **한국어 텍스트 편집기**입니다.    
개발에 활용할 수 있는 서브 텍스트 편집기를 목표로 합니다.
기존 GramEditor 에서 일부 한글 표현에 문제가 있어 Folk 하여 기능을 보완하고 있습니다.
    
포크 대상 프로젝트 :  [GramEditor](./GramEditor-docs/README.md)
    
GramEditor 은 AI를 사용하지 않고 개발된 프로젝트이나, 이 DonEditor 는 작업에 AI가 활용되었습니다.
기존 GramEditor 의 목표 일부는 이 프로젝트에서는 해당되지 않습니다.
     
라이선스는 GramEditor 와 동일한 GPLv3 을 채택합니다.    
[LICENSE](LICENSE)    
[GramEditor License](./GramEditor-docs/LICENSE)    
    
> [!WARNING]
> **현재 개발 초기 단계입니다.** 아래 "목표 기능"은 대부분 아직 구현되지 않았습니다.
> 단계별 진행 상황은 [PROGRESS.md](./PROGRESS.md) 에서 확인할 수 있습니다.

## 이름의 유래

GramEditor 은 가벼움을 모토로 개발된 텍스트 편집기 프로젝트입니다. 이름에 들어간 Gram 에서 이를 알 수 있습니다.
이는 이 프로젝트 또한 마찬가지로, 한국어의 "돈" 단위에서 이름을 따왔습니다.

## 목표 기능

| 기능 | 내용 | 상태 |
|------|------|------|
| **AI Agent 사이드 패널** | OpenAI 또는 Claude API Key 를 연동하는 Agent 패널 (기본 위치: 우측). 대화, 현재 파일·선택 영역 참조, 편집 작업 위임 | 예정 |
| **로컬 MCP 서버** | 외부 MCP 클라이언트(AI 도구)에서 편집기의 모든 기능을 제어 (로컬 전용) | 예정 |
| 한국어 UI | 메뉴·패널·탭·검색·대화상자 등 UI 한국어화, 한국어/영어 전환. 설정 `"ui_language"`: `"auto"`(기본, OS 언어를 따름) / `"ko"` / `"en"` (설정 화면 "일반" 맨 위에서도 바꿀 수 있음). 명령 팔레트는 한국어 이름과 영어 이름을 함께 보여 줌 | 1차 구현됨 (주요 UI 적용, 자동 테스트와 실제 창 캡처로 메뉴·우클릭 메뉴 확인). 설정 화면 본문·키맵 편집기·Git 패널·디버거 등은 아직 영어 |
| 기본 글꼴 | [D2Coding](./assets/fonts) 을 기본 글꼴로 사용 (편집기·UI 모두) | 적용됨 (화면 확인 대기) |
| 문법 강조 | 파일 확장자로 언어를 감지하여 기본 문법 강조. 대상 **26개**: Java, C, C++, SQL, C#, Rust, JavaScript(mjs·cjs 포함), TypeScript, Python, Ruby, JSON, JSON5, YAML(yml), Properties(`java.util.Properties` 호환), XML, HTML, XHTML, Markdown(md), Swift, Scala, R, Bat(Windows Batch), PowerShell, Sh(Unix Shell), Go, Dart. 이 중 15개는 원본에 내장되어 있고, **Java, C#, Ruby, JSON5, Properties, Swift, Scala, R, Bat, PowerShell, Dart 11개는 새로 추가** 샘플 파일은 [tests/samples/](./tests/samples) 에 있습니다. | 구현됨 (26개 감지·강조를 자동 테스트로 확인, 실제 창의 색상은 눈으로 미확인). 한계: `.xhtml` 의 `<?xml ?>` 선언 줄은 강조되지 않음 |
| 배포 zip 스크립트 | 빌드에 필요한 프로그램의 설치 여부를 검사하고, 없으면 설치 여부를 물은 뒤 배포용 zip 생성 ([사용법](#배포용-zip-만들기)) | 구현됨 (릴리스 zip 생성·실행 확인, 깨끗한 환경의 설치 흐름은 미확인) |
| 기존 기능 보존 | GramEditor 가 제공하는 기존 기능(Git, 디버거, 원격 개발, Vim/Helix 모드 등)을 삭제하지 않고 최대한 포함 | 원본 이관 상태 유지 (점검 예정) |

AI 관련 기능은 위 두 가지에서 시작하며, 이후 더 늘려 갈 예정입니다.

GramEditor 가 제공하는 기존 편집기 기능(다중 탭·분할, 검색, Git, 터미널, 디버거, LSP, Vim/Helix 모드 등)은 최대한 그대로 포함합니다.
사용법은 GramEditor 문서([GramEditor-docs/docs/](./GramEditor-docs/docs/))를 참고할 수 있으나, 일부 내용은 Don Editor 와 다를 수 있습니다.

## 원본 프로젝트와 고지

Don Editor 는 다음 프로젝트의 파생물입니다.

- **GramEditor** — <https://codeberg.org/GramEditor/gram> (GPL-3.0). 원본의 README 와 문서는 수정 없이 [GramEditor-docs/](./GramEditor-docs/) 폴더에 그대로 보존되어 있습니다(원본 소개: [GramEditor-docs/README.md](./GramEditor-docs/README.md)).
- **Zed** — GramEditor 는 Zed 편집기를 하드포크한 프로젝트입니다. Zed 코드베이스는 여러 라이선스로 배포되지만, 이 저장소에서 이루어지는 수정은 GPL-3.0 을 따릅니다.

이 저장소는 GramEditor 프로젝트 또는 그 관리자와 **관련이 없는 독립적인 파생 프로젝트**이며, 공식 배포물이 아닙니다.

### 원본과 달라지는 점

**가장 큰 차이: Don Editor 는 AI 기능을 적극적으로 도입합니다.**

- 원본 GramEditor 는 AI·텔레메트리 등을 의도적으로 제거한 편집기입니다. AI 기능을 넣지 않는 것을 프로젝트의 핵심 방침으로 삼고, AI 가 생성한 코드의 기여도 받지 않습니다
  (원본 [README](./GramEditor-docs/README.md) 의 "AI in Gram", 원본 [설립 취지](./GramEditor-docs/docs/mission.md), 원본 [기여 안내](./GramEditor-docs/CONTRIBUTING.md) 참고).
- Don Editor 는 이와 반대로 **AI 활용을 핵심 방향**으로 둡니다. 계획된 AI 관련 기능은 다음과 같으며, 이후에도 확장할 예정입니다.
  - AI Agent 사이드 패널: OpenAI 또는 Claude API Key 로 AI 와 대화하고 편집 작업을 맡깁니다.
  - 로컬 MCP 서버: 외부 AI 클라이언트가 편집기의 모든 기능을 제어할 수 있게 합니다.
- AI 기능은 별도의 서버나 구독 없이, 사용자가 직접 등록한 API Key 로 해당 서비스(OpenAI, Anthropic)를 호출하는 방식으로 설계합니다. API Key 는 설정 파일에 평문으로 저장하지 않습니다.
- 이 방향은 원본 프로젝트의 방침과 다르므로, Don Editor 는 원본과 **별개의 독립 파생 프로젝트**로 개발하며 원본 프로젝트에 기여하지 않습니다.
  원본 문서에 담긴 AI 사용 금지·인간 기여 정책은 **원본 프로젝트의 정책이며 이 프로젝트의 정책이 아닙니다**. 해당 문서는 [GramEditor-docs/](./GramEditor-docs/) 에 원본 그대로 보관합니다([원본 문서 모음](#원본-문서-모음) 참고).

그 밖에 한국어 UI, D2Coding 기본 글꼴, 문법 강조 언어 11개(Java·C#·Ruby·JSON5·Properties·Swift·Scala·R·Bat·PowerShell·Dart), 배포 스크립트가 추가·변경됩니다.
원본의 나머지 기능은 삭제하지 않고 유지합니다.
수정이 이루어진 위치에는 가능한 한 `[DonEditor]` 표시 주석을 남깁니다.

## 개발 환경 (Windows)

필요한 도구는 GramEditor 의 [Windows 개발 문서](./GramEditor-docs/docs/development/windows.md) 를 따릅니다.

- [rustup](https://www.rust-lang.org/tools/install) (사용할 Rust 버전은 [rust-toolchain.toml](./rust-toolchain.toml) 에 고정되어 있으며 자동 설치됩니다)
- Visual Studio 또는 Build Tools 의 MSVC x64 C++ 도구
- **Spectre 완화 라이브러리** (Visual Studio Installer 의 "MSVC 최신 버전 - C++ x64/x86 Spectre 완화 라이브러리", 컴포넌트 ID `Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre`). 설치되어 있지 않으면 의존 crate 의 빌드 스크립트가 실패합니다.
- Windows SDK
- [CMake](https://cmake.org/download)

```powershell
# 디버그 빌드 (기본 대상: crates/gram)
cargo build

# 실행
cargo run
```

실행 파일은 `target\debug\doneditor.exe` 입니다. 설정은 `%APPDATA%\DonEditor`, 데이터는 `%LOCALAPPDATA%\DonEditor` 에 저장되므로 원본 Gram 의 설정과 섞이지 않고, 원본 Gram 과 동시에 실행할 수 있습니다.

> 원본을 이관한 상태에서 Windows 디버그·릴리스 빌드가 모두 성공하고, (디버그 빌드에서) 파일 열기·편집·저장과 한글 입력이 동작함을 확인했습니다(2026-10-27). 상세 내용은 [PROGRESS.md](./PROGRESS.md) 의 Phase 1 항목을 참고하세요.
### 배포용 zip 만들기

```powershell
# 도구 점검 → (없으면 설치 여부 질문) → 릴리스 빌드 → dist\ 에 zip 생성
.\package-zip.bat

# 점검만 해 보기 (설치·빌드 안 함)
.\package-zip.bat -CheckOnly
```

[package-zip.bat](./package-zip.bat) 은 실행 정책에 막히지 않도록 [script/package-zip.ps1](./script/package-zip.ps1) 을 호출하는 래퍼이며, 인자는 그대로 전달됩니다.

1. **도구 점검:** Git, rustup, Rust 툴체인(`rust-toolchain.toml`), MSVC C++ 도구 + Spectre 완화 라이브러리, Windows SDK(≥ 10.0.20348), CMake 가 있는지 확인합니다. 없는 것은 **설치할지 묻고**, 동의하면 `winget`(Visual Studio 는 Installer)으로 설치합니다. 설치 중 관리자 권한 확인창이 나타날 수 있습니다.
2. **빌드:** `cargo build --release` 로 GUI(`doneditor.exe`)와 CLI 를 만듭니다. 릴리스 채널은 `crates/gram/RELEASE_CHANNEL` 값(stable)을 사용합니다.
3. **패키징:** `dist\DonEditor-<버전>-windows-<아키텍처>.zip` 과 `.sha256` 파일을 만듭니다. zip 안에는 `doneditor.exe`, `bin\doneditor.exe`(CLI), `LICENSE`, `licenses.md`, `licenses\`, `README.md`, `SOURCE.txt` 가 들어갑니다.

| 옵션 | 설명 |
|------|------|
| `-Yes` | 설치 질문에 모두 "예"로 답함 |
| `-CheckOnly` / `-DryRun` | 점검만 / 실행할 설치 명령만 출력 (둘 다 빌드하지 않음) |
| `-IncludeSource` | `git archive` 로 만든 소스 zip 도 함께 생성 (커밋된 내용만 포함) |
| `-SkipCheck`, `-SkipBuild` | 도구 점검 / 빌드 건너뛰기 |
| `-BuildProfile Debug` | 디버그 빌드로 패키징 (스크립트 점검용, 파일이 매우 큼) |
| `-Arch`, `-OutDir` | 대상 아키텍처(`aarch64` 는 미검증), 결과물 폴더 |

전체 옵션은 `package-zip.bat -Help` 로 볼 수 있습니다. 배포 시 GPL-3.0 에 따라 **소스 코드를 제공할 방법**(저장소 주소 또는 `-IncludeSource` 로 만든 소스 zip)이 필요하며, 이 정보는 zip 안의 `SOURCE.txt` 에 기록됩니다.

## 문서

| 문서 | 설명 |
|------|------|
| [PROGRESS.md](./PROGRESS.md) | 개발 계획, 단계별 절차, 진행 상황 |
| [AGENTS.md](./AGENTS.md) | 프로젝트 요구사항과 작업 규칙 |
| [FEATURE-INVENTORY.md](./FEATURE-INVENTORY.md) | 원본 GramEditor 기능 인벤토리와 점검 체크리스트 (기존 기능 보존 확인용) |
| [GramEditor-docs/](./GramEditor-docs/) | **원본 GramEditor 문서 모음** (아래 참고) |
| [GramEditor-docs/README.md](./GramEditor-docs/README.md) | 원본 GramEditor 의 README |
| [GramEditor-docs/docs/](./GramEditor-docs/docs/) | 원본 사용자 문서 ([설정](./GramEditor-docs/docs/configuring-gram.md), [단축키](./GramEditor-docs/docs/key-bindings.md), [언어](./GramEditor-docs/docs/languages.md), [테마](./GramEditor-docs/docs/themes.md) 등) |
| [GramEditor-docs/docs/mission.md](./GramEditor-docs/docs/mission.md) | 원본 GramEditor 의 설립 취지 |
| [GramEditor-docs/CONTRIBUTING.md](./GramEditor-docs/CONTRIBUTING.md), [CODE_OF_CONDUCT.md](./GramEditor-docs/CODE_OF_CONDUCT.md), [SECURITY.md](./GramEditor-docs/SECURITY.md), [CHANGELOG.md](./GramEditor-docs/CHANGELOG.md) | 원본의 기여 안내, 행동 강령, 보안 정책, 변경 이력 |

### 원본 문서 모음

[GramEditor-docs/](./GramEditor-docs/) 는 원본 GramEditor(버전 3.4.0)의 README, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`, `docs/` 를 **내용 수정 없이 그대로** 옮겨 둔 폴더입니다.
원본 저장소와 같은 상대 구조를 유지하므로, 문서 사이의 링크는 그대로 동작합니다.

- 이 문서들은 원본 프로젝트의 것이며 Don Editor 의 문서가 아닙니다. 특히 "AI 금지(No AI / No LLM)", "인간 기여 정책" 등 AI 사용을 제한하는 내용
  ([CONTRIBUTING.md](./GramEditor-docs/CONTRIBUTING.md), [CODE_OF_CONDUCT.md](./GramEditor-docs/CODE_OF_CONDUCT.md), [docs/index.md](./GramEditor-docs/docs/index.md),
  [docs/mission.md](./GramEditor-docs/docs/mission.md), [docs/HUMAN-CONTRIBUTION-POLICY-2-UP.md](./GramEditor-docs/docs/HUMAN-CONTRIBUTION-POLICY-2-UP.md))은
  원본 프로젝트의 정책이며, AI 를 적극 도입하는 Don Editor 와는 방향이 다릅니다.
- 앱에 내장된 도움말(`gram://docs/...`)은 현재 [GramEditor-docs/docs/](./GramEditor-docs/docs/) 를 그대로 표시합니다.
- 원본 문서 안의 일부 링크(소스 디렉터리 `crates/...`, `LICENSE`)는 폴더 이동으로 연결되지 않습니다. 해당 자료는 이 저장소의 [crates/](./crates/), [LICENSE](./LICENSE) 에 있습니다.
- Don Editor 자체 문서는 이 폴더와 별도로 작성합니다.

## 라이선스

Don Editor 는 **GNU General Public License v3.0** ([LICENSE](./LICENSE)) 에 따라 배포됩니다.
원본 GramEditor 가 GPL-3.0 으로 배포되므로, 이 파생물 또한 GPL-3.0 을 유지하며 원본의 라이선스와 저작권 표기를 삭제하지 않습니다.

- 라이선스 전문과 고지: [LICENSE](./LICENSE), [assets/licenses.md](./assets/licenses.md)
- 원본 저작권: Copyright (C) 2026 The Gram Project (GramEditor), 그리고 Zed 및 각 의존성의 저작권자
- 글꼴(D2Coding, Fira Sans, Myna, OpenMoji)과 그 밖의 서드파티 자료는 각자의 라이선스를 따릅니다. Fira Sans, Myna, OpenMoji 의 라이선스 파일은 [assets/fonts](./assets/fonts) 의 각 폴더에 있으며, D2Coding 의 라이선스 고지는 추가할 예정입니다([PROGRESS.md](./PROGRESS.md) 3.8 참고).

이 프로젝트는 원본 소스와 의존성의 라이선스를 따르며, 원본 크레딧은 <https://gram-editor.com/credits> 에서 확인할 수 있습니다.
