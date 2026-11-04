#Requires -Version 5.1
# [DonEditor] Windows 번호 선택/설치 테스트. 외부 설치 명령을 모두 가짜 함수로 대체한다.
# 실행: powershell.exe -NoProfile -ExecutionPolicy Bypass -File tests\test_install_build_tools.ps1
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\script\install-build-tools.ps1')

# 별도 테스트 프레임워크 없이 조건이 틀리면 즉시 실패한다.
function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw "검증 실패: $Message" }
}

# TOML에 선언된 실제 구성 요소가 설치 명령에 모두 전달되는지 확인한다.
$script:RustSpec = Read-RustSpec
$script:CapturedArguments = @()
function Invoke-InstallCommand {
    param([string]$File, [string[]]$Arguments)
    $script:CapturedArguments = @($File) + $Arguments
}
Install-ProjectRust
Assert-True ($CapturedArguments -contains 'rustfmt,clippy,rust-analyzer') 'Rust 추가 구성 요소'
Assert-True ($CapturedArguments -contains 'wasm32-wasip2') 'Rust 추가 타깃'
Assert-True ($CapturedArguments -contains 'minimal') 'Rust 프로필'

# rustup이 설치된 상태에서도 구성 요소/타깃 누락을 발견하는지 검사한다.
$script:FakeComponents = @('rustfmt', 'clippy', 'rust-analyzer')
$script:FakeTargets = @('wasm32-wasip2')
function Get-Command { param([string]$Name, [string]$ErrorAction) return $true }
function Get-QueryOutput {
    param([string]$File, [string[]]$Arguments)
    if ($File -eq 'rustup' -and $Arguments -contains '--version') { throw '활성 툴체인 암묵 설치 경로를 호출했습니다.' }
    if ($File -eq 'rustup' -and $Arguments -contains '--help') { return 'rustup help' }
    if ($Arguments[0] -eq 'toolchain') { return $RustSpec.Name }
    if ($Arguments[0] -eq 'component') {
        # rustup의 실제 출력처럼 호스트 이름을 붙여 반환한다.
        foreach ($component in $FakeComponents) { "$component-$RustHost" }
    }
    if ($Arguments[0] -eq 'target') { return $FakeTargets }
}
Assert-True (Test-RustToolchain) '준비된 툴체인 조회'
$script:FakeComponents = @('clippy', 'rust-analyzer')
Assert-True (-not (Test-RustToolchain)) '누락된 rustfmt 감지'
$script:FakeComponents = @('rustfmt', 'clippy', 'rust-analyzer')
$script:FakeTargets = @()
Assert-True (-not (Test-RustToolchain)) '누락된 타깃 감지'

# 상태 목록 생성 자체가 활성 Rust 컴파일러를 조회하지 않는지 검증한다.
function Get-VisualStudio { param([string[]]$Components) return $null }
function Test-WindowsSdk { return $false }
$safeItems = @(Get-ToolItems)
Assert-True ($safeItems[4].Ready) 'rustup 도움말로 설치 여부 확인'

# 메뉴 테스트에서는 모든 상태와 설치 동작을 메모리 안에서만 처리한다.
function Update-ToolPath {}
function Write-Host {
    param($Object, $ForegroundColor)
    $script:Messages += [string]$Object
}
function Get-ToolItems {
    # 여섯 도구의 준비 상태를 테스트별로 바꾼다.
    for ($index = 0; $index -lt 6; $index++) {
        [pscustomobject]@{ Name = "도구$index"; Why = '테스트'; Ready = $script:ReadyState[$index] }
    }
}
function Read-Host {
    param([string]$Prompt)
    if ($script:Answers.Count) { return $script:Answers.Dequeue() }
    return $null
}
function Install-WingetPackage {
    param([string]$Id, [string]$Override)
    $script:Installs += $Id
    if ($script:FailInstall) { throw '가짜 설치 실패' }
    if ($Id -eq 'Git.Git') { $script:ReadyState[0] = $true }
    if ($Id -eq 'Kitware.CMake') { $script:ReadyState[1] = $true }
}
function Install-VisualStudioComponents {
    param([bool]$Tools, [bool]$Sdk)
    $script:Installs += "VS Tools=$Tools Sdk=$Sdk"
    if ($Tools) { $script:ReadyState[2] = $true }
    if ($Sdk) { $script:ReadyState[3] = $true }
}
function Install-RustupOnly { $script:Installs += 'rustup'; $script:ReadyState[4] = $true }
function Install-ProjectRust { $script:Installs += 'toolchain'; $script:ReadyState[5] = $true }

# 각 시나리오의 입력·상태·모의 설치 기록을 초기화한다.
function Run-Scenario {
    param([string[]]$InputLines, [bool[]]$InitialState = (@($false) * 6), [switch]$Check, [switch]$Preview, [switch]$Fail)
    $script:Answers = New-Object 'System.Collections.Generic.Queue[string]'
    # 사용자 입력 순서를 그대로 큐에 보관한다.
    foreach ($line in $InputLines) { $script:Answers.Enqueue($line) }
    $script:ReadyState = $InitialState.Clone()
    $script:Installs = @()
    $script:Messages = @()
    $script:FailInstall = [bool]$Fail
    $script:CheckOnly = [bool]$Check
    $script:DryRun = [bool]$Preview
    Invoke-ToolInstaller
}

Assert-True ((Run-Scenario @('0')) -eq 0) '취소 종료 코드'
Assert-True ($Installs.Count -eq 0) '취소 시 미설치'
Assert-True ((Run-Scenario @()) -eq 0) 'EOF 종료'
Assert-True ($Installs.Count -eq 0) 'EOF 시 미설치'
Assert-True ((Run-Scenario @('A', 'N', 'I', '0')) -eq 0) '전체 선택 해제'
Assert-True ($Installs.Count -eq 0) '선택 해제 시 미설치'
Assert-True ((Run-Scenario @('1', '1', 'I', '0')) -eq 0) '번호 재선택 해제'
Assert-True ($Installs.Count -eq 0) '번호 재선택 시 미설치'
Assert-True ((Run-Scenario @('1,99', 'I', '0')) -eq 0) '잘못된 입력 거부'
Assert-True ($Installs.Count -eq 0) '잘못된 번호가 섞이면 선택 불변'
Assert-True ((Run-Scenario @('1,1', 'I')) -eq 0) '중복 번호 처리'
Assert-True (($Installs -join ',') -eq 'Git.Git') '선택한 Git만 설치'
Assert-True ((Run-Scenario @('3 4', 'I')) -eq 0) 'VS 및 SDK 선택'
Assert-True (($Installs -join ',') -eq 'VS Tools=True Sdk=True') 'VS 구성 요소 설치 중복 방지'
Assert-True ((Run-Scenario @('4', 'I')) -eq 0) 'SDK 개별 선택'
Assert-True (($Installs -join ',') -eq 'VS Tools=False Sdk=True') '미선택 MSVC 제외'
Assert-True ((Run-Scenario @('6', 'I', '0')) -eq 0) '미선택 rustup 의존성 안내'
Assert-True ($Installs.Count -eq 0) '의존성 미선택 시 설치 안 함'
Assert-True ((Run-Scenario @('5,6', 'I')) -eq 0) 'rustup과 툴체인 선택'
Assert-True (($Installs -join ',') -eq 'rustup,toolchain') 'Rust 설치 순서'
Assert-True ((Run-Scenario @('1', 'I', '0') -InitialState (@($true) * 6)) -eq 0) '준비된 항목 제외'
Assert-True ($Installs.Count -eq 0) '준비된 도구 재설치 안 함'
Assert-True ((Run-Scenario @() -Check) -eq 1) '미준비 점검 종료 코드'
Assert-True ($Installs.Count -eq 0) '조회 모드 미설치'
Assert-True ((Run-Scenario @() -Check -InitialState (@($true) * 6)) -eq 0) '준비된 점검 종료 코드'
Assert-True ((Run-Scenario @('1', 'I') -Fail) -eq 1) '설치 실패 종료 코드'
Assert-True ((Run-Scenario @('1', 'I') -Preview) -eq 0) '모의 실행 종료 코드'
[Console]::WriteLine('Windows 설치 스크립트 검증 통과 (Rust 구성 요소 및 메뉴 15개 시나리오).')
