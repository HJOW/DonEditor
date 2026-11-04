"""실제 빌드/설치 없이 Linux/macOS 배포 흐름, 아카이브와 대응 소스를 검증한다.

Linux/macOS/WSL: python3 -m unittest discover -s tests -p test_package_tar.py -v
"""

import hashlib
from pathlib import Path
import plistlib
import shutil
import subprocess
import tarfile
import unittest

import test_install_build_tools as installer_tests


# 실제 컴파일 대신 호스트 타깃 경로에 실행 가능한 테스트 프로그램을 만든다.
FAKE_CARGO = r'''#!/bin/bash
printf 'cargo %s\n' "$*" >> "$TEST_MUTATIONS"
[[ "$TEST_FAIL" != cargo ]] || exit 42
target=''; profile=debug
while (( $# )); do
    case "$1" in --target) target=$2; shift ;; --release) profile=release ;; esac
    shift
done
directory="$PWD/target/$target/$profile"
mkdir -p "$directory"
# 테스트의 바이너리는 GUI를 띄우지 않고 실행/권한 보존 여부만 알린다.
for name in doneditor cli don-mcp remote_server; do
    printf '#!/bin/bash\nprintf "Don Editor test\\n"\n' > "$directory/$name"
    chmod +x "$directory/$name"
done
'''


# 패키지 관리자/컴파일러는 모두 모의 구현이며 tar/git/hash는 실제 도구를 쓴다.
class PackageTarTests(unittest.TestCase):
    # 기존 설치 테스트의 격리 PATH와 가짜 패키지 DB를 재사용한다.
    def setUp(self):
        self.fixture = installer_tests.InstallerShellTests(methodName="runTest")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.bin = self.fixture.bin
        self.environment = self.fixture.environment
        repository = Path(__file__).resolve().parents[1]
        shutil.copyfile(repository / "package-tar.sh", self.root / "package-tar.sh")
        # 실제 압축/복사/체크섬 도구만 허용한다.
        for name in ("cp", "mkdir", "chmod", "ln", "cat", "mv", "tar", "gzip", "sha256sum", "git"):
            path = self.bin / name
            path.unlink(missing_ok=True)
            path.symlink_to(shutil.which(name))
        (self.bin / "cargo").write_text(FAKE_CARGO)
        (self.bin / "cargo").chmod(0o755)
        self.environment.update(TEST_PACKAGES_READY="1", TEST_COMMANDS_READY="1", TEST_RUSTUP_READY="1", TEST_TOOLCHAIN_READY="1")
        self.write("Cargo.toml", '[workspace.package]\nversion = "0.1.0"\n')
        self.write("crates/gram/RELEASE_CHANNEL", "stable\n")
        self.write("LICENSE", "GPL-3.0 test\n")
        self.write("README.md", "Don Editor test\n")
        self.write("assets/licenses.md", "GramEditor/Zed notices\n")
        self.write("assets/licenses/MIT", "MIT test\n")
        self.write("assets/fonts/D2Coding-LICENSE", "OFL test\n")
        self.write("assets/images/logo_dark.svg", "<svg/>\n")
        self.write("crates/gram/resources/app-icon.png", "test icon\n")
        # 실제 plist 조각으로 .app 메타데이터의 결합도 검사한다.
        for path in (repository / "crates/gram/resources/info").glob("*.plist"):
            self.write("crates/gram/resources/info/" + path.name, path.read_text())
        self.write("tracked.rs", "// 최초 소스\n")
        self.write(".gitignore", "bin/\nstate/\n*.app/\nfake-tool\nmutations\nqueries\ndist/\ntarget/\n.env\n")
        subprocess.run([str(self.bin / "git"), "init", "--quiet", str(self.root)], check=True, capture_output=True)
        subprocess.run([str(self.bin / "git"), "-C", str(self.root), "add", "."], check=True, capture_output=True)

    # 공백을 포함하는 테스트 저장소에 파일을 만든다.
    def write(self, name, value):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value)

    # 패키징 명령의 출력/설치 로그를 수집한다.
    def run_package(self, answer="", *arguments):
        result = subprocess.run([str(self.bin / "bash"), str(self.root / "package-tar.sh"), *arguments], input=answer, text=True, capture_output=True, env=self.environment, timeout=30)
        self.assertNotIn("command not found", result.stderr)
        log = self.fixture.mutations.read_text() if self.fixture.mutations.exists() else ""
        return result, log

    # 생성된 tar.gz를 실제로 풀고 체크섬과 라이선스 파일을 확인한다.
    def unpack(self, os_name="linux", suffix="", out_dir=None):
        output = out_dir or self.root / "dist"
        name = f"DonEditor-0.1.0-{os_name}-x86_64{suffix}"
        archive = output / (name + ".tar.gz")
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        self.assertEqual((output / (name + ".tar.gz.sha256")).read_text(), f"{digest} *{name}.tar.gz\n")
        extraction = self.root / "unpacked"
        extraction.mkdir(exist_ok=True)
        with tarfile.open(archive) as bundle:
            bundle.extractall(extraction, filter="fully_trusted")
        package = extraction / name
        for path in ("LICENSE", "licenses.md", "licenses/MIT", "licenses/D2Coding-LICENSE", "SOURCE.txt", "source/DonEditor-0.1.0-source.tar.gz", "bin/don-mcp"):
            self.assertTrue((package / path).is_file(), path)
        self.assertFalse(list(output.glob(".doneditor-package.*")))
        return package

    # Linux CLI의 호환 경로와 symlink, 실행 권한을 실제 압축 해제 후 확인한다.
    def test_linux_build_archive_hash_and_launch(self):
        result, log = self.run_package()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("--package mcp_server", log)
        self.assertIn("--package remote_server", log)
        package = self.unpack()
        self.assertTrue((package / "bin/doneditor").is_symlink())
        for name in ("bin/doneditor", "bin/don-mcp", "libexec/gram-editor", "libexec/remote_server"):
            run = subprocess.run([str(package / name), "--version"], capture_output=True, text=True)
            self.assertEqual(run.returncode, 0)
            self.assertEqual(run.stdout, "Don Editor test\n")

    # source 아카이브에는 HEAD가 아닌 미커밋 수정/추가 내용과 공백 파일명이 들어간다.
    def test_sources_match_working_tree_and_ignore_secrets(self):
        self.write("tracked.rs", "// 수정된 한국어 소스\n")
        self.write("new source.rs", "// 새 소스\n")
        self.write(".env", "API_KEY=secret\n")
        result, _ = self.run_package()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        package = self.unpack()
        with tarfile.open(package / "source/DonEditor-0.1.0-source.tar.gz") as source:
            prefix = "DonEditor-0.1.0-source/"
            self.assertEqual(source.extractfile(prefix + "tracked.rs").read().decode(), "// 수정된 한국어 소스\n")
            self.assertEqual(source.extractfile(prefix + "new source.rs").read().decode(), "// 새 소스\n")
            self.assertNotIn(prefix + ".env", source.getnames())
            self.assertFalse(any(name.startswith(prefix + "dist/") for name in source.getnames()))

    # macOS는 .app, plist, 원본 CLI 상대 경로를 보존한다. 실제 Cocoa 실행 테스트는 아니다.
    def test_macos_bundle_plist_and_cli_layout(self):
        self.environment.update(TEST_OS="Darwin", TEST_XCODE_READY="1", TEST_METAL_READY="1")
        self.fixture.use_manager("brew")
        result, _ = self.run_package()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        package = self.unpack("macos")
        contents = package / "Don Editor.app/Contents"
        plist = plistlib.loads((contents / "Info.plist").read_bytes())
        self.assertEqual(plist["CFBundleDisplayName"], "Don Editor")
        self.assertEqual(plist["CFBundleExecutable"], "gram")
        self.assertEqual(plist["CFBundleIdentifier"], "app.doneditor.DonEditor")
        self.assertEqual(plist["CFBundleShortVersionString"], "0.1.0")
        self.assertIn("CFBundleDocumentTypes", plist)
        self.assertTrue((package / "bin/doneditor").is_symlink())
        self.assertTrue((contents / "MacOS/remote_server").is_file())

    # 누락 도구를 거절하거나 EOF를 보내면 빌드 전에 종료한다.
    def test_cancel_and_eof_stop_before_build(self):
        self.environment["TEST_PACKAGES_READY"] = "0"
        for answer in ("", "\n", "0\n"):
            with self.subTest(answer=answer):
                result, log = self.run_package(answer)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertEqual(log, "")
                self.assertFalse((self.root / "dist").exists())

    # 사용자가 지정한 항목만 설치하고 재검사를 통과해야 빌드로 진행한다.
    def test_explicit_install_and_recheck(self):
        self.environment["TEST_PACKAGES_READY"] = "0"
        result, log = self.run_package("A\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("libdbus-1-dev", log)
        self.assertLess(log.index("apt-get install"), log.index("cargo build"))

    # 실패한 설치를 완료로 보고하지 않는다.
    def test_failed_install_stops_build(self):
        self.environment.update(TEST_PACKAGES_READY="0", TEST_FAIL="apt-get")
        result, log = self.run_package("A\nI\n")
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("cargo", log)
        self.assertFalse((self.root / "dist").exists())

    # macOS의 Xcode 외부 설치 미완료 종료 코드를 보존한다.
    def test_macos_pending_install_stops_build(self):
        self.environment.update(TEST_OS="Darwin", TEST_XCODE_READY="0")
        self.fixture.use_manager("brew")
        result, log = self.run_package("1\nI\n")
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("open macappstore", log)
        self.assertNotIn("cargo", log)

    # macOS에서도 취소/EOF는 재검사를 통과하지 못하므로 빌드하지 않는다.
    def test_macos_cancel_and_eof_stop_build(self):
        self.environment.update(TEST_OS="Darwin", TEST_XCODE_READY="0")
        self.fixture.use_manager("brew")
        for answer in ("", "0\n"):
            with self.subTest(answer=answer):
                result, log = self.run_package(answer)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertEqual(log, "")

    # macOS의 선택 설치 후 상태 갱신도 실제 패키징 진입점에서 검증한다.
    def test_macos_selected_cmake_install_then_build(self):
        self.environment.update(TEST_OS="Darwin", TEST_XCODE_READY="1", TEST_METAL_READY="1", TEST_COMMANDS_READY="0")
        self.fixture.use_manager("brew")
        result, log = self.run_package("4\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("brew install cmake", log)
        self.assertLess(log.index("brew install"), log.index("cargo build"))

    # 조회와 설치 모의 실행에서는 빌드/압축/설치 명령을 실행하지 않는다.
    def test_check_only_and_dry_run(self):
        for arguments, answer in ((("--check-only",), ""), (("--dry-run",), "A\nI\n0\n")):
            with self.subTest(arguments=arguments):
                result, log = self.run_package(answer, *arguments)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual(log, "")
                self.assertFalse((self.root / "dist").exists())

    # 컴파일 오류는 패키징 단계로 진행하지 않는다.
    def test_build_failure(self):
        self.environment["TEST_FAIL"] = "cargo"
        result, _ = self.run_package()
        self.assertEqual(result.returncode, 42)
        self.assertFalse(list((self.root / "dist").glob("*.tar.gz")))

    # skip-build에서도 MCP 등 필수 산출물이 없으면 실패한다.
    def test_missing_binary(self):
        result, log = self.run_package("", "--skip-build", "--skip-check")
        self.assertEqual(result.returncode, 1)
        self.assertIn("필수 산출물", result.stderr)
        self.assertEqual(log, "")

    # 디버그/사용자 출력 경로는 릴리스 산출물과 구분하고 경로 공백을 보존한다.
    def test_debug_and_custom_output_path(self):
        output = self.root / "custom output"
        result, log = self.run_package("", "--debug", "--out-dir", str(output))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn("--release", log)
        self.unpack(suffix="-debug", out_dir=output)

    # 잘못된 옵션과 모순된 실행 모드를 환경 변경 전에 거부한다.
    def test_invalid_options(self):
        for arguments in (("--unknown",), ("--out-dir",), ("--skip-check", "--check-only")):
            with self.subTest(arguments=arguments):
                result, log = self.run_package("", *arguments)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(log, "")


# 직접 실행 및 unittest 탐색을 모두 지원한다.
if __name__ == "__main__":
    unittest.main()
