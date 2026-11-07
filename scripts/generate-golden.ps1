param(
    [Parameter(Mandatory)][string]$ReferenceRoot,
    [string]$ProjectRoot = 'V:\Development\rust-banner'
)
$ErrorActionPreference = 'Stop'
$checkout = Split-Path $PSScriptRoot -Parent
$scratch = Join-Path $ProjectRoot 'tmp\issue-1\go-golden'
$reference = Join-Path $scratch 'banner'
$env:TEMP = Join-Path $ProjectRoot 'tmp\issue-1'
$env:TMP = $env:TEMP
$env:TMPDIR = $env:TEMP
$env:GOTMPDIR = Join-Path $env:TEMP 'go-build'
$env:GOCACHE = Join-Path $ProjectRoot 'caches\go-build'
$env:GOMODCACHE = Join-Path $ProjectRoot 'caches\go-modules'
@($reference,$env:GOTMPDIR,$env:GOCACHE,$env:GOMODCACHE) | ForEach-Object { New-Item -ItemType Directory -Path $_ -Force | Out-Null }
# Copy only owned reference library files; no third-party source or private inputs.
Get-ChildItem -LiteralPath $ReferenceRoot -Filter '*.go' | Where-Object Name -NotLike '*_test.go' | Copy-Item -Destination $reference
Copy-Item -LiteralPath (Join-Path $ReferenceRoot 'locales') -Destination $reference -Recurse -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'golden\main.go') -Destination (Join-Path $scratch 'main.go')
[IO.File]::WriteAllText((Join-Path $scratch 'go.mod'), "module golden-reference`n`ngo 1.22`n", [Text.UTF8Encoding]::new($false))
$env:GOWORK = 'off'
$go = Get-Command go -ErrorAction Stop
& $go.Source -C $scratch run . --locales (Join-Path $reference 'locales') --output (Join-Path $ProjectRoot 'artifacts\go-parity.json')
if ($LASTEXITCODE -ne 0) { throw 'Go golden generation failed' }
# Deliberately promote the immutable compatibility snapshot from external evidence.
Copy-Item -LiteralPath (Join-Path $ProjectRoot 'artifacts\go-parity.json') -Destination (Join-Path $checkout 'tests\fixtures\go-parity.json')
