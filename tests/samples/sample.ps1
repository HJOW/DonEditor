# 한 줄 주석
<# 블록 주석 #>
param([string]$Name = "Don Editor", [int]$Count = 3)

function Get-Greeting {
    param([string]$Who)
    return "안녕하세요, $Who!"
}

$items = @(1, 2, 3.5)
foreach ($item in $items) {
    if ($item -gt 1 -and $Name -ne $null) {
        Write-Host (Get-Greeting -Who $Name) $item
    }
}
$hash = @{ Key = 'value'; Number = 42 }
try {
    Get-ChildItem -Path . | Where-Object { $_.Length -gt 0 }
} catch {
    Write-Error $_.Exception.Message
}
