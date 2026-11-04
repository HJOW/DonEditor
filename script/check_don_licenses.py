#!/usr/bin/env python3
"""Don Editor 내부 워크스페이스의 SPDX 선언과 보존한 원저작 고지를 검사한다.

Python 3.8+ 표준 라이브러리만 사용한다. 외부 Cargo 의존성의 라이선스는 검사하지 않는다.
Windows: python script/check_don_licenses.py
Linux/macOS: bash script/check-don-licenses
CI/검증: --metadata <cargo metadata JSON 파일>
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


# 원본에서 가져온 GPL 전문의 UTF-8/BOM 제거·LF 정규화 SHA256이다.
GPL_TEXT_SHA256 = "0aa0322ea494da441088e4d9144c6720bf99d9e2e1eefcd29d1f58b19b5c4246"
# 내부 Rust 코드에 허용하는 GPLv3 호환 라이선스 선택지이다. 그 밖의 선언은 별도 검토한다.
ALLOWED_LICENSES = frozenset(("GPL-3.0-only", "GPL-3.0-or-later", "MIT", "Apache-2.0"))
# AND/OR와 괄호를 지원한다. WITH 예외와 LicenseRef는 자동 승인하지 않는다.
SPDX_TOKEN = re.compile(r"[A-Za-z0-9][A-Za-z0-9.+-]*|[()]")


# SPDX 식의 구문을 검증하고, 배포에 사용할 수 있는 허용 선택지가 있는지 판정한다.
def license_allowed(expression):
    if not isinstance(expression, str) or not expression.strip():
        raise ValueError("SPDX license 선언이 없습니다")
    tokens = []
    position = 0
    # 공백 외의 문자를 빠짐없이 토큰으로 읽어 잘못된 구분자를 거부한다.
    while position < len(expression):
        if expression[position].isspace():
            position += 1
            continue
        match = SPDX_TOKEN.match(expression, position)
        if not match:
            raise ValueError("지원하지 않는 SPDX 문자: " + expression[position:])
        tokens.append(match.group())
        position = match.end()
    cursor = 0

    # 괄호식 또는 개별 라이선스 식별자를 읽는다.
    def atom():
        nonlocal cursor
        if cursor >= len(tokens):
            raise ValueError("불완전한 SPDX 식입니다")
        token = tokens[cursor]
        cursor += 1
        if token == "(":
            allowed = disjunction()
            if cursor >= len(tokens) or tokens[cursor] != ")":
                raise ValueError("SPDX 괄호가 닫히지 않았습니다")
            cursor += 1
            return allowed
        if token in ("AND", "OR", "WITH", ")"):
            raise ValueError("예상하지 못한 SPDX 토큰: " + token)
        return token in ALLOWED_LICENSES

    # AND는 모든 구성 요소가 허용되어야 한다. 단락 평가로 구문 검사를 건너뛰지 않는다.
    def conjunction():
        nonlocal cursor
        allowed = atom()
        while cursor < len(tokens) and tokens[cursor] == "AND":
            cursor += 1
            right = atom()
            allowed = allowed and right
        return allowed

    # OR는 허용되는 라이선스 선택지가 하나 이상이면 된다.
    def disjunction():
        nonlocal cursor
        allowed = conjunction()
        while cursor < len(tokens) and tokens[cursor] == "OR":
            cursor += 1
            right = conjunction()
            allowed = allowed or right
        return allowed

    allowed = disjunction()
    if cursor != len(tokens):
        raise ValueError("지원하지 않거나 잘못된 SPDX 식: " + " ".join(tokens[cursor:]))
    return allowed


# pathlib.is_relative_to가 없는 Python 3.8에서도 저장소 경계를 검사한다.
def inside_workspace(path, root):
    try:
        path.resolve().relative_to(root.resolve())
        return True
    except ValueError:
        return False


# Cargo가 상속을 해결한 metadata만 검사하여 TOML 자체 해석이나 외부 의존성 조회를 피한다.
def check_metadata(metadata):
    if not isinstance(metadata, dict):
        raise ValueError("metadata는 JSON 객체여야 합니다")
    root_value = metadata.get("workspace_root")
    packages = metadata.get("packages")
    members = metadata.get("workspace_members")
    if not isinstance(root_value, str) or not isinstance(packages, list) or not isinstance(members, list) or not members:
        raise ValueError("workspace_root/packages/workspace_members가 올바르지 않습니다")
    root = Path(root_value).resolve()
    errors = []
    # 원본 GPL 전문이 일부만 남거나 다른 라이선스로 바뀌는 회귀를 차단한다.
    try:
        license_text = (root / "LICENSE").read_text(encoding="utf-8-sig").replace("\r\n", "\n")
        if hashlib.sha256(license_text.encode("utf-8")).hexdigest() != GPL_TEXT_SHA256:
            errors.append("루트 LICENSE가 보존한 원본 GPL-3.0 전문과 다릅니다")
    except (OSError, UnicodeError) as error:
        errors.append("루트 LICENSE를 읽을 수 없습니다: " + str(error))
    # 원본 저작권과 파생 관계를 별도 고지에서도 유지한다.
    try:
        notices = (root / "assets/licenses.md").read_text(encoding="utf-8-sig")
        if "Copyright (C) 2026 The Gram Project" not in notices or "Zed" not in notices:
            errors.append("assets/licenses.md에 GramEditor/Zed 원저작 고지가 없습니다")
    except (OSError, UnicodeError) as error:
        errors.append("원저작 고지를 읽을 수 없습니다: " + str(error))
    by_id = {
        package.get("id"): package for package in packages
        if isinstance(package, dict) and isinstance(package.get("id"), str)
    }
    checked = 0
    # workspace_members에 명시된 로컬 crate만 검사하며 registry/git 의존성은 순회하지 않는다.
    for member in members:
        if not isinstance(member, str) or member not in by_id:
            errors.append("워크스페이스 구성 요소 metadata가 누락되었습니다: " + str(member))
            continue
        package = by_id[member]
        name = package.get("name", member)
        manifest = package.get("manifest_path")
        if package.get("source") is not None or not isinstance(manifest, str) or not inside_workspace(Path(manifest), root):
            errors.append(str(name) + ": 로컬 워크스페이스 내부 manifest가 아닙니다")
            continue
        checked += 1
        try:
            if not license_allowed(package.get("license")):
                errors.append(str(name) + ": 허용 선택지가 없는 SPDX 선언: " + str(package.get("license")))
        except ValueError as error:
            errors.append(str(name) + ": " + str(error))
    return checked, errors


# 기본 경로는 이 파일이 속한 저장소이며 다른 디렉터리에서 호출해도 동일하게 동작한다.
def main(arguments=None):
    parser = argparse.ArgumentParser(description="내부 워크스페이스 SPDX/원본 GPL 검사. 외부 Cargo 의존성은 검사하지 않습니다.")
    parser.add_argument("--metadata", type=Path, help="기존 cargo metadata JSON 파일을 사용하여 Cargo 실행 생략")
    options = parser.parse_args(arguments)
    try:
        if options.metadata:
            metadata = json.loads(options.metadata.read_text(encoding="utf-8-sig"))
        else:
            root = Path(__file__).resolve().parent.parent
            # 잠금 파일 변경과 네트워크 다운로드 없이 기존 Cargo로 상속된 선언을 읽는다.
            result = subprocess.run(
                ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked", "--offline"],
                cwd=str(root), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                encoding="utf-8", check=True,
            )
            metadata = json.loads(result.stdout)
        checked, errors = check_metadata(metadata)
    except (OSError, UnicodeError, ValueError, subprocess.CalledProcessError) as error:
        print("라이선스 검사 실패: " + str(error), file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr.strip(), file=sys.stderr)
        return 1
    if errors:
        # 실패 원인을 모두 표시해 한 번에 수정할 수 있게 한다.
        for error in errors:
            print("[실패] " + error, file=sys.stderr)
        return 1
    print("내부 워크스페이스 {}개 crate의 SPDX·원본 GPL·원저작 고지 확인 완료".format(checked))
    print("외부 Cargo 의존성 라이선스는 이 검사에 포함되지 않습니다. assets/licenses.md의 별도 고지를 참고하세요.")
    return 0


# 직접 실행할 때만 CLI를 시작한다. 테스트에서는 함수를 가져와 사용한다.
if __name__ == "__main__":
    if sys.version_info < (3, 8):
        print("Python 3.8 이상이 필요합니다.", file=sys.stderr)
        sys.exit(1)
    sys.exit(main())
