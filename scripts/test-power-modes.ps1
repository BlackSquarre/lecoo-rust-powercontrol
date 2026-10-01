[CmdletBinding()]
param(
    [ValidateRange(1, 10)][int]$Cycles = 1,
    [switch]$ReadOnly,
    [switch]$SkipBuild,
    [switch]$FailAfterFirstSwitch,
    [string]$ReportPath
)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if (!$ReportPath) {
    $ReportPath = Join-Path $taskRoot ('logs\power-mode-tests\power-modes-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '.json')
}
$ReportPath = [IO.Path]::GetFullPath($ReportPath)
$taskReportDirectory = Split-Path -Parent $ReportPath
New-Item -ItemType Directory -Path $taskReportDirectory -Force | Out-Null

# Build before elevation; the test executable shares the GUI's production library.
if (!$SkipBuild) {
    & (Join-Path $PSScriptRoot 'build-release.ps1')
}

$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskElevated = ([Security.Principal.WindowsPrincipal]$taskIdentity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (!$taskElevated) {
    $taskChildArguments = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', ('"' + $PSCommandPath + '"'), '-SkipBuild', '-Cycles', $Cycles, '-ReportPath', ('"' + $ReportPath + '"'))
    if ($ReadOnly) { $taskChildArguments += '-ReadOnly' }
    if ($FailAfterFirstSwitch) { $taskChildArguments += '-FailAfterFirstSwitch' }
    $taskChild = Start-Process -FilePath "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -Verb RunAs -WindowStyle Hidden -ArgumentList $taskChildArguments -Wait -PassThru
    if (Test-Path -LiteralPath $ReportPath) { Get-Content -LiteralPath $ReportPath }
    exit $taskChild.ExitCode
}

$taskExecutable = Join-Path $taskRoot 'dist\hardware_test.exe'
$taskApp = Join-Path $taskRoot 'dist\lecoo-control-center.exe'
$taskReport = [ordered]@{
    StartedAt = (Get-Date).ToString('o')
    ReadOnly = [bool]$ReadOnly
    Cycles = $Cycles
    OriginalMode = $null
    FinalMode = $null
    Restored = $null
    Passed = $false
    Error = $null
    RestoreError = $null
    FallbackRestoreUsed = $false
    AppSHA256 = $null
    TestExecutableSHA256 = $null
    Calls = [Collections.Generic.List[object]]::new()
    Transitions = [Collections.Generic.List[object]]::new()
}
$taskInstance = $null
$taskWriteAttempted = $false

function Invoke-AppHardwareTest([string]$Arguments = '', [switch]$ExpectFailure) {
    $taskStartInfo = [Diagnostics.ProcessStartInfo]::new()
    $taskStartInfo.FileName = $taskExecutable
    $taskStartInfo.Arguments = $Arguments
    $taskStartInfo.UseShellExecute = $false
    $taskStartInfo.CreateNoWindow = $true
    $taskStartInfo.RedirectStandardOutput = $true
    $taskStartInfo.RedirectStandardError = $true
    $taskStartInfo.StandardOutputEncoding = [Text.Encoding]::UTF8
    $taskStartInfo.StandardErrorEncoding = [Text.Encoding]::UTF8
    $taskProcess = [Diagnostics.Process]::new()
    $taskProcess.StartInfo = $taskStartInfo
    try {
        if (!$taskProcess.Start()) { throw 'Could not start hardware_test.exe' }
        $taskOutputRead = $taskProcess.StandardOutput.ReadToEndAsync()
        $taskErrorRead = $taskProcess.StandardError.ReadToEndAsync()
        if (!$taskProcess.WaitForExit(30000)) {
            $taskProcess.Kill()
            $taskProcess.WaitForExit()
            throw 'hardware_test.exe exceeded 30 seconds'
        }
        $taskOutput = $taskOutputRead.Result
        $taskErrors = $taskErrorRead.Result
        $taskReport.Calls.Add([ordered]@{ Arguments=$Arguments; ExitCode=$taskProcess.ExitCode; Stdout=$taskOutput; Stderr=$taskErrors })
        if ($ExpectFailure) {
            if ($taskProcess.ExitCode -eq 0) { throw "Expected rejection of arguments: $Arguments" }
        } elseif ($taskProcess.ExitCode -ne 0) { throw "hardware_test.exe failed ($($taskProcess.ExitCode)): $taskErrors" }
        return $taskOutput
    } finally { $taskProcess.Dispose() }
}

function Get-IndependentMode {
    $taskResult = Invoke-CimMethod -InputObject $taskInstance -MethodName GetPowerMode
    if (!$taskResult.ReturnValue -or $null -eq $taskResult.CurrentPowerMode) { throw 'Independent CIM mode read failed' }
    $taskMode = [int]$taskResult.CurrentPowerMode
    if ($taskMode -notin @(0, 1, 2)) { throw "Unsupported current mode: $taskMode" }
    return $taskMode
}

try {
    $taskReport.AppSHA256 = (Get-FileHash -LiteralPath $taskApp -Algorithm SHA256).Hash
    $taskReport.TestExecutableSHA256 = (Get-FileHash -LiteralPath $taskExecutable -Algorithm SHA256).Hash
    $taskInstances = @(Get-CimInstance -Namespace root\wmi -ClassName PowerSwitchInterface | Where-Object { $_.Active -and $_.InstanceName -eq 'ACPI\PNP0C14\IP3POWERSWITCH_0' })
    if ($taskInstances.Count -ne 1) { throw "Expected exactly one active hardware instance; got $($taskInstances.Count)" }
    $taskInstance = $taskInstances[0]
    $taskReport.OriginalMode = Get-IndependentMode
    $taskOutput = Invoke-AppHardwareTest
    if ($taskOutput -notmatch "(?m)^MODE=$($taskReport.OriginalMode)\r?$" -or $taskOutput -notmatch '(?m)^MODE_COUNT=Some\(3\)\r?$') {
        throw "Production reads disagree with independent CIM or supported mode count: $taskOutput"
    }
    if (!$ReadOnly) {
        # Also restore if a future regression unexpectedly accepts the invalid input.
        $taskWriteAttempted = $true
        $null = Invoke-AppHardwareTest '--set-mode 3' -ExpectFailure
        if ((Get-IndependentMode) -ne $taskReport.OriginalMode) { throw 'Invalid mode input changed hardware state' }
        for ($taskCycle = 1; $taskCycle -le $Cycles; $taskCycle++) {
            foreach ($taskTarget in @(2, 0, 1)) {
                $taskWriteAttempted = $true
                $taskBefore = Get-IndependentMode
                $taskOutput = Invoke-AppHardwareTest "--set-mode $taskTarget"
                $taskAfter = Get-IndependentMode
                $taskReport.Transitions.Add([ordered]@{ Cycle=$taskCycle; Before=$taskBefore; Requested=$taskTarget; Actual=$taskAfter })
                if ($taskAfter -ne $taskTarget -or $taskOutput -notmatch "(?m)^SET_VERIFIED=$taskTarget\r?$" -or $taskOutput -notmatch "(?m)^MODE=$taskTarget\r?$") {
                    throw "Mode verification failed for target $taskTarget, actual $taskAfter"
                }
                if ($FailAfterFirstSwitch) { throw 'Injected failure after first switch to test restoration' }
            }
        }
    }
    $taskReport.Passed = $true
} catch {
    $taskReport.Error = $_.Exception.Message
} finally {
    if ($null -ne $taskReport.OriginalMode -and $taskWriteAttempted) {
        try {
            $null = Invoke-AppHardwareTest "--set-mode $($taskReport.OriginalMode)"
        } catch {
            $taskReport.Passed = $false
            $taskReport.RestoreError = $_.Exception.Message
            # A production regression must not prevent an independent restoration attempt.
            try {
                $taskReport.FallbackRestoreUsed = $true
                $taskRestore = Invoke-CimMethod -InputObject $taskInstance -MethodName SetPowerMode -Arguments @{PowerMode=[byte]$taskReport.OriginalMode}
                if (!$taskRestore.ReturnValue) { throw 'Fallback SetPowerMode returned false' }
            } catch { $taskReport.RestoreError += '; fallback: ' + $_.Exception.Message }
        }
    }
    if ($null -ne $taskReport.OriginalMode) {
        try {
            # Allow firmware to settle, then verify restoration independently.
            for ($taskAttempt = 0; $taskAttempt -lt 20; $taskAttempt++) {
                $taskReport.FinalMode = Get-IndependentMode
                if ($taskReport.FinalMode -eq $taskReport.OriginalMode) { break }
                Start-Sleep -Milliseconds 100
            }
            $taskReport.Restored = $taskReport.FinalMode -eq $taskReport.OriginalMode
            if (!$taskReport.Restored) { throw "Original mode was not restored: original=$($taskReport.OriginalMode), final=$($taskReport.FinalMode)" }
        } catch {
            $taskReport.Passed = $false
            $taskReport.Restored = $false
            $taskReport.RestoreError = $_.Exception.Message
        }
    }
    $taskReport['FinishedAt'] = (Get-Date).ToString('o')
    $taskReport | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
}
Write-Output "Passed=$($taskReport.Passed), Restored=$($taskReport.Restored), Report=$ReportPath"
if (!$taskReport.Passed) { exit 1 }
exit 0
