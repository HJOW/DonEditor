#!/usr/bin/env bash
# [DonEditor] Linux/macOS 배포본을 만든다. GPL-3.0-or-later (LICENSE 참고).
# macOS 기본 Bash 3.2와 Linux Bash를 함께 지원한다.
set -euo pipefail

# 저장소, 실행 옵션, 이번 실행에서만 사용하는 스테이징 디렉터리이다.
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
out_dir="$repo_root/dist"
profile=release
skip_check=0
skip_build=0
check_only=0
dry_run=0
stage=''

# 설치는 번호 선택 메뉴에서 사용자가 확정한 항목에만 수행한다.
usage() {
    printf '%s\n' '사용법: bash package-tar.sh [옵션]' \
        '  --check-only    도구 준비 상태만 조회' \
        '  --dry-run       도구 설치 메뉴의 명령만 표시 (빌드/압축 안 함)' \
        '  --skip-check    준비된 CI/개발 환경에서 도구 검사 생략' \
        '  --skip-build    기존 target/<호스트>/<프로파일> 산출물 사용' \
        '  --debug         릴리스 대신 디버그 빌드' \
        '  --out-dir 경로  결과물 위치 (상대 경로는 저장소 기준)' \
        '  --help          도움말' \
        '배포본에 현재 작업 트리의 소스, 라이선스 및 SHA256을 함께 제공합니다.'
}

# 옵션 파싱은 설치/환경 조회보다 먼저 끝낸다.
while (( $# )); do
    case "$1" in
        --help|-h) usage; exit 0 ;;
        --check-only) check_only=1 ;;
        --dry-run) dry_run=1 ;;
        --skip-check) skip_check=1 ;;
        --skip-build) skip_build=1 ;;
        --debug) profile=debug ;;
        --out-dir)
            (( $# >= 2 )) || { printf '결과물 경로가 필요합니다.\n' >&2; exit 1; }
            out_dir=$2; shift ;;
        *) printf '알 수 없는 옵션: %s\n' "$1" >&2; usage; exit 1 ;;
    esac
    shift
done
if (( skip_check && (check_only || dry_run) )); then
    printf '%s\n' '--skip-check는 --check-only/--dry-run과 함께 사용할 수 없습니다.' >&2
    exit 1
fi

# 종료할 때 이 실행이 생성한 임시 디렉터리만 지운다.
cleanup() {
    if [[ -n "$stage" && "$stage" == "$out_dir"/.doneditor-package.* && -d "$stage" ]]; then
        rm -rf -- "$stage"
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# 지원 OS와 네이티브 Rust 타깃을 정한다. 이 스크립트는 교차 빌드를 하지 않는다.
platform=$(uname -s)
case "$(uname -m)" in
    x86_64|amd64) architecture=x86_64 ;;
    aarch64|arm64) architecture=aarch64 ;;
    *) printf '지원하지 않는 CPU입니다 (x64/ARM64 필요).\n' >&2; exit 1 ;;
esac
case "$platform" in
    Linux) os_name=linux; target="$architecture-unknown-linux-gnu" ;;
    Darwin) os_name=macos; target="$architecture-apple-darwin" ;;
    *) printf 'Linux/macOS에서 실행하세요.\n' >&2; exit 1 ;;
esac

# 번호 선택 설치 스크립트가 상세 검사와 명시적 설치 동의를 공통 처리한다.
if (( ! skip_check )); then
    if (( check_only )); then exec bash "$repo_root/install-build-tools.sh" --check-only; fi
    if (( dry_run )); then exec bash "$repo_root/install-build-tools.sh" --dry-run; fi
    if ! bash "$repo_root/install-build-tools.sh" --check-only; then
        printf '\n빌드 도구가 부족합니다. 설치할 번호를 선택하고 I로 확정하세요. 취소하면 빌드를 중단합니다.\n'
        bash "$repo_root/install-build-tools.sh" || exit $?
        # 취소/EOF는 설치 스크립트에서 정상 종료이므로 준비 상태를 반드시 다시 검사한다.
        if ! bash "$repo_root/install-build-tools.sh" --check-only; then
            printf '필수 도구가 준비되지 않아 빌드 전에 중단합니다.\n' >&2
            exit 1
        fi
    fi
fi

# 설치 후 현재 세션에도 표준 Rust/Homebrew 경로를 반영한다.
for directory in "${CARGO_HOME:-$HOME/.cargo}/bin" /opt/homebrew/bin /usr/local/bin; do
    [[ ! -d "$directory" || ":$PATH:" == *":$directory:"* ]] || PATH="$directory:$PATH"
done
export PATH
cd -- "$repo_root"
version=$(awk '/^\[workspace.package\]/{active=1;next} active && /^\[/{exit} active && /^version[[:space:]]*=/{gsub(/"/, "", $3); print $3; exit}' Cargo.toml)
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+[-+a-zA-Z0-9.]*$ ]] || { printf 'workspace 버전을 읽지 못했습니다.\n' >&2; exit 1; }
channel=$(tr -d '\r\n' < crates/gram/RELEASE_CHANNEL)
# 이 tar 배포본은 시스템 Git을 사용하므로 macOS의 별도 Git 내장 모드는 사용하지 않는다.
export RELEASE_CHANNEL="$channel" GRAM_RELEASE_CHANNEL="$channel" RELEASE_VERSION="$version" GRAM_BUNDLE=false
suffix=''
[[ "$profile" != debug ]] || suffix=-debug
package_name="DonEditor-$version-$os_name-$architecture$suffix"
[[ "$out_dir" == /* ]] || out_dir="$repo_root/$out_dir"
mkdir -p -- "$out_dir"
out_dir=$(cd -- "$out_dir" && pwd)
target_dir=${CARGO_TARGET_DIR:-"$repo_root/target"}
[[ "$target_dir" == /* ]] || target_dir="$repo_root/$target_dir"
binary_dir="$target_dir/$target/$profile"

# GUI/CLI/MCP를 함께 빌드하고 원격 서버는 기능 통합을 피하려고 별도로 빌드한다.
if (( ! skip_build )); then
    build_args=(build --locked --target "$target" --package gram --package cli --package mcp_server)
    [[ "$profile" != release ]] || build_args+=(--release)
    cargo "${build_args[@]}"
    remote_args=(build --locked --target "$target" --package remote_server)
    [[ "$profile" != release ]] || remote_args+=(--release)
    cargo "${remote_args[@]}"
fi
# 누락된 바이너리를 가진 부분적인 배포본을 만들지 않는다.
for binary in doneditor cli don-mcp remote_server; do
    [[ -f "$binary_dir/$binary" ]] || { printf '필수 산출물이 없습니다: %s\n' "$binary_dir/$binary" >&2; exit 1; }
done
stage=$(mktemp -d "$out_dir/.doneditor-package.XXXXXX")
package_dir="$stage/$package_name"
mkdir -p "$package_dir/bin" "$package_dir/licenses" "$package_dir/source"
cp LICENSE README.md assets/licenses.md "$package_dir/"
cp -R assets/licenses/. "$package_dir/licenses/"
cp assets/fonts/D2Coding-LICENSE "$package_dir/licenses/"
cp "$binary_dir/don-mcp" "$package_dir/bin/don-mcp"
chmod +x "$package_dir/bin/don-mcp"

# Linux는 원본 CLI의 상대 경로 계약을 보존하고 doneditor라는 진입점을 추가한다.
if [[ "$platform" == Linux ]]; then
    mkdir -p "$package_dir/libexec" "$package_dir/share/applications" "$package_dir/share/icons/hicolor/scalable/apps"
    cp "$binary_dir/doneditor" "$package_dir/libexec/gram-editor"
    cp "$binary_dir/cli" "$package_dir/bin/gram"
    cp "$binary_dir/remote_server" "$package_dir/libexec/remote_server"
    chmod +x "$package_dir/libexec/gram-editor" "$package_dir/libexec/remote_server" "$package_dir/bin/gram"
    ln -s gram "$package_dir/bin/doneditor"
    cp assets/images/logo_dark.svg "$package_dir/share/icons/hicolor/scalable/apps/doneditor.svg"
    cat > "$package_dir/share/applications/doneditor.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Don Editor
Comment=텍스트 및 소스 코드 편집기
Exec=doneditor %F
Icon=doneditor
Terminal=false
Categories=Development;TextEditor;
MimeType=text/plain;inode/directory;
DESKTOP
else
    # macOS는 기본 Launch Services와 원본 CLI가 인식하는 .app 구조를 사용한다.
    contents="$package_dir/Don Editor.app/Contents"
    bundle_id=app.doneditor.DonEditor
    [[ "$channel" != dev ]] || bundle_id=app.doneditor.DonEditor-Dev
    mkdir -p "$contents/MacOS" "$contents/Resources"
    cp "$binary_dir/doneditor" "$contents/MacOS/gram"
    cp "$binary_dir/cli" "$contents/MacOS/cli"
    cp "$binary_dir/remote_server" "$contents/MacOS/remote_server"
    chmod +x "$contents/MacOS/gram" "$contents/MacOS/cli" "$contents/MacOS/remote_server"
    ln -s '../Don Editor.app/Contents/MacOS/cli' "$package_dir/bin/doneditor"
    # 기존 원본 아이콘도 보존하되 번들 표시 이름은 Don Editor로 지정한다.
    cp crates/gram/resources/app-icon.png "$contents/Resources/app-icon.png"
    cat > "$contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Don Editor</string>
<key>CFBundleDisplayName</key><string>Don Editor</string>
<key>CFBundleIdentifier</key><string>$bundle_id</string>
<key>CFBundleExecutable</key><string>gram</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>$version</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundleIconFile</key><string>app-icon.png</string>
<key>NSHighResolutionCapable</key><true/>
PLIST
    # 파일 연결/권한/지원 플랫폼 등 기존 번들 설정을 함께 제공한다.
    for fragment in crates/gram/resources/info/*.plist; do sed 's/Gram/Don Editor/g' "$fragment" >> "$contents/Info.plist"; done
    printf '</dict></plist>\n' >> "$contents/Info.plist"
fi

# git archive HEAD 대신 현재 파일 내용을 복사하므로 미커밋 수정과 새 소스도 포함한다.
# gitignore 대상(비밀키/target/dist 등)은 제외하며 별도 출력 경로의 생성물도 제외한다.
source_name="DonEditor-$version-source"
source_tree="$stage/$source_name"
mkdir -p "$source_tree"
git ls-files --cached --others --exclude-standard -z > "$stage/source-files"
source_count=0
while IFS= read -r -d '' relative; do
    case "$relative" in /*|../*|*/../*|.git/*|target/*|dist/*) continue ;; esac
    [[ "$repo_root/$relative" != "$out_dir"/* ]] || continue
    [[ -f "$relative" || -L "$relative" ]] || continue
    mkdir -p "$source_tree/$(dirname -- "$relative")"
    cp -pP -- "$relative" "$source_tree/$relative"
    source_count=$((source_count+1))
done < "$stage/source-files"
(( source_count > 0 )) || { printf '소스 파일 목록이 비어 있습니다. git 저장소에서 실행하세요.\n' >&2; exit 1; }
tar -czf "$package_dir/source/$source_name.tar.gz" -C "$stage" "$source_name"
commit=$(git rev-parse HEAD 2>/dev/null || printf '없음')
cat > "$package_dir/SOURCE.txt" <<SOURCE
Don Editor $version ($channel), $os_name-$architecture, $profile
기준 커밋: $commit
이 배포본의 프로젝트 소스는 source/$source_name.tar.gz에 들어 있습니다.
마지막 커밋만이 아니라 패키징 시점의 작업 트리 내용을 포함합니다.
Cargo.lock/rust-toolchain.toml과 빌드 스크립트가 함께 제공됩니다.
프로젝트 README와 docs/packaging.md를 참고해 빌드할 수 있습니다.
GPL-3.0-or-later 전문: LICENSE. 원저작(GramEditor/Zed), 제3자 및 글꼴: licenses.md, licenses/.
원본: https://codeberg.org/GramEditor/gram
Linux는 빌드한 배포판과 호환되는 glibc/GUI 시스템 라이브러리가 필요합니다.
Git 기능에는 시스템에 설치된 Git이 필요합니다 (macOS 포함).
macOS 배포본은 서명/공증되지 않았습니다. 정식 외부 배포 시 배포자가 서명/공증해야 합니다.
SOURCE

# 원자적 이름 변경으로 완성된 압축 파일만 최종 이름에 배치한다.
archive="$out_dir/$package_name.tar.gz"
tar -czf "$stage/package.tar.gz" -C "$stage" "$package_name"
mv -f -- "$stage/package.tar.gz" "$archive"
if command -v sha256sum >/dev/null 2>&1; then
    digest=$(sha256sum "$archive"); digest=${digest%% *}
else
    digest=$(shasum -a 256 "$archive"); digest=${digest%% *}
fi
printf '%s *%s\n' "$digest" "$package_name.tar.gz" > "$archive.sha256"
printf '완료: %s\nSHA256: %s\n' "$archive" "$digest"
