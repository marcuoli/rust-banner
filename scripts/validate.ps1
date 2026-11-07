param([string]$ProjectRoot = 'V:\Development\rust-banner')
$ErrorActionPreference = 'Stop'
$checkout = Split-Path $PSScriptRoot -Parent
$env:TEMP = Join-Path $ProjectRoot 'tmp\issue-1'
$env:TMP = $env:TEMP
$env:TMPDIR = $env:TEMP
$env:RUST_BANNER_FIXTURE_ROOT = Join-Path $env:TEMP 'fixtures'
$env:CARGO_HOME = Join-Path $ProjectRoot 'caches\cargo'
$env:CARGO_TARGET_DIR = Join-Path $ProjectRoot 'artifacts\build'
$env:RUSTUP_HOME = 'V:\Development\tools\rust\rustup'
$env:RUSTUP_TOOLCHAIN = '1.99.1-x86_64-pc-windows-msvc'
$env:PATH = 'V:\Development\tools\rust\cargo\bin;V:\Development\tools\rust\rustup\toolchains\1.99.1-x86_64-pc-windows-msvc\bin;' + $env:PATH
$logs = Join-Path $ProjectRoot ('artifacts\logs\validation-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
@($env:TEMP,$env:RUST_BANNER_FIXTURE_ROOT,$env:CARGO_HOME,$env:CARGO_TARGET_DIR,$logs) | ForEach-Object { New-Item -ItemType Directory -Path $_ -Force | Out-Null }
Push-Location $checkout
try {
    & rustc -vV 2>&1 | Tee-Object -FilePath (Join-Path $logs 'compiler.log')
    if ($LASTEXITCODE -ne 0) { throw 'rustc failed' }
    $commands = @(
        @{Name='fmt'; Args=@('fmt','--all','--','--check')},
        @{Name='clippy'; Args=@('clippy','--workspace','--all-targets','--locked','--','-D','warnings')},
        @{Name='test'; Args=@('test','--workspace','--locked')},
        @{Name='package'; Args=@('package','--locked','--allow-dirty')}
    )
    foreach ($check in $commands) {
        & cargo @($check.Args) 2>&1 | Tee-Object -FilePath (Join-Path $logs ($check.Name + '.log'))
        if ($LASTEXITCODE -ne 0) { throw ($check.Name + ' failed; logs: ' + $logs) }
    }
    & cargo tree --locked 2>&1 | Tee-Object -FilePath (Join-Path $logs 'dependencies.log')
    if ($LASTEXITCODE -ne 0) { throw 'dependency graph failed' }
    Write-Output ('Validation passed; full logs: ' + $logs)
} finally { Pop-Location }
