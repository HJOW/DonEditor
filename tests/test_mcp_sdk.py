"""공식 MCP Python SDK 2.x로 실제 Rust HTTP 서버와 stdio 브릿지를 검증한다.

준비: cargo build -p don_protocol_tests --bins --target-dir target/contracts
      python -m pip install 'mcp>=2,<3'
실행: python tests/test_mcp_sdk.py
사용자 편집기나 API 키를 사용하지 않으며 격리된 고정 문서 서버에 연결한다.
"""
import asyncio
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

from mcp import Client, StdioServerParameters
from mcp.client.streamable_http import streamable_http_client
import httpx2


async def verify(client):
    """도구·리소스 조회 및 한글 응답이 실제 SDK로 파싱되는지 검사한다."""
    tools = await client.list_tools()
    assert len(tools.tools) == 32
    result = await client.call_tool("get_buffer_text", {})
    assert result.structured_content["text"] == "Don Editor 한글🙂"
    resources = await client.list_resources()
    assert len(resources.resources) == 3
    contents = await client.read_resource("doneditor://active-buffer")
    assert json.loads(contents.contents[0].text)["text"] == "Don Editor 한글🙂"


async def connect(connection_path, bridge):
    """구형 초기화와 현재 SDK 자동 협상의 HTTP·stdio 전송을 모두 확인한다."""
    connection = json.loads(connection_path.read_text(encoding="utf-8"))
    # SDK 2.x의 자동 탐색은 구형 서버에서 initialize 방식으로 폴백해야 한다.
    for mode in ("legacy", "auto"):
        async with httpx2.AsyncClient(
            headers={"Authorization": "Bearer " + connection["token"]},
            timeout=30.0,
        ) as http:
            transport = streamable_http_client(
                f"http://127.0.0.1:{connection['port']}/mcp", http_client=http
            )
            async with Client(transport, mode=mode) as client:
                await verify(client)
        parameters = StdioServerParameters(
            command=str(bridge), args=["--connection", str(connection_path)]
        )
        async with Client(parameters, mode=mode) as client:
            await verify(client)
    print("공식 MCP SDK: HTTP·stdio 초기화/자동 협상/32개 도구/3개 리소스 검증 통과")


def main():
    """테스트 서버 프로세스를 격리해 시작하고 성공·실패 모두 정리한다."""
    root = Path(__file__).resolve().parents[1]
    suffix = ".exe" if os.name == "nt" else ""
    binaries = Path(os.environ.get("DON_PROTOCOL_TEST_BIN_DIR", str(root / "target/contracts/debug")))
    fixture = binaries / ("don-mcp-test-server" + suffix)
    bridge = binaries / ("don-mcp-contract-bridge" + suffix)
    assert fixture.exists() and bridge.exists(), "먼저 테스트 바이너리를 빌드하세요"
    with tempfile.TemporaryDirectory(prefix="don-sdk-", dir=root / "target") as folder:
        connection = Path(folder) / "private/connection.json"
        flags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
        process = subprocess.Popen(
            [str(fixture), str(connection)], stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, creationflags=flags,
        )
        try:
            deadline = time.monotonic() + 30
            # 연결 파일이 준비되기 전에는 포트나 토큰을 추측하지 않는다.
            while not connection.exists():
                assert process.poll() is None, "테스트 서버 시작 실패"
                assert time.monotonic() < deadline, "테스트 서버 시작 시간 초과"
                time.sleep(0.05)
            asyncio.run(connect(connection, bridge))
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.terminate()
                process.wait(timeout=10)
            process.stderr.close()


if __name__ == "__main__":
    main()
