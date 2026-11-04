# 설치와 기본 설정

[사용 안내 목차](index.md)

## 배포본 실행

Windows에서는 배포 zip을 쓰기 가능한 폴더에 풀고 `doneditor.exe`를 실행합니다. 명령줄 도구는 같은 폴더의 `bin/doneditor.exe`입니다.

Linux에서는 tar.gz를 풀고 `bin/doneditor`를 실행합니다. 빌드한 배포판과 호환되는 glibc 및 GUI/시스템 라이브러리가 필요합니다. macOS에서는 배포 폴더의 `Don Editor.app`을 열거나 `bin/doneditor`를 실행합니다. MCP 브릿지도 사용하려면 `.app`만 옮기지 말고 배포 폴더의 `bin/`도 함께 보관하세요. Linux/macOS의 Git 기능은 시스템 Git을 사용합니다. macOS 배포본의 서명·공증 여부는 배포자의 안내를 확인하세요.

파일은 파일 메뉴의 열기, 프로젝트는 폴더 열기로 엽니다. 파일을 편집한 뒤 저장하거나 다른 이름으로 저장할 수 있습니다. 명령 팔레트에서 기능 이름을 검색할 수 있으며, 한국어 UI에서는 영어 액션 이름도 함께 표시합니다.

## 표시 언어와 글꼴

설정 화면의 일반 항목에서 UI 언어를 선택하거나 사용자 설정 파일에 다음 값을 넣습니다.

```json
{
  "ui_language": "ko",
  "buffer_font_family": "D2Coding",
  "ui_font_family": "D2Coding"
}
```

`ui_language`는 `auto`(OS 언어), `ko`, `en`을 지원합니다. 일부 메뉴·열린 대화상자는 다시 열거나 앱을 다시 시작해야 변경된 언어가 보일 수 있습니다. D2Coding은 배포본에 내장되어 있으며 별도로 OS에 설치할 필요가 없습니다. 파일 확장자로 문법을 자동 감지하고, 필요한 경우 상태 표시줄의 언어 선택으로 바꿀 수 있습니다.

## 설정과 데이터 위치

| OS | 사용자 설정 | 데이터·대화 기록·MCP 연결 정보 |
|---|---|---|
| Windows | `%APPDATA%/DonEditor` | `%LOCALAPPDATA%/DonEditor` |
| macOS | `~/.config/doneditor` | `~/Library/Application Support/DonEditor` |
| Linux | `$XDG_CONFIG_HOME/doneditor` 또는 `~/.config/doneditor` | `$XDG_DATA_HOME/doneditor` 또는 `~/.local/share/doneditor` |

정확한 설정 파일은 앱의 설정 파일 열기 메뉴를 사용하면 찾을 수 있습니다. `--user-data-dir <경로>`로 실행하면 설정과 데이터를 해당 폴더에 격리합니다. 프로젝트 안의 `.gram/` 설정은 원본과 호환되도록 유지합니다. API 키는 이 설정 파일에 적지 않고 [Agent 패널](agent.md)의 키 저장 기능을 사용합니다.

## 소스에서 빌드

도구만 준비하려면 Windows의 `install-build-tools.bat` 또는 Linux/macOS의 `bash install-build-tools.sh`를 실행합니다. 처음에는 모든 항목이 미선택이며, 번호를 고르고 `I`를 입력해야 설치합니다. 조회만 하려면 `-CheckOnly` 또는 `--check-only`를 사용합니다.

프로젝트 Rust 버전은 `rust-toolchain.toml`을 따릅니다. 도구가 준비되면 저장소 루트에서 `cargo build --package gram --package cli --package mcp_server`로 GUI·CLI·MCP 브릿지를 빌드할 수 있습니다. Windows에는 MSVC, Spectre 완화 라이브러리와 Windows SDK가 필요하며, macOS에는 Xcode/SDK/Metal 도구가 필요합니다. 배포본 생성과 도구 누락 시 설치 절차는 [플랫폼별 패키징](packaging.md)을 참고하세요.
