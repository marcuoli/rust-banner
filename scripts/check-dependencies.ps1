param([string]$ProjectRoot = 'V:\Development\rust-banner')
$ErrorActionPreference = 'Stop'
$checkout = Split-Path $PSScriptRoot -Parent
$env:TEMP = Join-Path $ProjectRoot 'tmp\issue-1'
$env:TMP = $env:TEMP
$env:TMPDIR = $env:TEMP
$env:CARGO_HOME = Join-Path $ProjectRoot 'caches\cargo'
$env:CARGO_TARGET_DIR = Join-Path $ProjectRoot 'artifacts\build'
$env:RUSTUP_HOME = 'V:\Development\tools\rust\rustup'
$env:RUSTUP_TOOLCHAIN = '1.99.1-x86_64-pc-windows-msvc'
$logs = Join-Path $ProjectRoot 'artifacts\logs'
@($env:TEMP,$env:CARGO_HOME,$env:CARGO_TARGET_DIR,$logs) | ForEach-Object { New-Item -ItemType Directory -Path $_ -Force | Out-Null }
Push-Location $checkout
try {
    $metadata = & 'V:\Development\tools\rust\cargo\bin\cargo.exe' metadata --format-version 1 --locked
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
    $metadata = $metadata | ConvertFrom-Json
    $dependencies = @($metadata.packages | Where-Object source | Select-Object name,version,license,repository)
    $dependencies | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $logs 'dependency-metadata.json') -Encoding utf8
    $queries = @($dependencies | ForEach-Object { @{package=@{name=$_.name;ecosystem='crates.io'};version=$_.version} })
    $body = @{queries=$queries} | ConvertTo-Json -Depth 8
    $response = Invoke-RestMethod -Uri 'https://api.osv.dev/v1/querybatch' -Method Post -ContentType 'application/json' -Body $body
    $evidence = @{checkedAt=(Get-Date -AsUTC -Format o);endpoint='https://api.osv.dev/v1/querybatch';dependencies=$dependencies;response=$response}
    $evidence | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $logs 'osv-dependency-check.json') -Encoding utf8
    $findings = @($response.results | Where-Object vulns)
    Write-Output ('Exact locked dependencies checked: ' + $dependencies.Count + '; dependencies with OSV findings: ' + $findings.Count)
    if ($findings.Count -ne 0) { throw 'Dependency advisories found; inspect retained OSV response' }
} finally { Pop-Location }
