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

공통 편집기 사용법은 [원본 GramEditor 도움말](gram://docs/index.md)도 참고할 수 있습니다. 원본 문서는 그대로 보존하며, 앱에서 열 때 출처 안내를 표시합니다. 원본에 적힌 프로젝트 정책·제품명·경로는 Don Editor와 다를 수 있습니다.

## 기존 프로젝트와의 호환

프로젝트별 설정 폴더는 `.gram/`을 유지합니다. 원본 GramEditor와 같은 프로젝트 설정을 공유하려는 목적입니다. 사용자 설정·데이터는 DonEditor 전용 경로를 사용합니다. 기본 UI·편집기 글꼴은 D2Coding이며, 현재의 파란색/주황색 D 아이콘은 임시 디자인입니다.

## 라이선스와 소스

Don Editor는 GPL-3.0 또는 이후 버전을 따릅니다. GramEditor와 Zed의 원저작·제3자·글꼴 고지를 보존합니다. 배포본의 `LICENSE`, `licenses.md`, `licenses/`, `SOURCE.txt`를 확인하세요. Windows zip과 Unix tar.gz에는 패키징 시점의 프로젝트 소스를 `source/`에 동봉합니다.
