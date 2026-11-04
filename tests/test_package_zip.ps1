#Requires -Version 5.1
# [DonEditor] zip 구성/현재 소스 동봉을 실제 압축 파일로 검사한다. 설치/컴파일은 하지 않는다.
$ErrorActionPreference = 'Stop'

# 스크립트 본문을 실행하지 않고 함수 정의만 읽는다.
$repository = Split-Path -Parent $PSScriptRoot
$scriptFile = Join-Path $repository 'script\package-zip.ps1'
$parseErrors = $null
$tokens = $null
$syntax = [System.Management.Automation.Language.Parser]::ParseFile($scriptFile, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count -gt 0) { throw ($parseErrors | Out-String) }
# 모든 최상위 함수는 실제 배포 코드와 동일하게 로드한다.
foreach ($statement in $syntax.EndBlock.Statements) {
    if ($statement -is [System.Management.Automation.Language.FunctionDefinitionAst]) {
        Invoke-Expression $statement.Extent.Text
    }
}

# 실패 시에만 명확한 메시지를 표시한다.
function Assert-Package {
    Param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

# 테스트 파일을 BOM 없이 UTF-8로 저장한다.
function Write-TestFile {
    Param([string]$RelativePath, [string]$Value)
    $path = Join-Path $RepoRoot $RelativePath
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $path) | Out-Null
    [System.IO.File]::WriteAllText($path, $Value, (New-Object System.Text.UTF8Encoding $false))
}

# 유일한 임시 저장소를 만들고 삭제 전에 항상 절대 경로와 부모 폴더를 확인한다.
$temporaryParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\') + '\'
$RepoRoot = Join-Path $temporaryParent ('doneditor package test ' + [guid]::NewGuid().ToString('N'))
$OutDir = Join-Path $RepoRoot 'dist'
$ProductName = 'DonEditor'
$Arch = 'x86_64'
$BuildProfile = 'Debug'
$DryRun = $false
try {
    New-Item -ItemType Directory -Path $RepoRoot | Out-Null
    Write-TestFile 'LICENSE' 'GPL-3.0 test'
    # 배포 파일명이 제품의 워크스페이스 버전을 사용하는지 확인한다.
    Write-TestFile 'Cargo.toml' "[workspace.package]`nversion = `"0.1.0`"`n"
    Assert-Package ((Get-WorkspaceVersion) -eq '0.1.0') '워크스페이스 제품 버전 불일치'
    Write-TestFile 'README.md' 'Don Editor test'
    Write-TestFile 'assets\licenses.md' 'GramEditor/Zed notices'
    Write-TestFile 'assets\licenses\MIT' 'MIT test'
    Write-TestFile 'assets\fonts\D2Coding-LICENSE' 'OFL test'
    Write-TestFile '.gitignore' "dist/`ntarget/`n.env`n"
    Write-TestFile 'tracked.rs' '// 최초 소스'
    & git -C $RepoRoot init --quiet
    if ($LASTEXITCODE -ne 0) { throw '테스트 git init 실패' }
    & git -C $RepoRoot add .
    if ($LASTEXITCODE -ne 0) { throw '테스트 git add 실패' }
    # 인덱스와 다른 수정 내용 및 새 한글 파일명도 배포 소스에 들어가야 한다.
    Write-TestFile 'tracked.rs' '// 수정된 한국어 소스'
    Write-TestFile '새 소스.rs' '// 미커밋 새 소스'
    Write-TestFile '.env' 'API_KEY=secret'
    Write-TestFile 'target\debug\doneditor.exe' 'test GUI binary'
    Write-TestFile 'target\debug\cli.exe' 'test CLI binary'
    Write-TestFile 'target\debug\don-mcp.exe' 'test MCP binary'
    $packageDir = Join-Path $OutDir 'staging\DonEditor-0.1.0-windows-x86_64-debug'
    $count = New-PackageDirectory $packageDir '0.1.0' 'stable' (Join-Path $RepoRoot 'target\debug')
    Assert-Package ($count -ge 10) '배포 파일 수가 예상보다 적음'
    Assert-Package (Test-Path (Join-Path $packageDir 'bin\don-mcp.exe')) 'MCP stdio 브릿지 누락'
    Assert-Package (Test-Path (Join-Path $packageDir 'licenses\D2Coding-LICENSE')) '글꼴 라이선스 누락'
    $sourceArchive = Join-Path $packageDir 'source\DonEditor-0.1.0-source.zip'
    $archive = [System.IO.Compression.ZipFile]::OpenRead($sourceArchive)
    try {
        $names = @($archive.Entries | ForEach-Object { $_.FullName })
        Assert-Package ($names -contains 'DonEditor-0.1.0-source/새 소스.rs') '미커밋 한글 파일명 누락'
        Assert-Package (-not ($names -contains 'DonEditor-0.1.0-source/.env')) 'gitignore 비밀 파일 포함됨'
        Assert-Package (@($names | Where-Object { $_ -match '/(target|dist)/' }).Count -eq 0) '빌드/출력 폴더가 소스에 포함됨'
        $entry = $archive.GetEntry('DonEditor-0.1.0-source/tracked.rs')
        $reader = New-Object System.IO.StreamReader ($entry.Open())
        try { Assert-Package ($reader.ReadToEnd() -eq '// 수정된 한국어 소스') '미커밋 수정 내용 불일치' }
        finally { $reader.Dispose() }
    } finally { $archive.Dispose() }

    $zipPath = Join-Path $OutDir 'test-package.zip'
    New-ZipArchive $packageDir $zipPath
    # 외부 cmdlet이 없거나 섀도잉되어도 실제 SHA-256 값을 생성해야 한다.
    function Get-FileHash { throw 'Get-FileHash를 사용하면 안 됩니다' }
    $knownPath = Join-Path $OutDir 'known-hash.txt'
    [System.IO.File]::WriteAllText($knownPath, 'abc', (New-Object System.Text.UTF8Encoding $false))
    $knownDigest = New-HashFile $knownPath
    Assert-Package ($knownDigest -eq 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad') '알려진 SHA256 값 불일치'
    $digest = New-HashFile $zipPath
    Remove-Item Function:\Get-FileHash
    Assert-Package ((Get-Content "$zipPath.sha256" -Raw) -eq "$digest *test-package.zip`n") 'SHA256 파일 불일치'
    $archive = [System.IO.Compression.ZipFile]::OpenRead($zipPath)
    try {
        $names = @($archive.Entries | ForEach-Object { $_.FullName })
        Assert-Package (@($names | Where-Object { $_.Contains('\') }).Count -eq 0) 'zip 항목에 역슬래시가 있음'
        Assert-Package (@($names | Where-Object { $_.EndsWith('/source/DonEditor-0.1.0-source.zip') }).Count -eq 1) '배포본에 소스 zip이 없음'
        Assert-Package (@($names | Where-Object { $_.EndsWith('/bin/don-mcp.exe') }).Count -eq 1) '배포본에 MCP가 없음'
    } finally { $archive.Dispose() }

    # 필수 MCP 바이너리가 없을 때는 부분 배포본을 만들지 않는다.
    Remove-Item -LiteralPath (Join-Path $RepoRoot 'target\debug\don-mcp.exe')
    $missingFailed = $false
    try { New-PackageDirectory (Join-Path $OutDir 'missing') '0.1.0' 'stable' (Join-Path $RepoRoot 'target\debug') | Out-Null }
    catch { $missingFailed = $_.Exception.Message -like '*don-mcp.exe*' }
    Assert-Package $missingFailed 'MCP 누락을 허용함'

    # 버전 표시를 통한 암묵 설치를 유발하는 rustup을 모사한다.
    $script:rustQueries = @()
    function Get-NativeOutput {
        Param([string]$File, [string[]]$Arguments)
        if ($File -ne 'rustup') { throw '이 검증에서는 rustup만 조회해야 합니다.' }
        $script:rustQueries += ($Arguments -join ' ')
        if ($Arguments[0] -eq '--version') { throw '활성 툴체인의 암묵 설치가 발생했습니다.' }
        if ($Arguments[0] -eq '--help') { return , @('rustup help') }
        if ($Arguments[0] -eq 'toolchain') { return , @('1.99.1-x86_64-pc-windows-msvc') }
        if ($Arguments[0] -eq 'target') { return , @('wasm32-wasip2') }
        throw '예상하지 못한 rustup 조회'
    }
    $RustToolchain = [pscustomobject]@{ Channel = '1.99.1'; Targets = @('wasm32-wasip2') }
    $IsCrossBuild = $false
    $items = Get-Prerequisites
    Assert-Package ([bool](& $items[1].Check).Ok) 'rustup 안전 조회 실패'
    Assert-Package ([bool](& $items[2].Check).Ok) 'Rust 툴체인 안전 조회 실패'
    Assert-Package ($script:rustQueries -notcontains '--version') 'rustup 버전 조회가 남아 있음'
    Write-Host 'zip 검증 통과: MCP, 라이선스, 미커밋 소스/한글 파일명, 비밀 파일 제외, 해시, 경로, 누락 실패, 안전한 Rust 조회'
} finally {
    $resolved = [System.IO.Path]::GetFullPath($RepoRoot)
    if ($resolved.StartsWith($temporaryParent, [System.StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $resolved).StartsWith('doneditor package test ') -and (Test-Path -LiteralPath $resolved)) {
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
