#Requires -Version 5.1
# [DonEditor] 빌드 도구 선택 설치. GPL-3.0-or-later (저장소 LICENSE 참고).
[CmdletBinding()]
param([switch]$CheckOnly, [switch]$DryRun, [Alias('h')][switch]$Help)

$ErrorActionPreference = 'Stop'
# 저장소와 Visual Studio 설치 관리자 위치를 고정한다.
$RepoRoot = Split-Path -Parent $PSScriptRoot
$VsDirectory = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer'
$VsWhere = Join-Path $VsDirectory 'vswhere.exe'
$VsSetup = Join-Path $VsDirectory 'setup.exe'
# 네이티브 Windows 빌드에 사용할 아키텍처와 필수 구성 요소이다.
$Cpu = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
$RustHost = switch ($Cpu) {
    'AMD64' { 'x86_64-pc-windows-msvc' }
    'ARM64' { 'aarch64-pc-windows-msvc' }
    default { throw "지원하지 않는 Windows 아키텍처: $Cpu (x64/ARM64 필요)" }
}
$VsComponents = @('Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
    'Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre')
if ($Cpu -eq 'ARM64') {
    $VsComponents += 'Microsoft.VisualStudio.Component.VC.Tools.ARM64',
        'Microsoft.VisualStudio.Component.VC.Runtimes.ARM64.Spectre'
}
$SdkComponent = 'Microsoft.VisualStudio.Component.Windows11SDK.26100'
$MinSdk = [version]'10.0.20348.0'

# 실패할 수 있는 조회 명령을 실행한다. 도구 설치나 cargo/rustc 실행은 하지 않는다.
function Get-QueryOutput {
    param([string]$File, [string[]]$Arguments)
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $result = & $File @Arguments 2>$null
        if ($LASTEXITCODE -eq 0) { return $result }
    } catch { return } finally { $ErrorActionPreference = $previous }
}

# 새로 설치한 도구와 PATH 밖의 표준 설치 위치를 현재 프로세스에서 찾게 한다.
function Update-ToolPath {
    $cargoDirectory = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
    $paths = @($env:Path, [Environment]::GetEnvironmentVariable('Path', 'Machine'),
        [Environment]::GetEnvironmentVariable('Path', 'User'), (Join-Path $cargoDirectory 'bin'),
        (Join-Path $env:ProgramFiles 'CMake\bin'), (Join-Path $env:ProgramFiles 'Git\cmd'))
    $visualStudio = Get-VisualStudio @($VsComponents[0])
    if ($visualStudio) {
        $paths += Join-Path $visualStudio 'Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin'
    }
    $env:Path = ($paths -join ';')
}

# 필요한 구성 요소가 있는 가장 최신 VS 경로를 조회한다.
function Get-VisualStudio {
    param([string[]]$Components = @())
    if (-not (Test-Path -LiteralPath $VsWhere)) { return }
    $arguments = @('-latest', '-prerelease', '-products', '*', '-property', 'installationPath')
    if ($Components.Count) { $arguments += @('-requires') + $Components }
    Get-QueryOutput $VsWhere $arguments | Select-Object -First 1
}

# 실제 헤더, 링크 라이브러리, 리소스/셰이더 컴파일러가 있는 SDK만 인정한다.
function Test-WindowsSdk {
    $sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10'
    if (-not (Test-Path -LiteralPath "$sdkRoot\Include")) { return $false }
    $sdkArch = if ($Cpu -eq 'ARM64') { 'arm64' } else { 'x64' }
    # 버전별 SDK에서 빌드에 필요한 파일을 검사한다.
    foreach ($directory in (Get-ChildItem -LiteralPath "$sdkRoot\Include" -Directory)) {
        if ($directory.Name -notmatch '^10\.0\.\d+\.\d+$' -or [version]$directory.Name -lt $MinSdk) { continue }
        $version = $directory.Name
        if ((Test-Path -LiteralPath "$sdkRoot\Include\$version\um\Windows.h") -and
            (Test-Path -LiteralPath "$sdkRoot\Lib\$version\um\$sdkArch\kernel32.lib") -and
            (Test-Path -LiteralPath "$sdkRoot\Lib\$version\ucrt\$sdkArch\ucrt.lib") -and
            (Test-Path -LiteralPath "$sdkRoot\bin\$version\$sdkArch\rc.exe") -and
            (Test-Path -LiteralPath "$sdkRoot\bin\$version\$sdkArch\fxc.exe")) { return $true }
    }
    return $false
}

# 저장소가 요구하는 Rust 채널, 프로필, 구성 요소와 타깃을 읽는다.
function Read-RustSpec {
    $content = Get-Content -LiteralPath (Join-Path $RepoRoot 'rust-toolchain.toml') -Raw
    $channel = [regex]::Match($content, '(?m)^\s*channel\s*=\s*"([^"]+)"').Groups[1].Value
    $profile = [regex]::Match($content, '(?m)^\s*profile\s*=\s*"([^"]+)"').Groups[1].Value
    if (-not $channel) { throw 'rust-toolchain.toml에서 channel을 읽지 못했습니다.' }
    if (-not $profile) { $profile = 'default' }
    $lists = @{}
    # TOML의 문자열 배열 세 항목 중 설치에 쓰는 두 항목을 추출한다.
    foreach ($key in @('components', 'targets')) {
        $body = [regex]::Match($content, "(?ms)^\s*$key\s*=\s*\[(.*?)\]").Groups[1].Value
        $lists[$key] = @([regex]::Matches($body, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value })
    }
    [pscustomobject]@{ Name = "$channel-$RustHost"; Profile = $profile; Components = $lists.components; Targets = $lists.targets }
}

# rustup 조회만 사용해 툴체인과 모든 추가 구성 요소의 설치 여부를 확인한다.
function Test-RustToolchain {
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) { return $false }
    $installed = @(Get-QueryOutput 'rustup' @('toolchain', 'list'))
    if (-not ($installed | Where-Object { ($_ -split '\s+')[0] -eq $RustSpec.Name })) { return $false }
    $components = @(Get-QueryOutput 'rustup' @('component', 'list', '--installed', '--toolchain', $RustSpec.Name))
    $targets = @(Get-QueryOutput 'rustup' @('target', 'list', '--installed', '--toolchain', $RustSpec.Name))
    # rustfmt, clippy, rust-analyzer 등 저장소의 모든 추가 구성 요소를 검사한다.
    foreach ($component in $RustSpec.Components) {
        if (-not ($components | Where-Object { ($_ -split '\s+')[0] -in @($component, "$component-$RustHost") })) { return $false }
    }
    # wasm32-wasip2를 포함한 타깃이 빠졌는지 확인한다.
    foreach ($target in $RustSpec.Targets) { if ($targets -notcontains $target) { return $false } }
    return $true
}

# 각 도구의 상태를 읽기 전용으로 조회한다.
function Get-ToolItems {
    @(
        [pscustomobject]@{ Name = 'Git'; Why = 'Git 소스 의존성 다운로드'; Ready = [bool](Get-QueryOutput 'git' @('--version')) },
        [pscustomobject]@{ Name = 'CMake'; Why = '네이티브 C/C++ 의존성 빌드'; Ready = [bool](Get-QueryOutput 'cmake' @('--version')) },
        [pscustomobject]@{ Name = 'MSVC C++ + Spectre 완화 라이브러리'; Why = "Visual Studio/Build Tools ($Cpu)"; Ready = [bool](Get-VisualStudio $VsComponents) },
        [pscustomobject]@{ Name = "Windows SDK (>= $MinSdk)"; Why = '헤더, 라이브러리, rc.exe, fxc.exe'; Ready = Test-WindowsSdk },
        # --version은 활성 rustc 조회 중 툴체인을 설치할 수 있어 도움말로 실행 가능 여부만 확인한다.
        [pscustomobject]@{ Name = 'rustup'; Why = 'Rust 관리자만 설치 (툴체인은 6번)'; Ready = [bool](Get-QueryOutput 'rustup' @('--help')) },
        [pscustomobject]@{ Name = "Rust $($RustSpec.Name)"; Why = "구성 요소: $($RustSpec.Components -join ', '); 타깃: $($RustSpec.Targets -join ', ')"; Ready = Test-RustToolchain }
    )
}

# 설치 명령을 출력하고 실행하며, 실패 종료 코드를 누락하지 않는다.
function Invoke-InstallCommand {
    param([string]$File, [string[]]$Arguments)
    Write-Host "  > $File $($Arguments -join ' ')"
    if ($DryRun) { return }
    & $File @Arguments | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "설치 명령 실패 (종료 코드 $LASTEXITCODE): $File" }
}

# 선택한 WinGet 패키지만 설치한다. 패키지 관리자는 자동 설치하지 않는다.
function Install-WingetPackage {
    param([string]$Id, [string]$Override)
    if (-not $DryRun -and -not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw 'winget이 없습니다. 앱 설치 관리자 https://aka.ms/getwinget 설치 후 다시 실행하세요.'
    }
    $arguments = @('install', '--id', $Id, '--exact', '--source', 'winget',
        '--accept-package-agreements', '--accept-source-agreements')
    if ($Override) { $arguments += @('--override', $Override) }
    Invoke-InstallCommand 'winget' $arguments
}

# 선택한 MSVC/SDK 구성 요소만 기존 VS에 추가하거나 새 Build Tools를 설치한다.
function Install-VisualStudioComponents {
    param([bool]$Tools, [bool]$Sdk)
    $components = @()
    if ($Tools) { $components += $VsComponents }
    if ($Sdk) { $components += $SdkComponent }
    $existing = Get-VisualStudio @($VsComponents[0])
    if (-not $existing) { $existing = Get-VisualStudio }
    $add = ($components | ForEach-Object { "--add $_" }) -join ' '
    if ($existing -and (Test-Path -LiteralPath $VsSetup)) {
        $arguments = "modify --installPath `"$existing`" $add --passive --norestart"
        Write-Host "  > `"$VsSetup`" $arguments"
        if ($DryRun) { return }
        # 공백을 포함하는 설치 경로를 한 인자 문자열로 인용해 전달한다.
        $process = Start-Process -FilePath $VsSetup -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -Wait -PassThru
        if ($process.ExitCode -eq 3010) { Write-Host '설치 완료 후 재부팅이 필요합니다.' }
        elseif ($process.ExitCode -ne 0) { throw "VS 구성 요소 설치 실패 (종료 코드 $($process.ExitCode))" }
    } else {
        Install-WingetPackage 'Microsoft.VisualStudio.BuildTools' "--wait --passive --norestart $add"
    }
}

# 공식 rustup 설치 파일과 SHA256을 받아 검증한 뒤 관리자만 설치한다.
function Install-RustupOnly {
    $url = "https://static.rust-lang.org/rustup/dist/$RustHost/rustup-init.exe"
    Write-Host "  다운로드 및 SHA256 검증: $url"
    if ($DryRun) { Write-Host '  > rustup-init.exe -y --default-toolchain none --profile minimal'; return }
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ("doneditor-rustup-" + [guid]::NewGuid())
    [void](New-Item -ItemType Directory -Path $temporary)
    $installer = Join-Path $temporary 'rustup-init.exe'
    $oldMsvcCheck = $env:RUSTUP_INIT_SKIP_MSVC_CHECK
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $installer
        $checksum = (Invoke-WebRequest -UseBasicParsing -Uri "$url.sha256").Content
        $expected = [regex]::Match([string]$checksum, '^[a-fA-F0-9]{64}').Value
        if (-not $expected -or (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash -ne $expected) { throw 'rustup SHA256 검증 실패' }
        # MSVC는 3번에서만 설치하며 rustup의 별도 설치 제안을 막는다.
        $env:RUSTUP_INIT_SKIP_MSVC_CHECK = '1'
        Invoke-InstallCommand $installer @('-y', '--default-host', $RustHost, '--default-toolchain', 'none', '--profile', 'minimal')
    } finally {
        $env:RUSTUP_INIT_SKIP_MSVC_CHECK = $oldMsvcCheck
        # 재귀 삭제 없이 이 작업에서 만든 파일과 빈 임시 폴더만 제거한다.
        Get-ChildItem -LiteralPath $temporary -File | Remove-Item -Force
        Remove-Item -LiteralPath $temporary -Force
    }
}

# 프로젝트 지정 툴체인과 추가 구성 요소를 명시적으로 설치한다. 전역 기본 버전은 바꾸지 않는다.
function Install-ProjectRust {
    $arguments = @('toolchain', 'install', $RustSpec.Name, '--profile', $RustSpec.Profile, '--no-self-update')
    if ($RustSpec.Components.Count) { $arguments += @('--component', ($RustSpec.Components -join ',')) }
    if ($RustSpec.Targets.Count) { $arguments += @('--target', ($RustSpec.Targets -join ',')) }
    Invoke-InstallCommand 'rustup' $arguments
}

# 한국어 번호 선택 메뉴를 표시하고, 명시적으로 I를 누른 뒤에만 설치한다.
function Invoke-ToolInstaller {
    if ($Help) {
        Write-Host '사용법: install-build-tools.bat [-CheckOnly | -DryRun | -Help]'
        Write-Host '번호=선택 전환, A=미준비 전체 선택, N=전체 해제, I=선택 설치, 0=종료'
        Write-Host '-CheckOnly: 상태 조회만. -DryRun: 번호 선택 후 설치 명령만 출력.'
        return 0
    }
    Update-ToolPath
    $script:RustSpec = Read-RustSpec
    $items = @(Get-ToolItems)
    $selected = @($false) * $items.Count
    # 입력이 닫히거나 종료 번호를 선택할 때까지 메뉴를 반복한다.
    while ($true) {
        Write-Host "`nDon Editor 빌드 도구 ($Cpu)"
        # 설치 상태와 선택 표시를 분리해 보여 준다.
        for ($index = 0; $index -lt $items.Count; $index++) {
            $mark = if ($selected[$index]) { 'X' } else { ' ' }
            $state = if ($items[$index].Ready) { '준비됨' } else { '미준비' }
            Write-Host ("{0}. [{1}] [{2}] {3}`n   {4}" -f ($index + 1), $mark, $state, $items[$index].Name, $items[$index].Why)
        }
        Write-Host 'WinGet/VS Installer 및 공식 Rust 배포 서버를 사용합니다. 선택한 도구의 필수 하위 의존성도 설치됩니다.'
        if ($CheckOnly) { if ($items.Ready -contains $false) { return 1 }; return 0 }
        Write-Host '번호(예: 1 3 또는 1,3)=선택 전환 / A=미준비 전체 / N=전체 해제 / I=설치 / 0=종료'
        try { $answer = Read-Host '선택' } catch { return 0 }
        if ($null -eq $answer -or $answer.Trim() -eq '0' -or $answer.Trim() -eq '') { return 0 }
        switch ($answer.Trim().ToUpperInvariant()) {
            'A' {
                # 준비된 항목은 재설치 목록에서 제외한다.
                for ($index = 0; $index -lt $items.Count; $index++) { $selected[$index] = -not $items[$index].Ready }
                continue
            }
            'N' { $selected = @($false) * $items.Count; continue }
            'I' {
                if ($selected -notcontains $true) { Write-Host '선택한 항목이 없습니다.'; continue }
                if ($selected[5] -and -not $items[4].Ready -and -not $selected[4]) {
                    Write-Host '6번 툴체인을 설치하려면 5번 rustup도 선택해야 합니다.'; continue
                }
                $failed = $false
                # 번호 순서로 선택된 항목만 설치한다. VS/SDK는 한 번에 설치해 중복 작업을 피한다.
                for ($index = 0; $index -lt $items.Count; $index++) {
                    if (-not $selected[$index] -or ($index -eq 3 -and $selected[2])) { continue }
                    try {
                        Write-Host "`n설치: $($items[$index].Name)"
                        switch ($index) {
                            0 { Install-WingetPackage 'Git.Git' }
                            1 { Install-WingetPackage 'Kitware.CMake' }
                            2 { Install-VisualStudioComponents $true $selected[3] }
                            3 { Install-VisualStudioComponents $false $true }
                            4 { Install-RustupOnly }
                            5 { Install-ProjectRust }
                        }
                        Update-ToolPath
                    } catch { Write-Host "[실패] $($_.Exception.Message)"; $failed = $true }
                }
                if ($DryRun) { Write-Host '모의 실행 완료: 실제 설치는 하지 않았습니다.'; return 0 }
                $items = @(Get-ToolItems)
                # 선택한 항목은 설치 명령의 성공뿐 아니라 실제 준비 상태도 확인한다.
                for ($index = 0; $index -lt $items.Count; $index++) {
                    if ($selected[$index] -and -not $items[$index].Ready) { Write-Host "[미준비] $($items[$index].Name)"; $failed = $true }
                }
                if ($failed) { Write-Host '일부 항목이 준비되지 않았습니다. 오류를 해결하거나 재부팅한 뒤 다시 실행하세요.'; return 1 }
                Write-Host '선택한 항목 설치 완료. 새 터미널에서 빌드하세요. 미선택 항목은 설치하지 않았습니다.'
                return 0
            }
            default {
                $numbers = $answer.Trim() -split '[,\s]+' | Select-Object -Unique
                if ($numbers | Where-Object { $_ -notmatch '^[1-6]$' }) { Write-Host '목록에 있는 번호 또는 메뉴 명령을 입력하세요.'; continue }
                # 같은 번호를 다시 입력하면 선택이 해제된다.
                foreach ($number in $numbers) {
                    $index = [int]$number - 1
                    if ($items[$index].Ready) { Write-Host "$number 번은 이미 준비되어 있어 건너뜁니다."; continue }
                    $selected[$index] = -not $selected[$index]
                }
            }
        }
    }
}

# 직접 실행 시에만 메뉴를 시작한다. 점검 테스트는 함수를 불러와 외부 설치 명령을 대체한다.
if ($MyInvocation.InvocationName -ne '.') {
    try { exit (Invoke-ToolInstaller) } catch { Write-Host "[실패] $($_.Exception.Message)"; exit 1 }
}
