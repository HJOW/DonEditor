"""격리된 실제 편집기에 공식 MCP SDK를 연결해 새 기능(Typst·인코딩·미리보기 전환)의 액션을 확인한다.

준비: MCP 서버를 켠 격리 데이터 폴더로 doneditor 를 실행하고(`--user-data-dir`), 인자로 브릿지·연결 파일·작업 폴더를 넘긴다.
  python tests/test_mcp_live_actions.py <don-mcp 경로> <connection.json 경로> <작업 폴더> [--dispatch]

- 기본(읽기 전용): `list_actions` 에 새 액션이 모두 등록되어 있고 인자 스키마가 있는지 확인한다.
- `--dispatch`: `dispatch_action` 으로 실제 실행한다. 이 도구는 편집기에서 사람의 승인을 받아야 하므로, 시험하는 사람이 편집기의
  승인 창에서 허용해야 끝난다. 작업 폴더에 `문서.md` 가 열려 있어야 하고, 실행 뒤 결과 파일(`mcp-out.typ`, `mcp-out.pdf`)이
  저장 창 없이 만들어졌는지 확인한다.
사용자 편집기나 API 키를 사용하지 않는다.
"""
import asyncio
import sys
from pathlib import Path

from mcp import Client, StdioServerParameters

# 새 기능이 등록해야 하는 액션 이름
EXPECTED_ACTIONS = [
    "typst::TogglePreviewMode",
    "typst::ConvertMarkdownToTypst",
    "typst::ExportPdf",
    "typst::Print",
    "markdown::TogglePreviewMode",
    "encoding_selector::Toggle",
    "encoding_selector::ReloadWithEncoding",
]


async def main():
    bridge, connection, workdir = map(Path, sys.argv[1:4])
    dispatch = "--dispatch" in sys.argv[4:]
    params = StdioServerParameters(command=str(bridge), args=["--connection", str(connection)])
    async with Client(params, mode="legacy") as client:
        listed = await client.call_tool("list_actions", {})
        assert not listed.is_error, listed
        names = {action["name"] for action in listed.structured_content["actions"]}
        missing = [name for name in EXPECTED_ACTIONS if name not in names]
        assert not missing, f"등록되지 않은 액션: {missing}"
        print("list_actions: 새 액션 %d개 모두 등록됨" % len(EXPECTED_ACTIONS), flush=True)

        if not dispatch:
            return

        # 인코딩: 잘못 감지된 파일을 UTF-8 로 다시 불러온다 (작업 폴더의 mixed.txt 가 열려 있어야 한다)
        # 변환·PDF: 열려 있는 Markdown 문서를 저장 창 없이 지정한 경로에 저장한다
        for name, args, output in [
            ("typst::ConvertMarkdownToTypst", {"path": str(workdir / "mcp-out.typ")}, workdir / "mcp-out.typ"),
            ("typst::ExportPdf", {"path": str(workdir / "mcp-out.pdf")}, workdir / "mcp-out.pdf"),
        ]:
            output.unlink(missing_ok=True)
            print(f"dispatch_action {name} (편집기에서 승인하세요)", flush=True)
            result = await client.call_tool("dispatch_action", {"name": name, "args": args})
            assert not result.is_error, result
            # 저장은 백그라운드에서 끝나므로 잠시 기다린다
            for _ in range(60):
                if output.exists() and output.stat().st_size > 0:
                    break
                await asyncio.sleep(0.5)
            assert output.exists() and output.stat().st_size > 0, f"{output} 가 만들어지지 않았다"
            print(f"  -> {output.name} {output.stat().st_size}바이트", flush=True)
        assert (workdir / "mcp-out.typ").read_text(encoding="utf-8").startswith("// Don Editor")
        assert (workdir / "mcp-out.pdf").read_bytes().startswith(b"%PDF-")
        print("dispatch_action: 변환·PDF 저장 통과", flush=True)


asyncio.run(main())
