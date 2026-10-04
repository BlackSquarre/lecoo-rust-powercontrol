[CmdletBinding()]
param([switch]$SkipBuild, [switch]$Measure, [string]$ReportPath)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if (!$ReportPath) { $ReportPath = Join-Path $taskRoot 'logs\resource-optimization\verification.json' }
$ReportPath = [IO.Path]::GetFullPath($ReportPath)
$taskDirectory = Split-Path -Parent $ReportPath
New-Item -ItemType Directory -Path $taskDirectory -Force | Out-Null
$taskArtifactsPath = Join-Path $taskRoot 'logs\resource-optimization\test-build.jsonl'
New-Item -ItemType Directory -Path (Split-Path -Parent $taskArtifactsPath) -Force | Out-Null
function Get-ResourceSourceSnapshot {
    $taskSourceFiles = @('Cargo.toml', 'Cargo.lock', 'build.rs', 'app.manifest', '.cargo\config.toml' | ForEach-Object { Join-Path $taskRoot $_ })
    foreach ($taskSourceFolder in @('src', 'tests', 'build_support', 'resources', 'assets\icons')) {
        $taskFolderPath = Join-Path $taskRoot $taskSourceFolder
        if (Test-Path -LiteralPath $taskFolderPath) {
            $taskSourceFiles += @(Get-ChildItem -LiteralPath $taskFolderPath -Recurse -File | Where-Object { $_.Extension -in @('.rs', '.ico', '.toml') } | ForEach-Object FullName)
        }
    }
    $taskSnapshot = @{}
    foreach ($taskSourceFile in $taskSourceFiles) { $taskSnapshot[$taskSourceFile] = (Get-FileHash -LiteralPath $taskSourceFile -Algorithm SHA256).Hash }
    return $taskSnapshot
}
Push-Location $taskRoot
try {
    $taskSourcesBefore = Get-ResourceSourceSnapshot
    if (!$SkipBuild) {
        & cargo test --release --all-targets --locked --offline --no-run --message-format=json > $taskArtifactsPath
        if ($LASTEXITCODE -ne 0) { throw 'Resource test build failed' }
    }
    $taskArtifacts = @(Get-Content -LiteralPath $taskArtifactsPath | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
        $_.reason -eq 'compiler-artifact' -and $_.executable -and $_.profile.test
    })
    if ($taskArtifacts.Count -lt 4) { throw 'Expected library, application, hardware-tool and resource test artifacts' }
    $taskNative = ($taskArtifacts | Where-Object { $_.target.name -eq 'lecoo-control-center' -and $_.target.kind -contains 'bin' }).executable
    $taskProbe = ($taskArtifacts | Where-Object { $_.target.name -eq 'resource_usage' }).executable
    $taskTests = @($taskArtifacts | ForEach-Object {
        @{Name=('Unit-' + $_.target.name); Executable=$_.executable; Arguments=@('--nocapture', '--test-threads=1')}
    })
    $taskTests += @(
        @{Name='NativeIcons'; Executable=$taskNative; Arguments=@('native_mode_icons_and_resource_lifetime', '--ignored', '--nocapture', '--test-threads=1')},
        @{Name='IdleWake'; Executable=$taskNative; Arguments=@('events_wake_idle_controller_without_waiting_for_timer', '--ignored', '--nocapture', '--test-threads=1')}
    )
    if ($Measure) {
        $taskTests += @{Name='MonitorVisible'; Executable=$taskProbe; Arguments=@('monitor_resource_usage', '--exact', '--ignored', '--nocapture', '--test-threads=1')}
        $taskTests += @{Name='MonitorHidden'; Executable=$taskProbe; Arguments=@('monitor_hidden_resource_usage', '--exact', '--ignored', '--nocapture', '--test-threads=1')}
    }
    $taskOriginalCompatibility = $env:__COMPAT_LAYER
    $taskResults = @()
    try {
        # Only read-only/native and simulated-hardware tests run. No app main,
        # real fan worker, single-instance lock or preference writer is invoked.
        $env:__COMPAT_LAYER = 'RunAsInvoker'
        foreach ($taskTest in $taskTests) {
            $taskOut = Join-Path $taskDirectory "$($taskTest.Name)-stdout.txt"
            $taskErr = Join-Path $taskDirectory "$($taskTest.Name)-stderr.txt"
            $taskProcess = Start-Process -FilePath $taskTest.Executable -ArgumentList $taskTest.Arguments -WindowStyle Hidden -RedirectStandardOutput $taskOut -RedirectStandardError $taskErr -Wait -PassThru
            $taskResults += [pscustomobject]@{
                Name=$taskTest.Name; ExitCode=$taskProcess.ExitCode
                SHA256=(Get-FileHash -LiteralPath $taskTest.Executable -Algorithm SHA256).Hash
                Output=[IO.File]::ReadAllText($taskOut)
                Error=[IO.File]::ReadAllText($taskErr)
            }
            Write-Output "$($taskTest.Name): exit $($taskProcess.ExitCode)"
        }
    } finally { $env:__COMPAT_LAYER = $taskOriginalCompatibility }
    $taskSourcesAfter = Get-ResourceSourceSnapshot
    $taskChangedSources = @(@($taskSourcesBefore.Keys) + @($taskSourcesAfter.Keys) | Sort-Object -Unique | Where-Object {
        $taskSourcesBefore[$_] -ne $taskSourcesAfter[$_]
    })
    $taskReport = [ordered]@{
        Passed=(@($taskResults | Where-Object ExitCode -ne 0).Count -eq 0 -and $taskChangedSources.Count -eq 0)
        HardwareWrites=$false; PreferencesChanged=$false; StartupTasksChanged=$false
        BuildSkipped=[bool]$SkipBuild; SourceUnchangedDuringChecks=($taskChangedSources.Count -eq 0)
        ChangedSources=$taskChangedSources; Sources=$taskSourcesBefore
        Tests=$taskResults
    }
    $taskReport | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
    Write-Output "Resource checks passed: $($taskReport.Passed). Report: $ReportPath"
    if (!$taskReport.Passed) { throw 'Resource checks failed; see captured output' }
} finally { Pop-Location }
