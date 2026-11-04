#Requires -Version 5.1
<#
.SYNOPSIS
    Don Editor 배포용 zip 파일을 만든다 (Windows).

.DESCRIPTION
    1. 빌드에 필요한 프로그램(Git, rustup, Rust 툴체인, MSVC + Spectre 라이브러리,
       Windows SDK, CMake)이 있는지 검사하고, 없으면 설치할지 묻는다 (winget 사용).
    2. cargo 로 doneditor.exe(GUI) 와 cli.exe 를 빌드한다.
    3. 배포 폴더를 구성하고 zip 과 SHA256 파일을 만든다.

    결과물: <OutDir>\DonEditor-<버전>-windows-<아키텍처>.zip (+ .sha256)

    zip 안의 구성
      DonEditor-<버전>-windows-<아키텍처>\
        doneditor.exe        GUI 실행 파일
        bin\doneditor.exe    명령줄(CLI) 실행 파일
        LICENSE, licenses.md, licenses\, README.md, SOURCE.txt

.PARAMETER Arch
    빌드 대상 아키텍처 (x86_64 또는 aarch64). 기본값은 이 PC 의 아키텍처.
    aarch64 는 x64 PC 에서의 교차 빌드이며 아직 검증하지 않았다.

.PARAMETER OutDir
    결과물을 둘 폴더. 상대 경로는 저장소 루트 기준. 기본값 dist.

.PARAMETER BuildProfile
    Release(기본) 또는 Debug. Debug 는 빌드가 빠르지만 파일이 매우 크다. 스크립트 점검용.

.PARAMETER Yes
    설치 질문에 모두 "예"로 답한다. (관리자 권한 확인창(UAC)은 Windows 가 따로 띄운다.)

.PARAMETER SkipCheck
    빌드 도구 점검을 건너뛴다. 도구가 이미 준비된 환경(개발자 명령 프롬프트 등)용.

.PARAMETER CheckOnly
    빌드 도구 점검 결과만 보여 주고 끝낸다. 설치·빌드는 하지 않는다.

.PARAMETER DryRun
    설치 명령을 실행하지 않고 어떤 명령이 실행될지만 출력한다. 빌드도 하지 않는다.

.PARAMETER SkipBuild
    빌드를 건너뛰고 target 폴더에 이미 있는 결과물로 zip 만 만든다.

.PARAMETER IncludeSource
    git archive 로 만든 소스 zip(DonEditor-<버전>-source.zip)을 함께 만든다.
    커밋되지 않은 변경은 포함되지 않는다. (GPL 소스 제공 의무 이행용)

.EXAMPLE
    .\script\package-zip.ps1
.EXAMPLE
    .\script\package-zip.ps1 -CheckOnly
.EXAMPLE
    .\script\package-zip.ps1 -Yes -IncludeSource
#>
[CmdletBinding()]
Param(
    [ValidateSet('x86_64', 'aarch64')][string]$Arch,
    [string]$OutDir = 'dist',
    [ValidateSet('Release', 'Debug')][string]$BuildProfile = 'Release',
    [switch]$Yes,
    [switch]$SkipCheck,
    [switch]$CheckOnly,
    [switch]$DryRun,
    [switch]$SkipBuild,
    [switch]$IncludeSource,
    [Alias('h')][switch]$Help
)

# 오류가 나면 즉시 중단한다 (네이티브 명령의 종료 코드는 Invoke-Native 에서 따로 검사한다)
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# 상수
# ---------------------------------------------------------------------------

# 저장소 루트 (이 스크립트는 script\ 아래에 있다)
$RepoRoot = Split-Path -Parent $PSScriptRoot
# 제품 이름 (zip 파일 이름의 접두사)
$ProductName = 'DonEditor'
# 빌드에 필요한 Windows SDK 최소 버전 (원본 Windows 빌드 문서 기준)
$MinWindowsSdkVersion = [Version]'10.0.20348.0'
# Visual Studio Installer 위치 (vswhere.exe, setup.exe)
$VsInstallerDir = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer'
$VsWherePath = Join-Path $VsInstallerDir 'vswhere.exe'
$VsSetupPath = Join-Path $VsInstallerDir 'setup.exe'
# VS 컴포넌트: x64 C++ 도구, Spectre 완화 라이브러리(languages 가 의존하는 pet* crate 가 필수로 요구), Windows SDK
$VsToolsX64 = 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64'
$VsSpectreX64 = 'Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre'
$VsToolsArm64 = 'Microsoft.VisualStudio.Component.VC.Tools.ARM64'
$VsSpectreArm64 = 'Microsoft.VisualStudio.Component.VC.Runtimes.ARM64.Spectre'
$VsWindowsSdk = 'Microsoft.VisualStudio.Component.Windows11SDK.26100'
# winget 패키지 ID
$WingetGit = 'Git.Git'
$WingetRustup = 'Rustlang.Rustup'
$WingetCmake = 'Kitware.CMake'
$WingetBuildTools = 'Microsoft.VisualStudio.BuildTools'
# VS 설치 프로그램이 "성공(재부팅 필요)" 을 뜻하는 종료 코드
$ExitCodeRebootRequired = 3010

# ---------------------------------------------------------------------------
# 대상 아키텍처 결정
# ---------------------------------------------------------------------------

# 이 PC 의 아키텍처 (x86_64 / aarch64)
$OsArch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    'X64' { 'x86_64' }
    'Arm64' { 'aarch64' }
    default { throw '지원하지 않는 CPU 아키텍처입니다.' }
}
if (-not $Arch) { $Arch = $OsArch }
# 이 PC 와 대상 아키텍처가 다르면 교차 빌드 (--target 을 지정하고 target\<triple>\ 아래에 산출된다)
$IsCrossBuild = ($Arch -ne $OsArch)
$TargetTriple = "$Arch-pc-windows-msvc"
# 필요한 VS 컴포넌트 목록 (아키텍처에 따라 ARM64 도구가 추가된다)
$RequiredVsComponents = @($VsToolsX64, $VsSpectreX64)
if ($Arch -eq 'aarch64') { $RequiredVsComponents += @($VsToolsArm64, $VsSpectreArm64) }
# vswhere / Launch-VsDevShell 이 쓰는 아키텍처 이름
$VsArchName = @{ 'x86_64' = 'amd64'; 'aarch64' = 'arm64' }

# ---------------------------------------------------------------------------
# 출력 도우미
# ---------------------------------------------------------------------------

# 사용법을 출력한다
function Show-Usage {
    Write-Host @'
사용법: package-zip.ps1 [옵션]   (package-zip.bat 으로 실행해도 된다)

  -Arch <x86_64|aarch64>   빌드 대상 아키텍처 (기본: 이 PC)
  -OutDir <폴더>           결과물 폴더 (기본: dist)
  -BuildProfile <Release|Debug>  빌드 종류 (기본: Release)
  -Yes                     설치 질문에 모두 "예"로 답한다
  -SkipCheck               빌드 도구 점검을 건너뛴다
  -CheckOnly               점검 결과만 보여 주고 끝낸다
  -DryRun                  설치 명령을 실행하지 않고 출력만 한다 (빌드도 안 함)
  -SkipBuild               빌드 없이 기존 결과물로 zip 만 만든다
  -IncludeSource           소스 zip 도 함께 만든다 (git archive, 커밋된 내용만)
  -Help                    이 도움말

자세한 설명: Get-Help .\script\package-zip.ps1 -Full
'@
}

# 단계 제목을 출력한다
function Write-Step {
    Param([string]$Message)
    Write-Host ''
    Write-Host "== $Message" -ForegroundColor Cyan
}

# 일반 안내 문구를 출력한다
function Write-Info {
    Param([string]$Message)
    Write-Host "   $Message"
}

# 경고 문구를 출력한다
function Write-Warn {
    Param([string]$Message)
    Write-Host "[주의] $Message" -ForegroundColor Yellow
}

# 예/아니오 질문. 기본값은 "아니오"이며, -Yes 가 켜져 있으면 묻지 않고 "예"로 답한다.
# 입력을 받을 수 없는 환경(입력이 닫힌 경우)에서도 "아니오"로 처리한다.
function Read-YesNo {
    Param([string]$Question)
    if ($Yes) {
        Write-Host "$Question [y/N] -> 예 (-Yes)"
        return $true
    }
    try {
        $answer = Read-Host "$Question [y/N]"
    } catch {
        return $false
    }
    if ([string]::IsNullOrWhiteSpace($answer)) { return $false }
    return ($answer.Trim() -match '^(y|yes|예|네|ㅇ)$')
}

# ---------------------------------------------------------------------------
# 실행·조회 도우미
# ---------------------------------------------------------------------------

# 네이티브 명령을 실행하고 실패(종료 코드 != 0)하면 예외를 던진다. -DryRun 이면 출력만 한다.
function Invoke-Native {
    Param([string]$File, [string[]]$Arguments)
    Write-Host ("   > {0} {1}" -f $File, ($Arguments -join ' ')) -ForegroundColor DarkGray
    if ($DryRun) { return }
    # 출력은 화면에만 보낸다 (파이프라인에 섞이면 호출한 함수의 반환값이 오염된다)
    & $File @Arguments | Out-Host
    if ($LASTEXITCODE -ne 0) {
        throw "명령이 실패했습니다 (종료 코드 $LASTEXITCODE): $File $($Arguments -join ' ')"
    }
}

# 네이티브 명령의 출력(줄 배열)을 돌려준다. 실패하면 $null. 오류 출력은 버린다.
# (PowerShell 5.1 은 오류 출력을 리다이렉트하면 예외로 바꾸므로 ErrorActionPreference 를 잠시 바꾼다)
function Get-NativeOutput {
    Param([string]$File, [string[]]$Arguments)
    $old = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $output = & $File @Arguments 2>$null
        if ($LASTEXITCODE -ne 0) { return $null }
        # 앞의 쉼표는 한 줄짜리 결과가 문자열로 풀려 배열이 아니게 되는 것을 막는다 (호출하는 쪽은 @() 로 다시 감싸지 말 것)
        return , @($output)
    } catch {
        return $null
    } finally {
        $ErrorActionPreference = $old
    }
}

# 설치 직후 현재 세션의 PATH 를 갱신한다 (새 터미널을 열지 않아도 방금 설치한 도구를 쓰기 위해)
function Update-SessionPath {
    $machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $merged = @()
    # 시스템·사용자·현재 PATH 를 합치되 중복은 제거한다
    foreach ($entry in (@($machinePath, $userPath, $env:Path) -join ';').Split(';')) {
        if ($entry -and ($merged -notcontains $entry)) { $merged += $entry }
    }
    # rustup 이 설치하는 cargo 폴더도 포함한다
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
    $cargoBin = Join-Path $cargoHome 'bin'
    if ((Test-Path $cargoBin) -and ($merged -notcontains $cargoBin)) { $merged += $cargoBin }
    $env:Path = $merged -join ';'
}

# winget 으로 패키지를 설치한다. $Override 가 있으면 설치 프로그램에 그대로 전달한다.
function Invoke-Winget {
    Param([string]$PackageId, [string]$Override)
    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw 'winget 을 찾을 수 없어 자동 설치를 할 수 없습니다. "앱 설치 관리자"(https://aka.ms/getwinget)를 설치하거나 안내된 주소에서 직접 설치해 주세요.'
    }
    $wingetArgs = @('install', '--id', $PackageId, '-e', '--source', 'winget',
        '--accept-package-agreements', '--accept-source-agreements')
    if ($Override) { $wingetArgs += @('--override', $Override) }
    Invoke-Native 'winget' $wingetArgs
    if (-not $DryRun) { Update-SessionPath }
}

# ---------------------------------------------------------------------------
# 환경 조회
# ---------------------------------------------------------------------------

# 필요한 VS 컴포넌트를 모두 가진 가장 최신 Visual Studio 설치 경로를 돌려준다. 없으면 $null.
# (-latest 만 쓰면 C++ 도구가 없는 오래된 VS 가 선택될 수 있어 반드시 -requires 로 좁힌다)
function Get-VsInstallPath {
    Param([string[]]$Requires = @())
    if (-not (Test-Path $VsWherePath)) { return $null }
    $vsArgs = @('-latest', '-prerelease', '-products', '*', '-property', 'installationPath')
    if ($Requires.Count -gt 0) { $vsArgs += @('-requires') + $Requires }
    $output = Get-NativeOutput $VsWherePath $vsArgs
    if (-not $output) { return $null }
    $path = [string]($output | Select-Object -First 1)
    if ([string]::IsNullOrWhiteSpace($path)) { return $null }
    return $path.Trim()
}

# 설치된 Windows SDK 중 가장 높은 버전을 돌려준다. 없으면 $null.
function Get-WindowsSdkVersion {
    $includeDir = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\Include'
    if (-not (Test-Path $includeDir)) { return $null }
    $found = @()
    # 버전 이름의 폴더 중 실제 헤더(um\Windows.h)가 있는 것만 인정한다
    foreach ($dir in (Get-ChildItem $includeDir -Directory)) {
        if (($dir.Name -match '^10\.0\.\d+\.\d+$') -and (Test-Path (Join-Path $dir.FullName 'um\Windows.h'))) {
            $found += [Version]$dir.Name
        }
    }
    if ($found.Count -eq 0) { return $null }
    return ($found | Sort-Object -Descending | Select-Object -First 1)
}

# cmake.exe 경로를 찾는다. PATH 에 없으면 흔한 설치 위치를 찾아 이 세션의 PATH 에 추가한다.
function Get-CmakePath {
    $command = Get-Command cmake -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    $candidates = @(Join-Path $env:ProgramFiles 'CMake\bin\cmake.exe')
    # Visual Studio 에 포함된 CMake
    $vsPath = Get-VsInstallPath @($VsToolsX64)
    if ($vsPath) { $candidates += Join-Path $vsPath 'Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe' }
    # 후보 위치를 차례로 확인한다
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            $env:Path = (Split-Path -Parent $candidate) + ';' + $env:Path
            return $candidate
        }
    }
    return $null
}

# rust-toolchain.toml 에서 채널과 targets 목록을 읽는다
function Get-RustToolchainSpec {
    $text = Get-Content (Join-Path $RepoRoot 'rust-toolchain.toml') -Raw
    $channel = [regex]::Match($text, '(?m)^\s*channel\s*=\s*"([^"]+)"').Groups[1].Value
    $targets = @()
    $targetsMatch = [regex]::Match($text, '(?ms)^\s*targets\s*=\s*\[(.*?)\]')
    if ($targetsMatch.Success) {
        # 대괄호 안의 따옴표 문자열을 모두 모은다
        foreach ($m in [regex]::Matches($targetsMatch.Groups[1].Value, '"([^"]+)"')) { $targets += $m.Groups[1].Value }
    }
    if (-not $channel) { throw 'rust-toolchain.toml 에서 channel 을 읽지 못했습니다.' }
    return [pscustomobject]@{ Channel = $channel; Targets = $targets }
}

# 워크스페이스 버전을 Cargo.toml 의 [workspace.package] 에서 읽는다
function Get-WorkspaceVersion {
    $text = Get-Content (Join-Path $RepoRoot 'Cargo.toml') -Raw
    # [workspace.package] 절부터 다음 절 머리말 전까지만 본다
    $section = [regex]::Match($text, '(?ms)^\[workspace\.package\]\s*(.*?)(?=^\[|\z)').Groups[1].Value
    $version = [regex]::Match($section, '(?m)^\s*version\s*=\s*"([^"]+)"').Groups[1].Value
    if (-not $version) { throw 'Cargo.toml 의 [workspace.package] 에서 version 을 읽지 못했습니다.' }
    return $version
}

# 릴리스 채널(stable/dev)을 읽는다. 빌드 시 exe 아이콘과 앱 식별자를 정한다.
function Get-ReleaseChannel {
    return (Get-Content (Join-Path $RepoRoot 'crates\gram\RELEASE_CHANNEL') -Raw).Trim()
}

# ---------------------------------------------------------------------------
# 빌드 도구 점검·설치
# ---------------------------------------------------------------------------

# VS C++ 도구·Spectre 라이브러리·Windows SDK 를 설치한다.
# 이미 C++ 도구가 있는 VS 가 있으면 거기에 컴포넌트를 추가하고(관리자 권한), 없으면 Build Tools 를 새로 설치한다.
function Install-BuildToolsComponents {
    $components = $RequiredVsComponents + @($VsWindowsSdk)
    $existing = Get-VsInstallPath @($VsToolsX64)
    if ($existing -and (Test-Path $VsSetupPath)) {
        # 설치 경로에 공백이 있으므로 인자 전체를 한 문자열로 만들고 경로를 따옴표로 감싼다
        $addArgs = ($components | ForEach-Object { "--add $_" }) -join ' '
        $argLine = "modify --installPath `"$existing`" $addArgs --passive --norestart"
        Write-Host "   > `"$VsSetupPath`" $argLine" -ForegroundColor DarkGray
        if ($DryRun) { return }
        Write-Info '관리자 권한 확인창이 나타나면 승인해 주세요. 설치에 몇 분이 걸립니다.'
        $process = Start-Process -FilePath $VsSetupPath -ArgumentList $argLine -Verb RunAs -Wait -PassThru
        if (($process.ExitCode -ne 0) -and ($process.ExitCode -ne $ExitCodeRebootRequired)) {
            throw "Visual Studio 구성 요소 설치가 실패했습니다 (종료 코드 $($process.ExitCode))."
        }
        if ($process.ExitCode -eq $ExitCodeRebootRequired) { Write-Warn '설치가 끝났지만 재부팅이 필요합니다.' }
    } else {
        # VS 가 없으면 Build Tools(무료)를 winget 으로 설치한다
        $override = '--passive --wait --norestart ' + (($components | ForEach-Object { "--add $_" }) -join ' ')
        Invoke-Winget $WingetBuildTools $override
    }
}

# 점검 항목 목록을 만든다. 각 항목은 Name(이름), Why(필요한 이유), Check(점검), Install(설치), Manual(직접 설치 안내)을 가진다.
function Get-Prerequisites {
    # 점검·설치 블록은 나중에 다른 범위에서 실행되므로 지역 변수가 아닌 스크립트 변수($RustToolchain)를 참조한다
    return @(
        [pscustomobject]@{
            Name = 'Git'
            Why = '빌드 커밋 정보 기록과 소스 zip 생성'
            Manual = 'https://git-scm.com/download/win'
            Check = {
                $out = Get-NativeOutput 'git' @('--version')
                if ($out) { [pscustomobject]@{ Ok = $true; Detail = [string]$out[0] } }
                else { [pscustomobject]@{ Ok = $false; Detail = 'git 을 찾을 수 없음' } }
            }
            Install = { Invoke-Winget $WingetGit }
        },
        [pscustomobject]@{
            Name = 'rustup'
            Why = 'Rust 툴체인 관리자 (cargo 포함)'
            Manual = 'https://rustup.rs'
            Check = {
                $out = Get-NativeOutput 'rustup' @('--version')
                if ($out) { [pscustomobject]@{ Ok = $true; Detail = [string]$out[0] } }
                else { [pscustomobject]@{ Ok = $false; Detail = 'rustup 을 찾을 수 없음' } }
            }
            Install = { Invoke-Winget $WingetRustup }
        },
        [pscustomobject]@{
            Name = "Rust 툴체인 $($RustToolchain.Channel)"
            Why = 'rust-toolchain.toml 에 지정된 컴파일러와 대상(target)'
            Manual = 'rustup toolchain install   (저장소 폴더에서 실행)'
            Check = {
                if (-not (Get-NativeOutput 'rustup' @('--version'))) {
                    return [pscustomobject]@{ Ok = $false; Detail = 'rustup 이 먼저 필요함' }
                }
                $installed = Get-NativeOutput 'rustup' @('toolchain', 'list')
                $hasToolchain = @($installed | Where-Object { $_ -like "$($RustToolchain.Channel)-*" }).Count -gt 0
                if (-not $hasToolchain) {
                    return [pscustomobject]@{ Ok = $false; Detail = "툴체인 $($RustToolchain.Channel) 미설치" }
                }
                # 필요한 대상(target)이 모두 설치됐는지 확인한다
                $neededTargets = @($RustToolchain.Targets)
                if ($IsCrossBuild) { $neededTargets += $TargetTriple }
                $haveTargets = Get-NativeOutput 'rustup' @('target', 'list', '--installed', '--toolchain', $RustToolchain.Channel)
                $missing = @($neededTargets | Where-Object { $haveTargets -notcontains $_ })
                if ($missing.Count -gt 0) {
                    return [pscustomobject]@{ Ok = $false; Detail = "대상 미설치: $($missing -join ', ')" }
                }
                [pscustomobject]@{ Ok = $true; Detail = "$($RustToolchain.Channel) (대상: $($neededTargets -join ', '))" }
            }
            Install = {
                # 저장소 폴더에서 실행해야 rust-toolchain.toml 의 툴체인이 설치된다
                Push-Location $RepoRoot
                try {
                    $installed = Get-NativeOutput 'rustup' @('toolchain', 'list')
                    if (@($installed | Where-Object { $_ -like "$($RustToolchain.Channel)-*" }).Count -eq 0) {
                        Invoke-Native 'rustup' @('toolchain', 'install')
                    }
                    # 교차 빌드 대상이나 빠진 대상을 추가한다
                    $neededTargets = @($RustToolchain.Targets)
                    if ($IsCrossBuild) { $neededTargets += $TargetTriple }
                    foreach ($target in $neededTargets) {
                        Invoke-Native 'rustup' @('target', 'add', $target, '--toolchain', $RustToolchain.Channel)
                    }
                } finally {
                    Pop-Location
                }
            }
        },
        [pscustomobject]@{
            Name = 'MSVC C++ 도구 + Spectre 완화 라이브러리'
            Why = 'Rust 의 MSVC 링커와 languages 가 의존하는 pet* crate (Spectre 라이브러리 없으면 빌드 실패)'
            Manual = 'https://visualstudio.microsoft.com/visual-cpp-build-tools  (Visual Studio Installer 의 개별 구성 요소에서 "MSVC ... Spectre 완화 라이브러리(최신)" 선택)'
            Check = {
                $path = Get-VsInstallPath $RequiredVsComponents
                if ($path) { [pscustomobject]@{ Ok = $true; Detail = $path } }
                else { [pscustomobject]@{ Ok = $false; Detail = 'MSVC 도구 또는 Spectre 완화 라이브러리가 설치된 Visual Studio 없음' } }
            }
            Install = { Install-BuildToolsComponents }
        },
        [pscustomobject]@{
            Name = "Windows SDK (>= $MinWindowsSdkVersion)"
            Why = 'Windows API 헤더와 라이브러리'
            Manual = 'https://developer.microsoft.com/windows/downloads/windows-sdk'
            Check = {
                $version = Get-WindowsSdkVersion
                if ($version -and ($version -ge $MinWindowsSdkVersion)) { [pscustomobject]@{ Ok = $true; Detail = "$version" } }
                elseif ($version) { [pscustomobject]@{ Ok = $false; Detail = "설치된 최고 버전 $version 이 너무 낮음" } }
                else { [pscustomobject]@{ Ok = $false; Detail = 'Windows SDK 없음' } }
            }
            Install = { Install-BuildToolsComponents }
        },
        [pscustomobject]@{
            Name = 'CMake'
            Why = 'wasmtime 등 일부 의존성의 C/C++ 빌드'
            Manual = 'https://cmake.org/download'
            Check = {
                $path = Get-CmakePath
                if ($path) { [pscustomobject]@{ Ok = $true; Detail = $path } }
                else { [pscustomobject]@{ Ok = $false; Detail = 'cmake 를 찾을 수 없음' } }
            }
            Install = { Invoke-Winget $WingetCmake }
        }
    )
}

# 빌드 도구를 점검한다. 없는 항목은 설치할지 묻고(-CheckOnly 이면 묻지 않음), 모두 갖춰졌는지 돌려준다.
function Confirm-Prerequisites {
    Write-Step '빌드 도구 점검'
    if (-not (Test-Path $VsWherePath)) {
        Write-Info 'Visual Studio Installer(vswhere)가 없습니다. Visual Studio 계열 도구가 설치되어 있지 않은 것으로 보입니다.'
    }
    $items = Get-Prerequisites

    # 1차 점검 결과를 표로 보여 준다
    foreach ($item in $items) {
        $result = & $item.Check
        if ($result.Ok) { Write-Host ("  [OK]   {0}  -  {1}" -f $item.Name, $result.Detail) -ForegroundColor Green }
        else { Write-Host ("  [없음] {0}  -  {1}" -f $item.Name, $result.Detail) -ForegroundColor Yellow }
    }
    if ($CheckOnly) {
        $missingNames = @($items | Where-Object { -not (& $_.Check).Ok } | ForEach-Object { $_.Name })
        return ($missingNames.Count -eq 0)
    }

    # 없는 항목마다 설치 여부를 묻고 설치한다 (앞 항목의 설치로 해결된 항목은 다시 점검해 건너뛴다)
    $declined = @()
    foreach ($item in $items) {
        if ((& $item.Check).Ok) { continue }
        Write-Host ''
        Write-Host "'$($item.Name)' 이(가) 필요합니다. ($($item.Why))" -ForegroundColor Yellow
        if (-not (Read-YesNo "'$($item.Name)' 을(를) 지금 설치할까요?")) {
            $declined += $item
            continue
        }
        Write-Step "설치: $($item.Name)"
        try {
            & $item.Install | Out-Null
        } catch {
            Write-Host "[실패] $($item.Name) 설치 중 오류: $($_.Exception.Message)" -ForegroundColor Red
            $declined += $item
        }
    }

    # -DryRun 은 실제로 설치하지 않았으므로 설치 후 재점검을 하지 않는다
    if ($DryRun) { return $true }

    # 설치 후 재점검
    $stillMissing = @($items | Where-Object { -not (& $_.Check).Ok })
    if ($stillMissing.Count -eq 0) {
        Write-Host ''
        Write-Host '모든 빌드 도구가 준비되었습니다.' -ForegroundColor Green
        return $true
    }
    Write-Host ''
    Write-Host '다음 도구가 아직 준비되지 않아 빌드를 진행할 수 없습니다:' -ForegroundColor Red
    foreach ($item in $stillMissing) {
        Write-Host "  - $($item.Name)" -ForegroundColor Red
        Write-Host "    직접 설치: $($item.Manual)"
    }
    if ($declined.Count -eq 0) {
        Write-Info '방금 설치한 도구가 인식되지 않으면 새 터미널에서 이 스크립트를 다시 실행해 보세요.'
    }
    return $false
}

# 선택된 VS 의 개발자 환경을 이 세션에 불러온다 (cl.exe, 링커, SDK 경로를 확정해 도구 선택이 흔들리지 않게 한다)
function Enter-VsDevEnvironment {
    $vsPath = Get-VsInstallPath $RequiredVsComponents
    if (-not $vsPath) { Write-Warn 'Visual Studio 설치 경로를 찾지 못해 개발자 환경 불러오기를 건너뜁니다.'; return }
    $launch = Join-Path $vsPath 'Common7\Tools\Launch-VsDevShell.ps1'
    if (-not (Test-Path $launch)) { Write-Warn "Launch-VsDevShell.ps1 이 없어 개발자 환경 불러오기를 건너뜁니다: $vsPath"; return }
    Write-Info "Visual Studio 개발자 환경 불러오기: $vsPath"
    $env:VSCMD_SKIP_SENDTELEMETRY = '1'
    # VS 개발자 환경 스크립트가 vswhere.exe 를 이름만으로 호출하므로 Installer 폴더를 PATH 에 넣어 둔다
    if (($env:Path -split ';') -notcontains $VsInstallerDir) { $env:Path = "$VsInstallerDir;$env:Path" }
    # 스크립트가 현재 폴더를 바꿀 수 있으므로 끝난 뒤 되돌린다
    Push-Location
    try {
        & $launch -Arch $VsArchName[$Arch] -HostArch $VsArchName[$OsArch] -VsInstallationPath $vsPath -SkipAutomaticLocation | Out-Null
    } catch {
        Write-Warn "개발자 환경을 불러오지 못했습니다. 계속 진행합니다: $($_.Exception.Message)"
    } finally {
        Pop-Location
    }
}

# ---------------------------------------------------------------------------
# 빌드와 패키징
# ---------------------------------------------------------------------------

# cargo 산출물 폴더 (교차 빌드이면 target\<triple>\ 아래)
function Get-CargoOutDir {
    $profileDir = $BuildProfile.ToLower()
    if ($IsCrossBuild) { return Join-Path $RepoRoot "target\$TargetTriple\$profileDir" }
    return Join-Path $RepoRoot "target\$profileDir"
}

# doneditor.exe(GUI) 와 cli.exe 를 빌드한다
function Invoke-Build {
    Param([string]$Channel)
    Write-Step "빌드 ($BuildProfile, $TargetTriple, 채널 $Channel)"
    # 릴리스 채널은 exe 아이콘(stable=파란색)과 앱 식별자를 정한다. 지정하지 않으면 dev 아이콘이 들어간다.
    $env:RELEASE_CHANNEL = $Channel
    $env:GRAM_RELEASE_CHANNEL = $Channel
    $cargoArgs = @('build', '--package', 'gram', '--package', 'cli')
    if ($BuildProfile -eq 'Release') { $cargoArgs += '--release' }
    if ($IsCrossBuild) { $cargoArgs += @('--target', $TargetTriple) }
    Push-Location $RepoRoot
    try {
        Invoke-Native 'cargo' $cargoArgs
    } finally {
        Pop-Location
    }
}

# 배포 폴더(스테이징)를 구성한다. 안에 들어간 파일 수를 돌려준다.
function New-PackageDirectory {
    Param([string]$PackageDir, [string]$Version, [string]$Channel, [string]$CargoOutDir)
    $guiExe = Join-Path $CargoOutDir 'doneditor.exe'
    $cliExe = Join-Path $CargoOutDir 'cli.exe'
    # 필수 산출물이 없으면 바로 알린다
    foreach ($exe in @($guiExe, $cliExe)) {
        if (-not (Test-Path $exe)) { throw "빌드 산출물을 찾을 수 없습니다: $exe" }
    }

    New-Item -ItemType Directory -Force -Path (Join-Path $PackageDir 'bin') | Out-Null
    # GUI 는 루트에, CLI 는 bin\ 에 둔다 (앱이 bin\doneditor.exe 를 CLI 위치로 찾는다 - crates/util)
    Copy-Item $guiExe (Join-Path $PackageDir 'doneditor.exe')
    Copy-Item $cliExe (Join-Path $PackageDir 'bin\doneditor.exe')

    # 라이선스 고지: GPL-3.0 전문, 원저작(Gram/Zed) 고지가 담긴 licenses.md, 제3자 라이선스 본문, 글꼴 라이선스
    Copy-Item (Join-Path $RepoRoot 'LICENSE') $PackageDir
    Copy-Item (Join-Path $RepoRoot 'assets\licenses.md') $PackageDir
    $licensesDir = Join-Path $PackageDir 'licenses'
    New-Item -ItemType Directory -Force -Path $licensesDir | Out-Null
    Copy-Item (Join-Path $RepoRoot 'assets\licenses\*') $licensesDir -Recurse
    Copy-Item (Join-Path $RepoRoot 'assets\fonts\D2Coding-LICENSE') $licensesDir
    Copy-Item (Join-Path $RepoRoot 'README.md') $PackageDir

    New-SourceInfoFile (Join-Path $PackageDir 'SOURCE.txt') $Version $Channel
    return @(Get-ChildItem $PackageDir -Recurse -File).Count
}

# git 정보를 한 줄로 얻는다. 얻지 못하면 $null.
function Get-GitLine {
    Param([string[]]$GitArguments)
    $out = Get-NativeOutput 'git' (@('-C', $RepoRoot) + $GitArguments)
    if (-not $out) { return $null }
    return ([string]($out -join "`n")).Trim()
}

# 소스 입수 방법과 빌드 정보를 담은 SOURCE.txt 를 만든다 (GPL-3.0 의 소스 제공 의무 안내)
function New-SourceInfoFile {
    Param([string]$Path, [string]$Version, [string]$Channel)
    $commit = Get-GitLine @('rev-parse', 'HEAD')
    $dirty = Get-GitLine @('status', '--porcelain')
    $remote = Get-GitLine @('remote', 'get-url', 'origin')
    $commitText = if ($commit) { $commit } else { '(알 수 없음 - git 정보 없음)' }
    $dirtyText = if ($dirty) { '예 - 커밋되지 않은 변경이 있는 상태에서 빌드됨' } else { '아니오' }
    $remoteText = if ($remote) { $remote } else { '(지정되지 않음) 이 프로그램을 배포한 사람에게 소스 코드를 요청하세요.' }

    $lines = @(
        'Don Editor - 소스 코드 안내 / Source Code Information',
        '======================================================',
        '',
        "제품        : Don Editor $Version ($Channel)",
        "대상        : windows-$Arch ($BuildProfile)",
        "빌드 일시   : $((Get-Date).ToString('yyyy-MM-dd HH:mm:ss'))",
        "소스 커밋   : $commitText",
        "미커밋 변경 : $dirtyText",
        "소스 저장소 : $remoteText",
        '',
        'Don Editor 는 GramEditor(https://codeberg.org/GramEditor/gram)를 수정한 프로그램이며,',
        'GramEditor 는 Zed 편집기를 하드포크한 프로젝트입니다. 이 프로그램은 GNU General Public',
        'License v3.0 (또는 그 이후 버전)으로 배포됩니다. 라이선스 전문은 LICENSE 파일,',
        '원저작자 고지와 제3자 라이선스는 licenses.md 와 licenses\ 폴더를 참고하세요.',
        '',
        'GPL-3.0 에 따라, 이 바이너리에 대응하는 소스 코드를 받을 권리가 있습니다.',
        '위 "소스 저장소" 주소 또는 이 파일과 함께 배포된 소스 zip(DonEditor-<버전>-source.zip)을 확인하세요.',
        ''
    )
    # 한글이 깨지지 않도록 BOM 이 있는 UTF-8 로 저장한다 (메모장 등에서 바로 열 수 있게)
    [System.IO.File]::WriteAllText($Path, ($lines -join "`r`n"), (New-Object System.Text.UTF8Encoding $true))
}

# 폴더를 zip 으로 압축한다. zip 안에는 폴더 이름이 최상위로 들어간다.
function New-ZipArchive {
    Param([string]$SourceDir, [string]$ZipPath)
    # ZipFile 은 FileSystem 어셈블리에, ZipArchiveMode 등은 Compression 어셈블리에 있다
    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
    # CreateFromDirectory 는 Windows PowerShell 5.1 에서 항목 이름에 '\' 를 써서 다른 OS 의 압축 해제 도구와
    # 호환되지 않으므로, 항목 이름을 zip 규격대로 '/' 로 직접 지정해 한 파일씩 넣는다
    $baseName = Split-Path -Leaf $SourceDir
    $archive = [System.IO.Compression.ZipFile]::Open($ZipPath, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        # 폴더 안의 모든 파일을 최상위 폴더 이름 아래에 추가한다
        foreach ($file in (Get-ChildItem $SourceDir -Recurse -File)) {
            $relative = $file.FullName.Substring($SourceDir.Length).TrimStart('\') -replace '\\', '/'
            [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $file.FullName,
                "$baseName/$relative", [System.IO.Compression.CompressionLevel]::Optimal)
        }
    } finally {
        $archive.Dispose()
    }
}

# zip 의 SHA256 을 계산해 sha256sum 호환 형식("<해시> *<파일명>")으로 .sha256 파일에 쓴다
function New-HashFile {
    Param([string]$ZipPath)
    $hash = (Get-FileHash -Algorithm SHA256 -Path $ZipPath).Hash.ToLower()
    $hashPath = "$ZipPath.sha256"
    [System.IO.File]::WriteAllText($hashPath, "$hash *$([System.IO.Path]::GetFileName($ZipPath))`n", (New-Object System.Text.UTF8Encoding $false))
    return $hash
}

# ---------------------------------------------------------------------------
# 본문
# ---------------------------------------------------------------------------

if ($Help) { Show-Usage; exit 0 }

$startTime = Get-Date
try {
    # 상대 경로의 OutDir 은 저장소 루트 기준으로 해석한다
    if (-not [System.IO.Path]::IsPathRooted($OutDir)) { $OutDir = Join-Path $RepoRoot $OutDir }
    $OutDir = [System.IO.Path]::GetFullPath($OutDir)

    $version = Get-WorkspaceVersion
    $channel = Get-ReleaseChannel
    # 점검 항목(Get-Prerequisites)이 참조하는 Rust 툴체인 정보
    $RustToolchain = Get-RustToolchainSpec
    # 디버그 빌드 zip 이 릴리스 zip 과 혼동되지 않도록 이름에 표시한다
    $suffix = if ($BuildProfile -eq 'Debug') { '-debug' } else { '' }
    $packageName = "$ProductName-$version-windows-$Arch$suffix"

    Write-Host "Don Editor 배포 zip 생성: $packageName" -ForegroundColor Cyan
    Write-Info "저장소 : $RepoRoot"
    Write-Info "결과물 : $OutDir"

    # 1) 빌드 도구 점검
    if (-not $SkipCheck) {
        $ready = Confirm-Prerequisites
        if ($CheckOnly) {
            if ($ready) { Write-Host '점검 결과: 모든 빌드 도구가 준비되어 있습니다.' -ForegroundColor Green; exit 0 }
            Write-Host '점검 결과: 준비되지 않은 도구가 있습니다. -CheckOnly 없이 실행하면 설치 여부를 묻습니다.' -ForegroundColor Yellow
            exit 1
        }
        if (-not $ready) { exit 1 }
        if ($DryRun) {
            Write-Host ''
            Write-Host '-DryRun: 설치 명령만 출력했으며 실제 설치와 빌드는 하지 않았습니다.' -ForegroundColor Yellow
            exit 0
        }
        Enter-VsDevEnvironment
    } elseif ($CheckOnly -or $DryRun) {
        Write-Warn '-SkipCheck 와 함께 쓰면 -CheckOnly / -DryRun 이 할 일이 없습니다.'
        exit 0
    }

    # 2) 빌드
    if ($SkipBuild) { Write-Step '빌드 건너뜀 (-SkipBuild): 기존 산출물 사용' }
    else { Invoke-Build $channel }

    # 3) 스테이징 폴더 구성
    Write-Step '배포 폴더 구성'
    $stageRoot = Join-Path $OutDir 'staging'
    $packageDir = Join-Path $stageRoot $packageName
    # 지우기 전에 대상이 결과물 폴더 안쪽인지 확인한다 (경로 오류로 엉뚱한 곳이 지워지는 것을 막는다)
    if ((Test-Path $stageRoot) -and $stageRoot.StartsWith($OutDir, [System.StringComparison]::OrdinalIgnoreCase)) {
        Remove-Item $stageRoot -Recurse -Force
    }
    $fileCount = New-PackageDirectory $packageDir $version $channel (Get-CargoOutDir)
    Write-Info "파일 $fileCount 개 구성: $packageDir"

    # 4) zip + SHA256
    Write-Step 'zip 압축'
    $zipPath = Join-Path $OutDir "$packageName.zip"
    New-ZipArchive $packageDir $zipPath
    $hash = New-HashFile $zipPath
    $zipSizeMb = [Math]::Round((Get-Item $zipPath).Length / 1MB, 1)

    # 5) 소스 zip (선택)
    $sourceZip = $null
    if ($IncludeSource) {
        Write-Step '소스 zip (git archive)'
        if (Get-GitLine @('status', '--porcelain')) {
            Write-Warn '커밋되지 않은 변경이 있습니다. 소스 zip 에는 마지막 커밋의 내용만 들어가므로 바이너리와 다를 수 있습니다.'
        }
        $sourceZip = Join-Path $OutDir "$ProductName-$version-source.zip"
        if (Test-Path $sourceZip) { Remove-Item $sourceZip -Force }
        Invoke-Native 'git' @('-C', $RepoRoot, 'archive', '--format=zip', "--prefix=$ProductName-$version-source/", '-o', $sourceZip, 'HEAD')
    }

    # 스테이징 폴더는 zip 이 만들어졌으므로 정리한다
    Remove-Item $stageRoot -Recurse -Force

    $elapsed = (Get-Date) - $startTime
    Write-Host ''
    Write-Host '완료' -ForegroundColor Green
    Write-Info "zip    : $zipPath ($zipSizeMb MB)"
    Write-Info "SHA256 : $hash"
    if ($sourceZip) { Write-Info "소스   : $sourceZip" }
    Write-Info ("소요   : {0:mm\:ss}" -f $elapsed)
    exit 0
} catch {
    Write-Host ''
    Write-Host "[실패] $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
