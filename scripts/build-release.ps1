[CmdletBinding()]
param([switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskVersion = (Select-String -LiteralPath (Join-Path $taskRoot 'Cargo.toml') -Pattern '^version = "([0-9.]+)"').Matches.Groups[1].Value
if (!$taskVersion) { throw 'Package version missing' }
if (!$SkipBuild) {
    Push-Location $taskRoot
    try {
        & cargo build --release --locked --offline --bin lecoo-control-center --bin hardware_test
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    } finally { Pop-Location }
}
$taskCompiler = Get-Command ISCC -ErrorAction SilentlyContinue
$taskCompilerPath = if ($taskCompiler) { $taskCompiler.Source } else { Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe' }
if (!(Test-Path -LiteralPath $taskCompilerPath)) { throw 'Inno Setup 6 compiler required to package the installer' }
$taskDist = Join-Path $taskRoot 'dist'
$taskBase = "lecoo-rust-powercontrol-v$taskVersion-windows-x64"
$taskStage = Join-Path $taskDist $taskBase
New-Item -ItemType Directory -Path $taskStage -Force | Out-Null
$taskNames = @('lecoo-control-center.exe', 'hardware_test.exe')
$taskHashes = foreach ($taskName in $taskNames) {
    $taskSource = Join-Path $taskRoot "target\release\$taskName"
    $taskDestination = Join-Path $taskStage $taskName
    Copy-Item -LiteralPath $taskSource -Destination $taskDestination -Force
    Copy-Item -LiteralPath $taskSource -Destination (Join-Path $taskDist $taskName) -Force
    $taskHash = (Get-FileHash -LiteralPath $taskSource -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($taskHash -ne (Get-FileHash -LiteralPath $taskDestination -Algorithm SHA256).Hash.ToLowerInvariant()) { throw "Copy verification failed: $taskName" }
    "$taskHash  $taskName"
}
$taskHashes | Set-Content -LiteralPath (Join-Path $taskStage 'SHA256SUMS.txt') -Encoding ASCII
$taskHashes | Set-Content -LiteralPath (Join-Path $taskDist 'SHA256SUMS.txt') -Encoding ASCII
$taskFiles = @($taskNames + 'SHA256SUMS.txt' | ForEach-Object { Join-Path $taskStage $_ })
$taskArchive = Join-Path $taskDist "$taskBase.zip"
Compress-Archive -LiteralPath $taskFiles -DestinationPath $taskArchive -CompressionLevel Optimal -Force
& $taskCompilerPath "/DAppVersion=$taskVersion" "/O$taskDist" (Join-Path $taskRoot 'installer\setup.iss')
if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
$taskSetup = Join-Path $taskDist "$taskBase-setup.exe"
foreach ($taskFile in @($taskArchive, $taskSetup)) {
    "$((Get-FileHash -LiteralPath $taskFile -Algorithm SHA256).Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($taskFile))" | Set-Content -LiteralPath "$taskFile.sha256" -Encoding ASCII
}
Write-Output "ZIP and installer: $taskDist"
