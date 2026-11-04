Don Editor
------------------------------------------------------------------
# 프로젝트 목표

GramEditor 을 클론 및 커스텀하여
요구사항을 만족하는 텍스트 편집기 개발

# GramEditor 프로젝트

[D:\Workspace\git\GramEditor](https://codeberg.org/GramEditor/gram)
소스코드 위치 (HJOW-Athlon) : H:\Workspace\git\GramEditor 
소스코드 위치 (vm-debian-codex) : /home/hjow/Workspaces/git/gram

# 추가 요구사항

- 한국어 UI 지원
- 기본 글꼴은 D2Coding 을 사용한다. (assets/fonts 디렉토리 참고)
- 다음 프로그래밍 언어에 대해 기본적인 문법 강조 지원 (파일 확장자를 통해 프로그래밍 언어 감지)
  java, c, cpp, sql, c#, rust, javascript (mjs, cjs 포함), typescript, python, ruby, json, json5, yml (yaml), properties (java 의 java.util.Properties 와 호환), xml, html, xhtml, md (Markdown), swift, scala, r, bat (Windows Batch), powershell, sh (Unix Shell), go, dart
- 프로그램의 모든 기능을 컨트롤할 수 있는 MCP (로컬) 적용
- OpenAI 혹은 Claude API Key 를 연동하여 Agent 기능을 사용할 수 있는 사이드 패널 (기본적으로 우측에 배치) 개발
  이 사이드 패널은 숨길 수 있어야 하며, 별도 아이콘을 화면 내에 배치해 아이콘을 클릭해 다시 사이드 패널을 띄울 수 있도록 한다.
- Windows 배포용 zip 파일 생성 스크립트 (별도 파일로 준비, bat 혹은 ps1 사용) 작성
  이 스크립트 호출 시, 빌드에 필요한 프로그램 존재여부도 검사하여, 미설치 시 사용자에게 설치를 묻는 절차를 포함한다.
- Linux/MAC 배포용 tar.gz 파일 생성 스크립트 (별도 파일로 준비, sh 사용) 작성
  이 스크립트 호출 시, 빌드에 필요한 프로그램 존재여부도 검사하여, 미설치 시 사용자에게 설치를 묻는 절차를 포함한다.
- 그외 기존에 존재하던 기능은 최대한 포함한다.

# 지켜야 할 사항

- 작업 전 PROGRESS.md 를 읽어 진행 상황을 파악한다.
- 작업 완료, 혹은 중단 시 PROGRESS.md 에 진행 상황을 한국어로 기재한다.
- 소스코드 내 상수, 함수, 반복문에는 한국어로 주석을 단다.
- 라이센스는 원본 프로그램인 GramEditor 의 라이센스에 따라 GPL 3.0 으로 한다.
  원본 프로그램 라이센스에 따라 원본 라이센스 또한 규칙에 따라 명기가 필요하다.