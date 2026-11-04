"""선택 설치 셸 스크립트를 가짜 도구로 검증한다. 실제 설치/네트워크 요청은 없다.

Linux/macOS 또는 WSL에서 python3 -m unittest discover -s tests -p test_install_build_tools.py
"""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


# 실제 시스템 명령 대신 읽기 전용 조회와 모의 설치를 제공한다.
FAKE_TOOL = r'''#!/bin/bash
tool=${0##*/}
printf '%s %s\n' "$tool" "$*" >> "$TEST_QUERIES"
host=x86_64-unknown-linux-gnu
[[ "$TEST_OS" != Darwin ]] || host=x86_64-apple-darwin
# 테스트 전용 설치 로그를 기록하며, 설정된 실패를 그대로 돌려준다.
mutate() {
    printf '%s %s\n' "$tool" "$*" >> "$TEST_MUTATIONS"
    [[ "$TEST_FAIL" != "$tool" ]] || return 42
}
# 선택한 패키지만 가짜 설치 DB에 추가한다.
mark_packages() {
    local package
    for package in "$@"; do
        [[ "$package" == -* ]] || touch "$TEST_STATE/$package"
    done
}
case "$tool" in
    uname) if [[ "$1" == -s ]]; then echo "$TEST_OS"; else echo x86_64; fi ;;
    id) echo 1000 ;;
    sudo) "$@" ;;
    dpkg-query)
        package=${@: -1}
        [[ "$TEST_PACKAGES_READY" == 1 || -f "$TEST_STATE/$package" ]] || exit 1
        printf 'install ok installed' ;;
    rpm)
        [[ "$TEST_PACKAGES_READY" == 1 || -f "$TEST_STATE/$2" ]] || exit 1 ;;
    apt-get)
        mutate "$@" || exit $?
        if [[ "$1" == install ]]; then shift; mark_packages "$@"; fi ;;
    dnf|zypper)
        mutate "$@" || exit $?
        [[ "$1" != --non-interactive ]] || shift
        shift; mark_packages "$@" ;;
    pacman)
        if [[ "$1" == -Q ]]; then
            [[ "$TEST_PACKAGES_READY" == 1 || -f "$TEST_STATE/$2" ]] || exit 1
        else mutate "$@" || exit $?; shift; mark_packages "$@"; fi ;;
    git|cmake)
        [[ "$TEST_COMMANDS_READY" == 1 || -f "$TEST_STATE/$tool" ]] || exit 1
        echo "$tool version test" ;;
    curl) mutate "$@"; exit 42 ;;
    rustup)
        if [[ "$1 $2" == 'toolchain install' ]]; then
            mutate "$@" || exit $?
            touch "$TEST_STATE/toolchain"
        elif [[ "$1" == --version ]]; then
            # 일부 rustup 버전의 활성 툴체인 암묵 설치를 모사한다.
            mutate 'forbidden implicit toolchain download'
            exit 99
        elif [[ "$1" == --help ]]; then
            [[ "$TEST_RUSTUP_READY" == 1 ]] || exit 1
            echo 'rustup test'
        elif [[ "$1 $2" == 'toolchain list' ]]; then
            [[ "$TEST_TOOLCHAIN_READY" == 1 || -f "$TEST_STATE/toolchain" ]] || exit 0
            echo "1.99.1-$host"
        elif [[ "$1 $2" == 'component list' ]]; then
            # 설치 후에는 누락됐던 구성 요소도 준비됐다고 응답한다.
            for component in rustfmt clippy rust-analyzer; do
                if [[ "$component" != "$TEST_MISSING_COMPONENT" || -f "$TEST_STATE/toolchain" ]]; then echo "$component-$host"; fi
            done
        elif [[ "$1 $2" == 'target list' ]]; then
            [[ "$TEST_MISSING_TARGET" != 1 || -f "$TEST_STATE/toolchain" ]] || exit 0
            echo wasm32-wasip2
        else exit 1
        fi ;;
    xcode-select)
        if [[ "$1" == -p ]]; then
            [[ "$TEST_XCODE_READY" == 1 ]] || exit 1
            echo "$TEST_DEVELOPER"
        else mutate "$@"
        fi ;;
    xcodebuild)
        if [[ "$1" == -version ]]; then [[ "$TEST_XCODE_READY" == 1 ]] || exit 1; echo 'Xcode test'
        else mutate "$@" || exit $?; [[ "$1" != -downloadComponent ]] || touch "$TEST_STATE/metal"
        fi ;;
    xcrun) [[ "$TEST_METAL_READY" == 1 || -f "$TEST_STATE/metal" ]] ;;
    open) mutate "$@" ;;
    brew) mutate "$@" || exit $?; shift; mark_packages "$@" ;;
    *) exit 99 ;;
esac
'''


# 공백 경로의 복사본과 최소 PATH로 실제 패키지 관리자 접근을 차단한다.
class InstallerShellTests(unittest.TestCase):
    # 테스트마다 설치 DB와 로그를 분리한다.
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="doneditor test ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.state = self.root / "state"
        self.state.mkdir()
        repository = Path(__file__).resolve().parents[1]
        # 실제 스크립트와 툴체인 설정을 수정 없이 복사한다.
        for name in ("install-build-tools.sh", "rust-toolchain.toml"):
            shutil.copyfile(repository / name, self.root / name)
        # 셸 자체와 문자열/임시 파일 유틸리티만 실제 실행 파일로 연결한다.
        for name in ("bash", "sh", "dirname", "awk", "grep", "tr", "sed", "rm", "rmdir", "mktemp", "touch"):
            (self.bin / name).symlink_to(shutil.which(name))
        self.fake = self.root / "fake-tool"
        self.fake.write_text(FAKE_TOOL)
        self.fake.chmod(0o755)
        self.mutations = self.root / "mutations"
        self.queries = self.root / "queries"
        self.developer = self.root / "Xcode.app/Contents/Developer"
        self.developer.mkdir(parents=True)
        self.environment = {
            **os.environ,
            "PATH": str(self.bin) + ":/usr/local/bin",
            "HOME": str(self.root),
            "CARGO_HOME": str(self.root / "cargo"),
            "TEST_OS": "Linux",
            "TEST_QUERIES": str(self.queries),
            "TEST_MUTATIONS": str(self.mutations),
            "TEST_STATE": str(self.state),
            "TEST_DEVELOPER": str(self.developer),
        }
        # 모든 가짜 도구의 기본 응답은 미설치이다.
        for key in ("PACKAGES_READY", "COMMANDS_READY", "RUSTUP_READY", "TOOLCHAIN_READY", "MISSING_TARGET", "XCODE_READY", "METAL_READY"):
            self.environment["TEST_" + key] = "0"
        self.environment.update(TEST_FAIL="", TEST_MISSING_COMPONENT="")
        # 패키지 관리자는 테스트별로 한 종류만 노출한다.
        for name in ("uname", "id", "sudo", "dpkg-query", "rpm", "git", "cmake", "curl", "rustup", "xcode-select", "xcodebuild", "xcrun", "open"):
            (self.bin / name).symlink_to(self.fake)
        self.use_manager("apt-get")

    # 실제 관리자 대신 테스트에서 지정한 한 종류만 PATH에 넣는다.
    def use_manager(self, name):
        for manager in ("apt-get", "dnf", "pacman", "zypper", "brew"):
            (self.bin / manager).unlink(missing_ok=True)
        (self.bin / name).symlink_to(self.fake)

    # 입력을 파이프로 전달하고 설치 명령 로그를 수집한다.
    def run_installer(self, answer="", *arguments):
        result = subprocess.run(
            [str(self.bin / "bash"), str(self.root / "install-build-tools.sh"), *arguments],
            input=answer, text=True, capture_output=True, env=self.environment, timeout=20,
        )
        self.assertNotIn("command not found", result.stderr)
        log = self.mutations.read_text() if self.mutations.exists() else ""
        return result, log

    # 시작, EOF, 취소, 잘못된 입력만으로는 설치하지 않는다.
    def test_no_install_before_explicit_start(self):
        for answer in ("", "0\n", "\n", "1\n0\n", "A\nN\nI\n0\n", "1,999\nI\n0\n"):
            with self.subTest(answer=answer):
                result, log = self.run_installer(answer)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual(log, "")

    # 같은 번호를 다시 입력하면 선택을 해제한다.
    def test_toggle_off(self):
        result, log = self.run_installer("2\n2\nI\n0\n")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(log, "")
        self.assertIn("선택한 항목이 없습니다", result.stdout)

    # 설치하지 않은 항목은 선택된 패키지 설치 명령에 끼어들지 않는다.
    def test_only_selected_package_is_installed(self):
        result, log = self.run_installer("2,2\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(log.splitlines(), ["apt-get update", "apt-get install -y git"])

    # A 선택 후 N 해제는 전부 설치로 오해되지 않는다.
    def test_clear_then_choose(self):
        result, log = self.run_installer("A\nN\n3\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("install -y cmake", log)
        self.assertNotIn("git", log)

    # 준비된 도구는 재설치하지 않는다.
    def test_ready_items_are_skipped(self):
        self.environment["TEST_COMMANDS_READY"] = "1"
        result, log = self.run_installer("2\nI\n0\n")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(log, "")
        self.assertIn("이미 준비", result.stdout)

    # 점검 모드는 네트워크나 패키지 설치 없이 미준비 상태를 알린다.
    def test_check_only(self):
        result, log = self.run_installer("", "--check-only")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(log, "")
        self.assertIn("rust-analyzer", result.stdout)

    # 모든 구성 요소가 이미 있으면 점검 성공 코드를 반환한다.
    def test_check_all_ready(self):
        self.environment.update(TEST_PACKAGES_READY="1", TEST_COMMANDS_READY="1", TEST_RUSTUP_READY="1", TEST_TOOLCHAIN_READY="1")
        result, log = self.run_installer("", "--check-only")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(log, "")

    # rustup이 있어도 선택 전 버전 표시를 통한 암묵 설치를 유발하지 않는다.
    def test_rustup_presence_probe_does_not_inspect_active_compiler(self):
        self.environment["TEST_RUSTUP_READY"] = "1"
        result, log = self.run_installer("0\n")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(log, "")
        self.assertNotIn("rustup --version", self.queries.read_text())

    # 모의 실행은 관리자 권한 요청·다운로드를 포함해 아무것도 설치하지 않는다.
    def test_dry_run(self):
        result, log = self.run_installer("A\nI\n", "--dry-run")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(log, "")
        self.assertIn("--component rustfmt", result.stdout)
        self.assertIn("wasm32-wasip2", result.stdout)

    # rustup을 선택하지 않았으면 툴체인 설치를 시작하지 않는다.
    def test_missing_dependency(self):
        result, log = self.run_installer("7\nI\n0\n")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(log, "")
        self.assertIn("rustup 항목도 선택", result.stdout)

    # 이미 설치된 rustup은 선택하지 않아도 툴체인을 설치할 수 있다.
    def test_project_toolchain_and_components(self):
        self.environment["TEST_RUSTUP_READY"] = "1"
        result, log = self.run_installer("7\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(len(log.splitlines()), 1)
        self.assertIn("toolchain install 1.99.1-x86_64-unknown-linux-gnu", log)
        self.assertIn("--component rustfmt,clippy,rust-analyzer --target wasm32-wasip2", log)

    # rustfmt나 타깃 누락도 미준비로 인식해 복구한다.
    def test_missing_rust_component_or_target(self):
        self.environment.update(TEST_RUSTUP_READY="1", TEST_TOOLCHAIN_READY="1", TEST_MISSING_COMPONENT="rustfmt", TEST_MISSING_TARGET="1")
        result, log = self.run_installer("7\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("toolchain install", log)

    # 각 배포판용 명령과 GUI 라이브러리 패키지 매핑을 실제 호출 경로에서 확인한다.
    def test_distribution_package_commands(self):
        for manager, expected in (("apt-get", "libfontconfig-dev"), ("dnf", "fontconfig-devel"), ("pacman", "vulkan-icd-loader"), ("zypper", "libopenssl-devel")):
            with self.subTest(manager=manager):
                self.use_manager(manager)
                result, log = self.run_installer("5\nI\n")
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn(expected, log)
                self.mutations.unlink()

    # 설치 실패를 성공으로 보고하지 않으며 그 종료 코드를 전파한다.
    def test_install_failure(self):
        self.environment["TEST_FAIL"] = "apt-get"
        result, log = self.run_installer("2\nI\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("[실패]", result.stdout)
        self.assertEqual(log, "apt-get update\n")

    # macOS에서 Xcode가 없으면 조회 중 Git 스텁도 실행하지 않는다.
    def test_macos_no_developer_popup_before_selection(self):
        self.environment["TEST_OS"] = "Darwin"
        result, log = self.run_installer("0\n")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(log, "")
        self.assertNotIn("git --version", self.queries.read_text())

    # Xcode App Store를 연 것은 설치 완료가 아니므로 별도 종료 코드를 쓴다.
    def test_macos_xcode_external_install_is_pending(self):
        self.environment["TEST_OS"] = "Darwin"
        result, log = self.run_installer("1\nI\n")
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertEqual(log, "open macappstore://apps.apple.com/app/id497799835\n")
        self.assertIn("외부 설치", result.stdout)

    # 설치된 Xcode의 Metal 구성 요소만 빠졌을 때 선택해서 복구할 수 있다.
    def test_macos_metal_component(self):
        self.environment.update(TEST_OS="Darwin", TEST_XCODE_READY="1")
        result, log = self.run_installer("1\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("xcode-select --switch", log)
        self.assertIn("xcodebuild -downloadComponent MetalToolchain", log)

    # macOS에서 CMake만 선택하면 Homebrew 외의 도구를 설치하지 않는다.
    def test_macos_cmake(self):
        self.environment.update(TEST_OS="Darwin", TEST_XCODE_READY="1", TEST_METAL_READY="1")
        self.use_manager("brew")
        result, log = self.run_installer("4\nI\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(log, "brew install cmake\n")

    # macOS에서도 Homebrew를 미선택 상태로 몰래 설치하지 않는다.
    def test_macos_requires_selected_homebrew(self):
        self.environment["TEST_OS"] = "Darwin"
        result, log = self.run_installer("4\nI\n0\n")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(log, "")
        self.assertIn("Homebrew 항목도 선택", result.stdout)


# 직접 실행과 unittest 탐색 실행을 모두 지원한다.
if __name__ == "__main__":
    unittest.main()
