[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [string]$UnitTestExecutable,
    [string]$ReportPath
)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if (!$ReportPath) { $ReportPath = Join-Path $taskRoot ('logs\driver-research\sensors-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '.json') }
$ReportPath = [IO.Path]::GetFullPath($ReportPath)
New-Item -ItemType Directory -Path (Split-Path -Parent $ReportPath) -Force | Out-Null
if (!$SkipBuild) {
    Push-Location $taskRoot
    try {
        & cargo build --bin sensor_probe --locked --offline
        if ($LASTEXITCODE -ne 0) { throw 'Sensor probe build failed' }
        $taskArtifacts = @(& cargo test --lib --no-run --locked --offline --message-format=json)
        if ($LASTEXITCODE -ne 0) { throw 'Unit test build failed' }
        foreach ($line in $taskArtifacts) {
            $artifact = $line | ConvertFrom-Json
            if ($artifact.reason -eq 'compiler-artifact' -and $artifact.profile.test -and $artifact.executable) {
                $UnitTestExecutable = $artifact.executable
            }
        }
        if (!$UnitTestExecutable) { throw 'Unit test executable not found in Cargo artifacts' }
    } finally { Pop-Location }
}
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskElevated = ([Security.Principal.WindowsPrincipal]$taskIdentity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (!$taskElevated) {
    $taskArguments = @('-NoProfile','-ExecutionPolicy','Bypass','-File',('"' + $PSCommandPath + '"'),'-SkipBuild','-ReportPath',('"' + $ReportPath + '"'))
    if ($UnitTestExecutable) { $taskArguments += @('-UnitTestExecutable',('"' + $UnitTestExecutable + '"')) }
    $taskChild = Start-Process -FilePath "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -Verb RunAs -WindowStyle Hidden -ArgumentList $taskArguments -Wait -PassThru
    if (Test-Path -LiteralPath $ReportPath) { Get-Content -LiteralPath $ReportPath }
    exit $taskChild.ExitCode
}
function Invoke-Captured([string]$Executable) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Executable
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardOutputEncoding = [Text.Encoding]::UTF8
    $info.StandardErrorEncoding = [Text.Encoding]::UTF8
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    try {
        if (!$process.Start()) { throw "Cannot start $Executable" }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(30000)) { $process.Kill(); $process.WaitForExit(); throw 'Sensor test timed out' }
        return [ordered]@{ ExitCode=$process.ExitCode; Stdout=$stdout.Result; Stderr=$stderr.Result; SHA256=(Get-FileHash -LiteralPath $Executable -Algorithm SHA256).Hash }
    } finally { $process.Dispose() }
}
function Get-Mode {
    $instance = Get-CimInstance -Namespace root/WMI -ClassName PowerSwitchInterface | Where-Object { $_.InstanceName -eq 'ACPI\PNP0C14\IP3POWERSWITCH_0' }
    if (!$instance) { throw 'Compatible WMI instance not found' }
    return [int](Invoke-CimMethod -InputObject $instance -MethodName GetPowerMode).CurrentPowerMode
}
function Get-Driver {
    $driver = Get-CimInstance Win32_SystemDriver -Filter "Name='WinRing0_1_2_0'"
    if (!$driver) { throw 'Existing WinRing0 driver not found' }
    return [ordered]@{ Name=$driver.Name; State=$driver.State; StartMode=$driver.StartMode; PathName=$driver.PathName }
}
$taskReport = [ordered]@{ StartedAt=(Get-Date).ToString('o'); Passed=$false; Error=$null; UnitTests=$null; Probe=$null; ModeBefore=$null; ModeAfter=$null; DriverBefore=$null; DriverAfter=$null }
try {
    $taskReport.ModeBefore = Get-Mode
    $taskReport.DriverBefore = Get-Driver
    if ($UnitTestExecutable) {
        $taskReport.UnitTests = Invoke-Captured $UnitTestExecutable
        if ($taskReport.UnitTests.ExitCode -ne 0) { throw 'Sensor unit tests failed' }
    }
    $taskReport.Probe = Invoke-Captured (Join-Path $taskRoot 'target\debug\sensor_probe.exe')
    if ($taskReport.Probe.ExitCode -ne 0) { throw ('Native probe failed: ' + $taskReport.Probe.Stderr) }
    if ([regex]::Matches($taskReport.Probe.Stdout, '(?m)^SAMPLE=').Count -ne 5) { throw 'Expected five successful native samples' }
    $taskReport.Passed = $true
} catch { $taskReport.Error = $_.Exception.Message }
finally {
    try {
        $taskReport.ModeAfter = Get-Mode
        $taskReport.DriverAfter = Get-Driver
        if ($taskReport.ModeBefore -ne $taskReport.ModeAfter) { throw 'Power mode changed during sampling' }
        if (($taskReport.DriverBefore | ConvertTo-Json -Compress) -ne ($taskReport.DriverAfter | ConvertTo-Json -Compress)) { throw 'Driver state changed during sampling' }
    } catch { $taskReport.Passed = $false; $taskReport.Error = ($taskReport.Error + '; ' + $_.Exception.Message).Trim('; ') }
    $taskReport['FinishedAt'] = (Get-Date).ToString('o')
    $taskReport | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
}
Write-Output "Passed=$($taskReport.Passed), Report=$ReportPath"
if (!$taskReport.Passed) { exit 1 }
