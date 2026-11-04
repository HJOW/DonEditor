#!/usr/bin/env bash
# [DonEditor] Linux/macOS 빌드 도구 선택 설치. GPL-3.0-or-later (LICENSE 참고).
# macOS 기본 Bash 3.2에서도 실행할 수 있도록 연관 배열과 mapfile을 사용하지 않는다.
set -o pipefail

# 저장소 위치, 실행 모드와 설치 중에만 사용할 임시 파일 경로이다.
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd) || exit 1
check_only=0
dry_run=0
download_directory=''
download_file=''

# 사용법을 표시한다. 옵션만으로 설치를 자동 승인하는 기능은 제공하지 않는다.
usage() {
    printf '%s\n' '사용법: bash install-build-tools.sh [--check-only | --dry-run | --help]' \
        '번호=선택 전환, A=미준비 전체 선택, N=전체 해제, I=선택 설치, 0=종료' \
        '--check-only: 상태 조회만. --dry-run: 번호 선택 후 설치 명령만 출력.'
}

# 옵션은 환경 조회 전에 검사하므로 도움말은 어떤 OS에서도 볼 수 있다.
for argument in "$@"; do
    case "$argument" in
        --check-only) check_only=1 ;;
        --dry-run) dry_run=1 ;;
        --help|-h) usage; exit 0 ;;
        *) printf '알 수 없는 옵션: %s\n' "$argument" >&2; usage; exit 1 ;;
    esac
done

# 현재 세션에서 표준 위치의 Rust/Homebrew를 찾되 사용자 설정 파일은 실행하지 않는다.
refresh_path() {
    local directory
    # 기존 PATH 밖에 있는 표준 설치 폴더만 추가한다.
    for directory in "${CARGO_HOME:-$HOME/.cargo}/bin" /opt/homebrew/bin /usr/local/bin; do
        if [[ -d "$directory" && ":$PATH:" != *":$directory:"* ]]; then PATH="$directory:$PATH"; fi
    done
    export PATH
    hash -r
}

# 외부 명령의 존재 여부만 조회한다.
has_command() { command -v "$1" >/dev/null 2>&1; }

# 이 스크립트가 만든 단일 다운로드 파일과 빈 디렉터리만 정리한다.
cleanup_download() {
    [[ -z "$download_file" ]] || rm -f -- "$download_file"
    [[ -z "$download_directory" ]] || rmdir -- "$download_directory"
    download_directory=''
    download_file=''
}
trap cleanup_download EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# 실행할 명령을 인용해 출력한다. 모의 실행에서는 sudo와 다운로드도 하지 않는다.
run() {
    printf '  >'; printf ' %q' "$@"; printf '\n'
    (( dry_run )) || "$@"
}

# 시스템 패키지 설치 시에만 관리자 권한을 요청한다.
run_admin() {
    if [[ $(id -u) -eq 0 ]]; then run "$@"
    elif has_command sudo; then run sudo "$@"
    else printf '관리자 권한이 필요합니다. sudo를 준비하거나 root로 실행하세요.\n' >&2; return 1
    fi
}

# 저장소 TOML의 문자열 배열을 줄 단위로 읽는다 (여러 줄 배열도 지원).
read_rust_list() {
    awk -v key="$1" '
        # 주석을 제거하고 요청한 배열의 시작을 찾는다.
        { sub(/#.*/, "") }
        $0 ~ "^[[:space:]]*" key "[[:space:]]*=" { active=1 }
        active { print; if (index($0, "]")) exit }
    ' "$repo_root/rust-toolchain.toml" | grep -oE '"[^"]+"' | tr -d '"'
}

# 설치 목록 및 패키지 묶음을 OS에 맞게 구성한다. 원본 script/linux의 의존성을 기준으로 한다.
configure_platform() {
    platform=$(uname -s)
    architecture=$(uname -m)
    case "$architecture" in
        x86_64|amd64) rust_cpu=x86_64 ;;
        arm64|aarch64) rust_cpu=aarch64 ;;
        *) printf '지원하지 않는 CPU: %s (x64/ARM64 필요)\n' "$architecture" >&2; return 1 ;;
    esac
    if [[ "$platform" == Darwin ]]; then
        rust_host="$rust_cpu-apple-darwin"
        manager=brew
        ids=(xcode brew git cmake rustup rust)
        names=('Xcode + C/C++/macOS SDK + Metal 도구' 'Homebrew (Git/CMake 설치 관리자)' 'Git' 'CMake' 'rustup (관리자만)' "Rust $rust_channel + 구성 요소/타깃")
        details=('Xcode가 없으면 App Store를 열어 설치를 안내합니다. 설치 후 재실행이 필요합니다.'
            'Xcode/명령줄 도구가 먼저 필요하며, 사용자 계정으로 설치합니다.'
            'Git 소스 의존성 다운로드' '네이티브 C/C++ 의존성 빌드'
            '공식 rustup만 설치하며 툴체인은 별도 항목입니다.'
            "구성 요소: ${rust_components[*]}; 타깃: ${rust_targets[*]}")
        return 0
    fi
    if [[ "$platform" != Linux ]]; then printf '지원하지 않는 OS: %s\n' "$platform" >&2; return 1; fi
    rust_host="$rust_cpu-unknown-linux-gnu"
    if has_command apt-get; then
        manager=apt
        native_packages=(build-essential clang libclang-dev pkg-config perl)
        library_packages=(libfontconfig-dev libwayland-dev libx11-xcb-dev libxcb1-dev libxkbcommon-x11-dev libssl-dev libzstd-dev libvulkan1 libsqlite3-dev zlib1g-dev libdbus-1-dev)
    elif has_command dnf; then
        manager=dnf
        native_packages=(gcc gcc-c++ make clang clang-devel llvm-devel pkgconf-pkg-config perl-FindBin perl-IPC-Cmd perl-File-Compare perl-File-Copy)
        library_packages=(fontconfig-devel wayland-devel libxcb-devel libxkbcommon-x11-devel openssl-devel libzstd-devel vulkan-loader sqlite-devel zlib-devel dbus-devel)
    elif has_command pacman; then
        manager=pacman
        native_packages=(base-devel clang pkgconf perl)
        library_packages=(fontconfig wayland libxcb libxkbcommon-x11 openssl zstd vulkan-icd-loader sqlite zlib dbus)
    elif has_command zypper; then
        manager=zypper
        native_packages=(gcc gcc-c++ make clang clang-devel llvm-devel pkg-config perl)
        library_packages=(fontconfig-devel wayland-devel libX11-devel libxcb-devel libxkbcommon-devel libxkbcommon-x11-devel libopenssl-devel libzstd-devel libvulkan1 sqlite3-devel zlib-devel dbus-devel)
    else
        printf '지원하는 Linux 패키지 관리자: apt-get, dnf, pacman, zypper. 이 시스템은 수동 설치가 필요합니다.\n' >&2
        return 1
    fi
    ids=(curl git cmake native libraries rustup rust)
    names=('curl + CA 인증서' 'Git' 'CMake' 'C/C++ 빌드 도구 + Clang/libclang + pkg-config' 'GUI/시스템 개발 라이브러리' 'rustup (관리자만)' "Rust $rust_channel + 구성 요소/타깃")
    details=('curl ca-certificates (공식 Rust 설치 파일 다운로드)' 'git (Git 소스 의존성 다운로드)' 'cmake (네이티브 의존성 빌드)'
        "${native_packages[*]}" "${library_packages[*]}" '공식 rustup만 설치하며 툴체인은 7번입니다.'
        "구성 요소: ${rust_components[*]}; 타깃: ${rust_targets[*]}")
}

# 설치된 패키지 DB만 확인한다. 저장소 갱신이나 네트워크 조회는 하지 않는다.
packages_ready() {
    local package status
    # 묶음의 모든 패키지가 설치되어야 준비됨으로 표시한다.
    for package in "$@"; do
        case "$manager" in
            apt) status=$(dpkg-query -W -f='${Status}' "$package" 2>/dev/null); [[ "$status" == 'install ok installed' ]] || return 1 ;;
            dnf|zypper) rpm -q "$package" >/dev/null 2>&1 || return 1 ;;
            pacman) pacman -Q "$package" >/dev/null 2>&1 || return 1 ;;
        esac
    done
}

# Xcode/Git/Clang의 실행 스텁이 선택 전 설치 대화상자를 띄우지 않도록 먼저 확인한다.
has_developer_tools() { xcode-select -p >/dev/null 2>&1; }

# 정식 Xcode 개발자 디렉터리를 찾는다. 사용자 지정 위치는 xcode-select 설정을 따른다.
find_xcode() {
    local active
    active=$(xcode-select -p 2>/dev/null)
    if [[ "$active" == *.app/Contents/Developer && -d "$active" ]]; then printf '%s\n' "$active"
    elif [[ -d /Applications/Xcode.app/Contents/Developer ]]; then printf '%s\n' /Applications/Xcode.app/Contents/Developer
    else return 1
    fi
}

# 툴체인 조회 전에 설치된 목록을 확인하여 Rust의 암묵적 다운로드를 피한다.
rust_ready() {
    local component target installed_components installed_targets
    has_command rustup || return 1
    rustup toolchain list 2>/dev/null | awk '{print $1}' | grep -Fxq "$rust_channel-$rust_host" || return 1
    installed_components=$(rustup component list --installed --toolchain "$rust_channel-$rust_host" 2>/dev/null) || return 1
    installed_targets=$(rustup target list --installed --toolchain "$rust_channel-$rust_host" 2>/dev/null) || return 1
    # 프로젝트에서 지정한 추가 구성 요소가 하나라도 빠졌으면 미준비로 표시한다.
    for component in "${rust_components[@]}"; do
        printf '%s\n' "$installed_components" | awk '{print $1}' | grep -Fxq "$component-$rust_host" || return 1
    done
    # wasm32-wasip2 등 추가 타깃을 모두 확인한다.
    for target in "${rust_targets[@]}"; do
        printf '%s\n' "$installed_targets" | grep -Fxq "$target" || return 1
    done
}

# 항목별 설치 상태를 조회한다. cargo와 rustc는 호출하지 않는다.
item_ready() {
    case "$1" in
        xcode) has_developer_tools && xcodebuild -version >/dev/null 2>&1 && xcrun metal --version >/dev/null 2>&1 ;;
        brew) has_command brew ;;
        curl) packages_ready curl ca-certificates ;;
        git) if [[ "$platform" == Darwin ]]; then has_developer_tools && git --version >/dev/null 2>&1; else git --version >/dev/null 2>&1; fi ;;
        cmake) cmake --version >/dev/null 2>&1 ;;
        native) packages_ready "${native_packages[@]}" ;;
        libraries) packages_ready "${library_packages[@]}" ;;
        # --version은 활성 rustc를 확인할 수 있으므로 설치 전에는 도움말만 조회한다.
        rustup) has_command rustup && rustup --help >/dev/null 2>&1 ;;
        rust) rust_ready ;;
    esac
}

# 선택 항목의 설치에 꼭 필요한 선행 항목이 준비 또는 선택됐는지 확인한다.
dependency_available() {
    local index
    # 번호 대신 내부 ID로 검색하므로 OS별 번호가 달라도 같은 검사를 쓴다.
    for ((index=0; index<${#ids[@]}; index++)); do
        if [[ "${ids[index]}" == "$1" ]]; then (( ready[index] || selected[index] )); return; fi
    done
    return 1
}

# 의존성을 몰래 추가하지 않고, 필요한 번호를 함께 선택하도록 안내한다.
validate_selection() {
    local index item
    # 선택된 각 항목의 선행 도구를 설치 시작 전에 검사한다.
    for ((index=0; index<${#ids[@]}; index++)); do
        (( selected[index] )) || continue
        item=${ids[index]}
        if [[ "$item" == rust ]] && ! dependency_available rustup; then
            printf 'Rust 툴체인에는 rustup 항목도 선택해야 합니다.\n'; return 1
        fi
        if [[ "$platform" == Linux && "$item" == rustup ]] && ! dependency_available curl; then
            printf 'rustup 다운로드에는 curl + CA 인증서 항목도 선택해야 합니다.\n'; return 1
        fi
        if [[ "$platform" == Darwin && ( "$item" == git || "$item" == cmake ) ]] && ! dependency_available brew; then
            printf 'Git/CMake 설치에는 Homebrew 항목도 선택해야 합니다.\n'; return 1
        fi
        if [[ "$platform" == Darwin && "$item" == brew ]] && ! has_developer_tools && ! dependency_available xcode; then
            printf 'Homebrew 설치에는 Xcode 항목도 선택해야 합니다.\n'; return 1
        fi
    done
}

# 선택한 시스템 패키지만 설치한다. apt 저장소 갱신은 첫 설치 때 한 번만 한다.
install_packages() {
    case "$manager" in
        apt)
            if (( ! apt_updated )); then run_admin apt-get update || return 1; apt_updated=1; fi
            run_admin apt-get install -y "$@" ;;
        dnf) run_admin dnf install -y "$@" ;;
        pacman) run_admin pacman -S --needed --noconfirm "$@" ;;
        zypper) run_admin zypper --non-interactive install "$@" ;;
        brew) run brew install "$@" ;;
    esac
}

# 선택한 설치 스크립트를 HTTPS로 임시 파일에 내려받는다. 파이프로 실행하지 않는다.
download_script() {
    cleanup_download
    download_directory=$(mktemp -d "${TMPDIR:-/tmp}/doneditor-tools.XXXXXX") || return 1
    download_file="$download_directory/install.sh"
    curl --proto '=https' --tlsv1.2 -fsSL "$1" -o "$download_file"
}

# Xcode 자체는 Apple 인증이 필요하다. 외부 설치를 시작한 경우 미완료(2)를 반환한다.
install_xcode() {
    local developer
    if ! developer=$(find_xcode); then
        printf 'Xcode를 App Store에서 설치하고 처음 실행해 macOS 구성 요소를 준비한 뒤 이 스크립트를 다시 실행하세요.\n'
        run open 'macappstore://apps.apple.com/app/id497799835' || return 1
        return 2
    fi
    run_admin xcode-select --switch "$developer" || return 1
    run_admin xcodebuild -runFirstLaunch || return 1
    if ! xcrun metal --version >/dev/null 2>&1; then
        run xcodebuild -downloadComponent MetalToolchain || return 1
    fi
}

# 선택한 항목만 설치한다. 선행 항목의 실제 설치 실패도 여기서 다시 확인한다.
install_item() {
    local arguments
    case "$1" in
        xcode) install_xcode ;;
        brew)
            if (( ! dry_run )) && ! has_developer_tools; then printf 'Xcode 설치를 완료하고 다시 실행하세요.\n'; return 2; fi
            if (( ! dry_run )) && [[ $(id -u) -eq 0 ]]; then printf 'Homebrew/rustup은 sudo 없이 사용자 계정으로 실행하세요.\n'; return 1; fi
            if (( dry_run )); then printf '  > HTTPS 다운로드 후 Homebrew 공식 install.sh 실행\n'; return 0; fi
            download_script 'https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh' || return 1
            run /bin/bash "$download_file" ;;
        rustup)
            if (( ! dry_run )) && [[ $(id -u) -eq 0 ]]; then printf 'rustup은 sudo 없이 사용자 계정으로 실행하세요.\n'; return 1; fi
            if (( dry_run )); then printf '  > https://sh.rustup.rs 다운로드 후 sh install.sh -y --default-toolchain none --profile minimal\n'; return 0; fi
            download_script 'https://sh.rustup.rs' || return 1
            run sh "$download_file" -y --default-host "$rust_host" --default-toolchain none --profile minimal ;;
        rust)
            arguments=(toolchain install "$rust_channel-$rust_host" --profile "$rust_profile" --no-self-update)
            if (( ${#rust_components[@]} )); then arguments+=(--component "$(IFS=,; printf '%s' "${rust_components[*]}")"); fi
            if (( ${#rust_targets[@]} )); then arguments+=(--target "$(IFS=,; printf '%s' "${rust_targets[*]}")"); fi
            run rustup "${arguments[@]}" ;;
        curl) install_packages curl ca-certificates ;;
        git|cmake) install_packages "$1" ;;
        native) install_packages "${native_packages[@]}" ;;
        libraries) install_packages "${library_packages[@]}" ;;
    esac
}

# 상태와 선택 여부를 나란히 표시하고 패키지 묶음의 실제 내용도 보여 준다.
show_menu() {
    local index mark state
    printf '\nDon Editor 빌드 도구 (%s / %s / %s)\n' "$platform" "$architecture" "$manager"
    # 준비된 항목과 선택한 항목을 별개로 표시한다.
    for ((index=0; index<${#ids[@]}; index++)); do
        mark=' '; state='미준비'
        (( selected[index] )) && mark=X
        (( ready[index] )) && state='준비됨'
        printf '%d. [%s] [%s] %s\n   %s\n' "$((index+1))" "$mark" "$state" "${names[index]}" "${details[index]}"
    done
    printf '%s\n' '선택한 도구의 필수 하위 의존성도 설치됩니다. 전체 OS 업데이트나 빌드는 수행하지 않습니다.'
}

# 한 번 확정된 선택만 실행하고 설치 뒤 선택 항목의 실제 상태를 재검사한다.
install_selected() {
    local index result failed=0 pending=0
    apt_updated=0
    # curl→패키지→rustup→툴체인 또는 Xcode→brew 순서로 실행한다.
    for ((index=0; index<${#ids[@]}; index++)); do
        (( selected[index] )) || continue
        printf '\n설치: %s\n' "${names[index]}"
        install_item "${ids[index]}"
        result=$?
        refresh_path
        if (( result == 2 )); then pending=1
        elif (( result != 0 )); then printf '[실패] %s (종료 코드 %d)\n' "${names[index]}" "$result"; failed=1
        elif (( ! dry_run )) && ! item_ready "${ids[index]}"; then printf '[미준비] %s\n' "${names[index]}"; failed=1
        fi
    done
    if (( dry_run )); then printf '모의 실행 완료: 실제 설치는 하지 않았습니다.\n'; return "$failed"; fi
    if (( failed )); then printf '일부 항목 설치 실패. 오류를 해결한 뒤 다시 실행하세요.\n'; return 1; fi
    if (( pending )); then printf '외부 설치를 완료하고 다시 실행하세요. 아직 모든 선택 항목이 준비되지 않았습니다.\n'; return 2; fi
    printf '선택한 항목 설치 완료. 새 터미널에서 빌드하세요. 미선택 항목은 설치하지 않았습니다.\n'
}

# 입력을 검증한 뒤 모든 번호가 유효할 때만 선택을 바꾼다.
toggle_numbers() {
    local token index seen=' '
    local tokens=()
    IFS=$' ,\t' read -r -a tokens <<< "$1"
    (( ${#tokens[@]} )) || return 1
    # 잘못된 번호가 섞여 있으면 선택 전체를 그대로 유지한다.
    for token in "${tokens[@]}"; do
        [[ "$token" =~ ^[1-9][0-9]*$ && ${#token} -le 2 ]] || return 1
        (( token <= ${#ids[@]} )) || return 1
    done
    # 중복 번호는 한 번만 전환한다.
    for token in "${tokens[@]}"; do
        [[ "$seen" != *" $token "* ]] || continue
        seen+="$token "
        index=$((token-1))
        if (( ready[index] )); then printf '%s 번은 이미 준비되어 있어 건너뜁니다.\n' "$token"
        else selected[index]=$((1-selected[index]))
        fi
    done
}

refresh_path
# Rust 버전과 추가 구성 요소는 설치 스크립트에 중복해서 고정하지 않는다.
rust_channel=$(sed -nE 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$repo_root/rust-toolchain.toml")
rust_profile=$(sed -nE 's/^[[:space:]]*profile[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$repo_root/rust-toolchain.toml")
[[ -n "$rust_channel" ]] || { printf 'rust-toolchain.toml에서 channel을 읽지 못했습니다.\n' >&2; exit 1; }
rust_profile=${rust_profile:-default}
rust_components=(); rust_targets=()
# Bash 3.2에 없는 mapfile 대신 줄 단위로 배열을 구성한다.
while IFS= read -r value; do rust_components+=("$value"); done < <(read_rust_list components)
# 추가 빌드 타깃도 같은 방식으로 읽는다.
while IFS= read -r value; do rust_targets+=("$value"); done < <(read_rust_list targets)
configure_platform || exit 1
ready=(); selected=()
# 최초 실행에서는 설치된 상태와 무관하게 모든 항목을 미선택으로 시작한다.
for ((index=0; index<${#ids[@]}; index++)); do
    ready[index]=0; selected[index]=0
    item_ready "${ids[index]}" && ready[index]=1
done

if (( check_only )); then
    show_menu
    # 상태 점검 결과는 미준비 항목이 있으면 종료 코드 1로 알린다.
    for state in "${ready[@]}"; do (( state )) || exit 1; done
    exit 0
fi

printf '처음에는 모든 항목이 미선택입니다. 번호를 선택하고 I를 입력해야 설치가 시작됩니다.\n'
# 빈 입력, EOF 또는 0은 설치 없이 종료한다.
while :; do
    show_menu
    printf '번호(예: 1 3 또는 1,3)=선택 전환 / A=미준비 전체 / N=전체 해제 / I=설치 / 0=종료\n선택: '
    IFS= read -r answer || exit 0
    answer=$(printf '%s' "$answer" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
    case "$answer" in
        ''|0|q|Q) exit 0 ;;
        a|A)
            # 준비되지 않은 항목만 모두 선택한다.
            for ((index=0; index<${#ids[@]}; index++)); do selected[index]=$((1-ready[index])); done ;;
        n|N)
            # 전체 해제는 설치 상태를 바꾸지 않는다.
            for ((index=0; index<${#ids[@]}; index++)); do selected[index]=0; done ;;
        i|I)
            if [[ " ${selected[*]} " != *' 1 '* ]]; then printf '선택한 항목이 없습니다.\n'; continue; fi
            validate_selection || continue
            install_selected; exit $? ;;
        *) toggle_numbers "$answer" || printf '목록에 있는 번호 또는 메뉴 명령을 입력하세요.\n' ;;
    esac
done
