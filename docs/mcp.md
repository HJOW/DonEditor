# 로컬 MCP 연결

[사용 안내 목차](index.md) · [도구 승인과 데이터 보관](security.md)

Don Editor는 실행 중인 편집기를 외부 MCP 클라이언트에서 제어할 수 있도록 루프백 HTTP 서버와 `don-mcp` stdio 브릿지를 제공합니다. MCP 서버는 기본적으로 꺼져 있으며 Agent 패널 사용만을 위해 켤 필요는 없습니다.

## 서버 켜기

명령 팔레트의 `mcp_server::ToggleServer` 또는 MCP 메뉴로 켜고, `mcp_server::ShowConnectionInfo`로 상태와 연결 정보 파일 경로를 확인합니다. 사용자 설정으로도 제어할 수 있습니다.

```json
{
  "mcp_server": {
    "enabled": true,
    "port": 0
  }
}
```

`port: 0`은 비어 있는 로컬 포트를 자동 배정합니다. 지정한 포트를 사용할 수 없으면 연결 정보에 오류가 표시됩니다. 서버를 끄거나 다시 시작하면 연결 토큰도 교체합니다. 연결 파일의 실제 토큰을 채팅·설정 예시·저장소에 복사하지 마세요.

## Claude Desktop 예시

Claude Desktop의 개발자 설정에서 MCP 설정 파일을 열고 기존 `mcpServers`에 아래 항목을 합칩니다. 기존 서버 설정은 보존하세요. 경로는 압축을 푼 실제 배포 폴더의 **절대 경로**로 바꿉니다.

```json
{
  "mcpServers": {
    "doneditor": {
      "command": "C:\\Apps\\DonEditor\\bin\\don-mcp.exe",
      "args": []
    }
  }
}
```

macOS/Linux에서는 `command`를 `/Users/me/Apps/DonEditor/bin/don-mcp` 또는 `/home/me/Apps/DonEditor/bin/don-mcp`처럼 지정합니다. 이 브릿지는 독립 실행 파일이므로 Node.js나 npx를 요구하지 않습니다. 설정 저장 후 Claude Desktop을 완전히 종료했다가 다시 열어 연결합니다. 설정 구조와 개발자 설정 위치는 [공식 로컬 MCP 연결 안내](https://modelcontextprotocol.io/docs/develop/connect-local-servers)를 참고하세요.

## Claude Code 예시

Don Editor를 열고 서버를 켠 뒤 등록합니다.

```powershell
claude mcp add --transport stdio doneditor -- "C:\Apps\DonEditor\bin\don-mcp.exe"
claude mcp get doneditor
```

```sh
claude mcp add --transport stdio doneditor -- /home/me/Apps/DonEditor/bin/don-mcp
claude mcp get doneditor
```

클라이언트의 `/mcp`에서 상태를 확인할 수 있습니다. 명령의 `--`는 클라이언트 옵션과 브릿지 실행 명령을 구분합니다. [Claude Code 공식 MCP 문서](https://code.claude.com/docs/en/mcp)의 stdio 등록 형식에 따른 예시입니다.

## 격리 데이터 폴더와 연결 파일

브릿지는 기본적으로 앱 데이터 폴더의 `mcp/connection.json`을 읽습니다. Windows는 `%LOCALAPPDATA%/DonEditor/mcp/connection.json`, macOS는 `~/Library/Application Support/DonEditor/mcp/connection.json`, Linux는 `$XDG_DATA_HOME/doneditor/mcp/connection.json` 또는 `~/.local/share/doneditor/mcp/connection.json`입니다.

편집기를 `--user-data-dir`로 실행했다면 연결 정보 창에 표시한 경로를 브릿지의 `--connection` 인자로 넘깁니다.

```json
{
  "mcpServers": {
    "doneditor-isolated": {
      "command": "C:\\Apps\\DonEditor\\bin\\don-mcp.exe",
      "args": ["--connection", "C:\\Temp\\DonEditor-test\\mcp\\connection.json"]
    }
  }
}
```

`DONEDITOR_MCP_CONNECTION` 환경 변수로도 파일 경로를 지정할 수 있습니다. 토큰을 직접 넘기는 방식이 아니라 보호된 파일에서 읽습니다. 초기화할 때 연결 정보와 세션을 확정하므로 편집기의 MCP 서버를 다시 시작했다면 클라이언트의 MCP 연결도 다시 시작합니다.

직접 HTTP로 연결하는 클라이언트는 `/mcp`에 Bearer 인증을 보내고, 초기화 응답의 `MCP-Session-Id`를 후속 요청에 유지해야 합니다. `notifications/cancelled`로 요청을 취소하거나 `DELETE /mcp`로 세션을 종료할 수 있습니다. 연결이 끊긴 요청도 취소되며, 취소된 승인창은 편집을 적용하지 않습니다.

전송 메시지는 16 MiB, 중복 인코딩되는 도구 결과는 4 MiB로 제한하며 초과하면 오류를 반환합니다. 큰 파일의 전체 조회 대신 파일 내 검색처럼 결과가 작은 도구를 사용할 수 있습니다. 현재 범위 편집은 단일 파일 탭에서 지원하며, 여러 파일의 발췌 화면에서는 원본 파일 탭을 먼저 엽니다.

## 사용할 수 있는 기능

`list_actions`로 편집기에 등록된 액션을 조회하고 `dispatch_action`으로 이름과 인자를 전달합니다. 저장, 편집, 검색, 프로젝트 파일 목록, 진단, 개요, 패널, 분할, 테마·언어·설정, 터미널 실행을 위한 전용 도구도 제공합니다. `doneditor://active-buffer`, `doneditor://project-tree`, `doneditor://diagnostics` 리소스로 현재 상태를 조회할 수 있습니다.

변경·터미널·범용 액션은 편집기 승인창을 확인해야 합니다. 승인 대기 시간은 제한되어 있으며, 취소되거나 만료된 요청은 새로 요청해야 합니다. 범용 액션의 성공 응답은 **액션이 전달됐다는 뜻**이며 비동기 작업의 완료까지 보장하지 않습니다. 텍스트 범위 도구의 위치는 UTF-8 바이트 오프셋입니다. 편집 결과를 확인하고 별도로 저장하세요.

연결되지 않으면 편집기 실행 여부, MCP 상태, 실행 파일 경로, 격리 연결 경로를 순서대로 확인합니다. 읽기 전용 요청인 열린 탭 조회부터 시도하고, 승인창이 나타나면 해당 Don Editor 창에서 응답하세요.
