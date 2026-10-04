[CmdletBinding()]
param([string]$ReportPath)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if (!$ReportPath) { $ReportPath = Join-Path $taskRoot 'logs\mode-icons\native-icons.json' }
$taskReportDirectory = Split-Path -Parent $ReportPath
New-Item -ItemType Directory -Path $taskReportDirectory -Force | Out-Null

# These native tests simulate state and never open the hardware interface, alter
# preferences, or acquire the running app's single-instance lock.
Push-Location $taskRoot
try {
    $taskArtifacts = @{}
    foreach ($taskLine in @(& cargo test --release --lib --bin lecoo-control-center --locked --offline --no-run --target-dir target/icon-update --message-format=json)) {
        if ($taskLine -notmatch '^\{') { continue }
        $taskArtifact = $taskLine | ConvertFrom-Json
        if ($taskArtifact.reason -eq 'compiler-artifact' -and $taskArtifact.executable -and $taskArtifact.profile.test) {
            if ($taskArtifact.target.kind -contains 'lib') { $taskArtifacts.Library = $taskArtifact.executable }
            if ($taskArtifact.target.kind -contains 'bin' -and $taskArtifact.target.name -eq 'lecoo-control-center') { $taskArtifacts.Native = $taskArtifact.executable }
        }
    }
    if ($LASTEXITCODE -ne 0 -or !$taskArtifacts.Library -or !$taskArtifacts.Native) { throw 'Native icon test build failed' }

    $taskOriginalCompatibility = $env:__COMPAT_LAYER
    $taskResults = @()
    try {
        # Only test processes inherit this flag. Their embedded app manifest asks
        # for elevation, but none of these tests requires administrator access.
        $env:__COMPAT_LAYER = 'RunAsInvoker'
        foreach ($taskTest in @(
            @{Name='Library';Executable=$taskArtifacts.Library;Arguments=@('--nocapture')},
            @{Name='NativeUnit';Executable=$taskArtifacts.Native;Arguments=@('--nocapture','--test-threads=1')},
            @{Name='Icons';Executable=$taskArtifacts.Native;Arguments=@('native_mode_icons_and_resource_lifetime','--ignored','--nocapture','--test-threads=1')}
        )) {
            $taskOut = Join-Path $taskReportDirectory "$($taskTest.Name)-stdout.txt"
            $taskErr = Join-Path $taskReportDirectory "$($taskTest.Name)-stderr.txt"
            # Waiting is essential for a Windows-subsystem test executable.
            $taskProcess = Start-Process -FilePath $taskTest.Executable -ArgumentList $taskTest.Arguments -WindowStyle Hidden -RedirectStandardOutput $taskOut -RedirectStandardError $taskErr -Wait -PassThru
            $taskOutput = Get-Content -LiteralPath $taskOut -Raw
            $taskResults += [pscustomobject]@{
                Name=$taskTest.Name; ExitCode=$taskProcess.ExitCode
                SHA256=(Get-FileHash -LiteralPath $taskTest.Executable -Algorithm SHA256).Hash
                Output=$taskOutput; Error=(Get-Content -LiteralPath $taskErr -Raw)
            }
        }
    } finally { $env:__COMPAT_LAYER = $taskOriginalCompatibility }

    $taskIconOutput = ($taskResults | Where-Object Name -eq Icons).Output
    $taskCycles = @([regex]::Matches($taskIconOutput, 'Cycle (\d+): GDI=(\d+) USER=(\d+)') | ForEach-Object {
        [pscustomobject]@{Cycle=[int]$_.Groups[1].Value;Gdi=[int]$_.Groups[2].Value;User=[int]$_.Groups[3].Value}
    })
    $taskReport = [ordered]@{
        Passed=(@($taskResults | Where-Object ExitCode -ne 0).Count -eq 0 -and $taskCycles.Count -eq 12)
        HardwareWrites=$false; PreferencesChanged=$false; ResourceCycles=$taskCycles; Tests=$taskResults
    }
    $taskReport | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
    Write-Output "Native icon checks passed: $($taskReport.Passed). Report: $ReportPath"
    if (!$taskReport.Passed) { throw 'Native icon checks failed; see report and captured output' }
} finally { Pop-Location }
