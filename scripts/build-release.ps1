[CmdletBinding()]
param([switch]$SkipBuild)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if (!$SkipBuild) {
    $taskCargo = Get-Command cargo -ErrorAction SilentlyContinue
    $taskCargoPath = if ($taskCargo) { $taskCargo.Source } else { Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe' }
    Push-Location $taskRoot
    try {
        & $taskCargoPath build --release --locked --offline --bin lecoo-control-center --bin hardware_test
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed; nothing was published.' }
    } finally { Pop-Location }
}

$taskDist = Join-Path $taskRoot 'dist'
New-Item -ItemType Directory -Path $taskDist -Force | Out-Null
$taskHashes = foreach ($taskName in @('lecoo-control-center.exe', 'hardware_test.exe')) {
    $taskSource = Join-Path $taskRoot (Join-Path 'target\release' $taskName)
    $taskDestination = Join-Path $taskDist $taskName
    Copy-Item -LiteralPath $taskSource -Destination $taskDestination -Force
    $taskSourceHash = (Get-FileHash -LiteralPath $taskSource -Algorithm SHA256).Hash
    $taskDestinationHash = (Get-FileHash -LiteralPath $taskDestination -Algorithm SHA256).Hash
    if ($taskSourceHash -ne $taskDestinationHash) { throw "Copy verification failed for $taskName" }
    "$taskDestinationHash  $taskName"
}
$taskHashes | Set-Content -LiteralPath (Join-Path $taskDist 'SHA256SUMS.txt') -Encoding ASCII
Write-Output "Release files: $taskDist"
