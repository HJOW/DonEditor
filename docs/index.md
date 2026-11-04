# Don Editor 사용 안내

Don Editor는 GramEditor를 기반으로 한국어 UI, D2Coding 기본 글꼴, 로컬 MCP와 AI Agent 패널을 제공하는 텍스트 편집기입니다. **AI 기능을 적극적으로 도입**하며, 원본 GramEditor의 AI 사용 금지 정책은 이 프로젝트에 적용되지 않습니다.

현재 Don Editor 버전은 **0.1.0**이며, 기반이 된 GramEditor 3.4.0과 별도로 관리합니다.

## 시작하기

1. [설치와 기본 설정](installation.md)에서 배포본을 실행하고 파일이나 프로젝트를 엽니다.
2. [Agent 패널](agent.md)에서 OpenAI 또는 Anthropic API 키를 설정합니다.
3. 외부 도구와 연결하려면 [로컬 MCP 연결](mcp.md)을 참고합니다.
4. [도구 승인과 데이터 보관](security.md)에서 변경 승인, 전송 범위와 기록 저장 위치를 확인합니다.

소스에서 배포본을 만드는 방법은 [플랫폼별 패키징](packaging.md)에 있습니다.

## 편집기 기능

파일 열기·편집·저장, 다중 탭·분할 창, 찾기·바꾸기, Git, 터미널, 디버거, 언어 서버, Vim/Helix 모드 등 기존 기능을 유지합니다. 기본 문법 강조 대상은 Java, C, C++, SQL, C#, Rust, JavaScript(mjs/cjs 포함), TypeScript, Python, Ruby, JSON, JSON5, YAML, Java Properties, XML, HTML, XHTML, Markdown, Swift, Scala, R, Windows Batch, PowerShell, Unix Shell, Go, Dart입니다.

Markdown 파일(`.md` 등)은 열면 **미리보기 모드**로 먼저 표시됩니다. 탭 아래 현재 단락 제목 줄의 오른쪽에 있는 **미리보기 | 편집** 버튼으로 같은 탭 안에서 편집 모드와 오갈 수 있으며, 명령 팔레트의 `markdown: toggle preview mode`로도 전환합니다. 미리보기에서 블록을 두 번 누르면 그 위치에서 편집 모드로 바뀝니다. 내용이 없는 파일은 편집 모드로 열립니다. 저장·변경 표시·닫기 확인은 두 모드에서 똑같이 동작하며, 별도 탭이나 옆 분할 창에 미리보기를 여는 기존 기능도 그대로 쓸 수 있습니다.

### 문자 인코딩(캐릭터셋)

화면 오른쪽 아래 상태 표시줄에서 줄 번호와 언어 이름 옆에 **현재 파일의 인코딩**(예: `UTF-8`, `EUC-KR`, `UTF-16LE`, BOM이 있으면 `(BOM)`)이 항상 표시됩니다. 파일을 열 때 BOM과 바이트 모양, 문자 분포로 인코딩을 자동 감지하며, 설정 `status_bar.active_encoding_button`으로 표시 방식을 바꿀 수 있습니다.

- **다른 인코딩으로 다시 불러오기:** 인코딩 이름을 누르면 선택 화면이 열리고(명령 팔레트 `encoding selector: toggle`), 고른 인코딩으로 **디스크의 파일을 강제로 다시 읽습니다.** 파일을 연 뒤 글자가 깨져 있을 때, 또는 한 파일에 여러 인코딩이 섞여 있어 인코딩을 바꿔 가며 부분씩 복원할 때 씁니다. 깨져서 해독할 수 없는 바이트는 `�`로 표시됩니다.
- **다시 불러오기는 저장하지 않습니다.** 디스크의 파일은 바뀌지 않으며, 파일을 새로 연 것과 같이 실행 취소 기록이 지워지고 변경 없음 상태가 됩니다. 저장하지 않은 변경이 있으면 버릴지 한 번 물어봅니다. 이 컴퓨터에 저장된 파일만 다시 불러올 수 있습니다.
- **저장은 현재 선택된 인코딩으로 합니다.** 상태 표시줄에 보이는 인코딩과 BOM 여부 그대로 저장합니다. 그 인코딩으로 표현할 수 없는 문자(예: EUC-KR 파일 속 이모지)가 있으면 글자를 바꿔 쓰지 않고 **저장을 중단하고** 문제의 문자와 줄 번호를 알려 줍니다.
- MCP나 키 바인딩에서는 `encoding_selector::ReloadWithEncoding` 액션(`{"encoding": "euc-kr"}`, 저장하지 않은 변경을 확인 없이 버리려면 `"force": true`)으로 선택 화면 없이 다시 불러올 수 있습니다.

공통 편집기 사용법은 [원본 GramEditor 도움말](gram://docs/index.md)도 참고할 수 있습니다. 원본 문서는 그대로 보존하며, 앱에서 열 때 출처 안내를 표시합니다. 원본에 적힌 프로젝트 정책·제품명·경로는 Don Editor와 다를 수 있습니다.

## 기존 프로젝트와의 호환

프로젝트별 설정 폴더는 `.gram/`을 유지합니다. 원본 GramEditor와 같은 프로젝트 설정을 공유하려는 목적입니다. 사용자 설정·데이터는 DonEditor 전용 경로를 사용합니다. 기본 UI·편집기 글꼴은 D2Coding이며, 현재의 파란색/주황색 D 아이콘은 임시 디자인입니다.

## 라이선스와 소스

Don Editor는 GPL-3.0 또는 이후 버전을 따릅니다. GramEditor와 Zed의 원저작·제3자·글꼴 고지를 보존합니다. 배포본의 `LICENSE`, `licenses.md`, `licenses/`, `SOURCE.txt`를 확인하세요. Windows zip과 Unix tar.gz에는 패키징 시점의 프로젝트 소스를 `source/`에 동봉합니다.
