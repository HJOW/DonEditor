"""가짜 Cargo metadata로 라이선스 정책·범위·고지 보존과 CLI 실패를 검사한다."""

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


# 실제 Cargo나 네트워크를 호출하지 않고 검사 함수를 로드한다.
REPOSITORY = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("don_license_check", REPOSITORY / "script/check_don_licenses.py")
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


# SPDX의 대안/동시 적용과 손상된 구문을 독립적으로 확인한다.
class SpdxTests(unittest.TestCase):
    # OR는 허용 대안을 선택하고 AND는 모든 조건을 충족해야 한다.
    def test_allowed_and_disallowed_expressions(self):
        for expression in ("GPL-3.0-or-later", "GPL-3.0-only", "Apache-2.0", "MIT OR Apache-2.0", "MIT OR GPL-2.0-only", "(MIT AND Apache-2.0) OR LicenseRef-Private"):
            with self.subTest(expression=expression):
                self.assertTrue(CHECK.license_allowed(expression))
        for expression in ("GPL-2.0-only", "LicenseRef-Private", "MIT AND GPL-2.0-only", "LicenseRef-Private OR GPL-2.0-only"):
            with self.subTest(expression=expression):
                self.assertFalse(CHECK.license_allowed(expression))

    # 잘못된 식이 앞의 허용 토큰만 보고 통과하지 않게 한다.
    def test_malformed_or_unsupported_expressions(self):
        for expression in (None, "", "MIT OR", "(MIT", "MIT)", "MIT Apache-2.0", "MIT/Apache-2.0", "MIT OR (AND MIT)", "Apache-2.0 WITH LLVM-exception"):
            with self.subTest(expression=expression):
                with self.assertRaises(ValueError):
                    CHECK.license_allowed(expression)


# 임시 저장소 밖으로 접근하지 않는 metadata를 만들고 원본 라이선스를 복사한다.
class MetadataTests(unittest.TestCase):
    # 각 테스트마다 고지와 metadata를 독립적으로 구성한다.
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="don licenses ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "assets").mkdir()
        shutil.copyfile(REPOSITORY / "LICENSE", self.root / "LICENSE")
        (self.root / "assets/licenses.md").write_text("Copyright (C) 2026 The Gram Project\nGramEditor is a fork of Zed.\n", encoding="utf-8")
        self.metadata = {
            "workspace_root": str(self.root),
            "workspace_members": ["local-a", "local-b"],
            "packages": [
                {"id": "local-a", "name": "local-a", "source": None, "license": "GPL-3.0-or-later", "manifest_path": str(self.root / "crates/a/Cargo.toml")},
                {"id": "local-b", "name": "local-b", "source": None, "license": "MIT OR Apache-2.0", "manifest_path": str(self.root / "crates/b/Cargo.toml")},
            ],
        }

    # 해석 완료된 상속 값은 license_file 심볼릭 링크 없이도 검사할 수 있다.
    def test_resolved_workspace_licenses(self):
        count, errors = CHECK.check_metadata(self.metadata)
        self.assertEqual(count, 2)
        self.assertEqual(errors, [])

    # 외부 라이선스 전체 감사를 했다고 잘못 보고하지 않도록 workspace_members만 검사한다.
    def test_external_dependency_is_not_audited(self):
        self.metadata["packages"].append({"id": "external", "name": "external", "source": "registry+https://example.invalid", "license": "LicenseRef-Private", "manifest_path": "/outside/Cargo.toml"})
        count, errors = CHECK.check_metadata(self.metadata)
        self.assertEqual((count, errors), (2, []))

    # 누락·금지·잘못된 SPDX를 각각 거절한다.
    def test_missing_or_disallowed_member_license(self):
        for value in (None, "GPL-2.0-only", "MIT OR"):
            with self.subTest(value=value):
                self.metadata["packages"][0]["license"] = value
                count, errors = CHECK.check_metadata(self.metadata)
                self.assertEqual(count, 2)
                self.assertTrue(any("local-a" in error for error in errors))

    # 워크스페이스 밖 manifest나 source가 있는 구성 요소는 내부 crate로 간주하지 않는다.
    def test_outside_or_nonlocal_workspace_member_is_rejected(self):
        self.metadata["packages"][0]["manifest_path"] = str(self.root.parent / "outside/Cargo.toml")
        self.assertTrue(CHECK.check_metadata(self.metadata)[1])
        self.metadata["packages"][0]["manifest_path"] = str(self.root / "crates/a/Cargo.toml")
        self.metadata["packages"][0]["source"] = "git+https://example.invalid/repo"
        self.assertTrue(CHECK.check_metadata(self.metadata)[1])

    # 구성 요소 목록이 일부 빠진 metadata를 성공으로 처리하지 않는다.
    def test_missing_workspace_member(self):
        self.metadata["workspace_members"].append("missing")
        self.assertTrue(any("missing" in error for error in CHECK.check_metadata(self.metadata)[1]))

    # GPL 전문의 CRLF는 허용하지만 누락·변조된 본문은 거절한다.
    def test_gpl_text_preserved(self):
        path = self.root / "LICENSE"
        text = path.read_text(encoding="utf-8")
        path.write_bytes(text.replace("\n", "\r\n").encode("utf-8"))
        self.assertEqual(CHECK.check_metadata(self.metadata)[1], [])
        path.write_text(text[:-100], encoding="utf-8")
        self.assertTrue(any("LICENSE" in error for error in CHECK.check_metadata(self.metadata)[1]))

    # 원저작 고지 제거도 검사 실패로 처리한다.
    def test_original_notices_required(self):
        (self.root / "assets/licenses.md").write_text("Don Editor only", encoding="utf-8")
        self.assertTrue(any("GramEditor/Zed" in error for error in CHECK.check_metadata(self.metadata)[1]))

    # 저장된 metadata 입력은 Cargo를 실행하지 않고 성공/오류 종료 코드를 반환한다.
    def test_cli_metadata_file_and_invalid_json(self):
        path = self.root / "metadata.json"
        path.write_text(json.dumps(self.metadata), encoding="utf-8")
        environment = dict(os.environ, PYTHONIOENCODING="utf-8")
        command = [sys.executable, str(REPOSITORY / "script/check_don_licenses.py"), "--metadata", str(path)]
        result = subprocess.run(command, capture_output=True, encoding="utf-8", env=environment)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("외부 Cargo 의존성", result.stdout)
        path.write_text("broken JSON", encoding="utf-8")
        result = subprocess.run(command, capture_output=True, encoding="utf-8", env=environment)
        self.assertEqual(result.returncode, 1)


# 직접 실행과 unittest 탐색 실행을 지원한다.
if __name__ == "__main__":
    unittest.main()
