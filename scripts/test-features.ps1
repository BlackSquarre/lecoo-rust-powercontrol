[CmdletBinding()]
param([switch]$SkipBuild, [string]$Executable, [string]$UnitTestExecutable, [string]$ReportPath)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
if (!$Executable) { $Executable = Join-Path $taskRoot 'target\debug\lecoo-control-center.exe' }
if (!$ReportPath) { $ReportPath = Join-Path $taskRoot ('logs\feature-tests\features-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '.json') }
$ReportPath = [IO.Path]::GetFullPath($ReportPath)
New-Item -ItemType Directory -Path (Split-Path -Parent $ReportPath) -Force | Out-Null
if (!$SkipBuild) {
    Push-Location $taskRoot
    try {
        & cargo build --bin lecoo-control-center --locked --offline
        if ($LASTEXITCODE -ne 0) { throw 'Application build failed' }
        foreach ($line in @(& cargo test --lib --no-run --locked --offline --message-format=json)) {
            $artifact = $line | ConvertFrom-Json
            if ($artifact.reason -eq 'compiler-artifact' -and $artifact.profile.test -and $artifact.executable) { $UnitTestExecutable = $artifact.executable }
        }
        if ($LASTEXITCODE -ne 0 -or !$UnitTestExecutable) { throw 'Unit test build failed' }
    } finally { Pop-Location }
}
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskElevated = ([Security.Principal.WindowsPrincipal]$taskIdentity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (!$taskElevated) {
    $taskArguments = @('-NoProfile','-ExecutionPolicy','Bypass','-File',('"'+$PSCommandPath+'"'),'-SkipBuild','-Executable',('"'+$Executable+'"'),'-ReportPath',('"'+$ReportPath+'"'))
    if ($UnitTestExecutable) { $taskArguments += @('-UnitTestExecutable',('"'+$UnitTestExecutable+'"')) }
    $taskChild = Start-Process "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -Verb RunAs -WindowStyle Hidden -ArgumentList $taskArguments -Wait -PassThru
    if (Test-Path -LiteralPath $ReportPath) { Get-Content -LiteralPath $ReportPath }
    exit $taskChild.ExitCode
}
function New-Captured([string]$File, [string]$Arguments) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName=$File; $info.Arguments=$Arguments; $info.UseShellExecute=$false; $info.CreateNoWindow=$true
    $info.RedirectStandardInput=$true; $info.RedirectStandardOutput=$true; $info.RedirectStandardError=$true
    $info.StandardOutputEncoding=[Text.Encoding]::UTF8; $info.StandardErrorEncoding=[Text.Encoding]::UTF8
    $process=[Diagnostics.Process]::new(); $process.StartInfo=$info
    if (!$process.Start()) { throw 'Process start failed' }
    return $process
}
function Invoke-Captured([string]$File, [string]$Arguments) {
    $process = New-Captured $File $Arguments
    try {
        $output=$process.StandardOutput.ReadToEndAsync(); $errors=$process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(30000)) { throw 'Command timed out' }
        if ($process.ExitCode -ne 0) { throw ('Command failed: '+$errors.Result+' '+$output.Result) }
        return $output.Result.Trim()
    } finally { $process.Dispose() }
}
function Startup([string]$Operation) { return Invoke-Captured $Executable ('--startup '+$Operation) }
function Read-Fan { return [int](Invoke-CimMethod -InputObject $script:taskInstance -MethodName GetFanControl -Arguments @{FanNumber=[byte]1}).FanDuty -band 0xffff }
function Read-Mode { return [int](Invoke-CimMethod -InputObject $script:taskInstance -MethodName GetPowerMode).CurrentPowerMode }
function Drain-Worker([double]$Seconds, [bool]$Heartbeat) {
    $watch=[Diagnostics.Stopwatch]::StartNew()
    do {
        if ($Heartbeat) { $script:taskWorker.StandardInput.WriteLine('HB'); $script:taskWorker.StandardInput.Flush() }
        while ($script:taskLine.IsCompleted) {
            $line=$script:taskLine.Result
            if ($null -eq $line) { return }
            $script:taskLines.Add($line)
            $script:taskLine=$script:taskWorker.StandardOutput.ReadLineAsync()
        }
        Start-Sleep -Milliseconds 100
    } while ($watch.Elapsed.TotalSeconds -lt $Seconds)
}
function Send-Worker([string]$Command) { $script:taskWorker.StandardInput.WriteLine($Command); $script:taskWorker.StandardInput.Flush() }
$taskReport=[ordered]@{StartedAt=(Get-Date).ToString('o');Passed=$false;Error=$null;Executable=$Executable;SHA256=(Get-FileHash -LiteralPath $Executable).Hash;UnitTests=$null;Startup=@();Gui=@();Fan=@();Restored=$false}
$taskWorker=$null; $taskGui=$null; $taskOriginalXml=$null; $taskScheduler=$null; $taskTouched=$false
$taskCoolingTouched=$false
$taskLines=[Collections.Generic.List[string]]::new()
try {
    if (Get-Process 'lecoo-control-center' -ErrorAction SilentlyContinue) { throw 'Close existing Rust application before this test; the official Control Center may remain open.' }
    $taskInstance=Get-CimInstance -Namespace root/WMI -ClassName PowerSwitchInterface | Where-Object InstanceName -eq 'ACPI\PNP0C14\IP3POWERSWITCH_0'
    if (!$taskInstance) { throw 'Compatible WMI provider unavailable' }
    $taskReport['ModeBefore']=Read-Mode
    if ($UnitTestExecutable) { $taskReport.UnitTests=Invoke-Captured $UnitTestExecutable '' }
    $taskScheduler=New-Object -ComObject Schedule.Service; $taskScheduler.Connect(); $taskFolder=$taskScheduler.GetFolder('\')
    $taskName='LecooRustPowerControl-'+$taskIdentity.User.Value
    try {
        $taskOriginal=$taskFolder.GetTask($taskName)
        if ($taskOriginal.Definition.RegistrationInfo.Description -ne 'Lecoo Rust PowerControl: current-user opt-in elevated startup') { throw 'Unrelated task occupies application task name; no test changes made.' }
        $taskOriginalXml=$taskOriginal.Xml
    } catch { if ($_.Exception.HResult -ne -2147024894) { throw } }
    $taskTouched=$true
    foreach ($operation in @('disable','status','enable','enable','status')) {
        $status=Startup $operation; $taskReport.Startup+=@{Operation=$operation;Status=$status}
        $expected=if ($operation -in @('disable','status') -and $taskReport.Startup.Count -le 2) { 'Disabled' } else { 'Enabled' }
        if ($status -ne $expected) { throw "Unexpected startup state: $status" }
    }
    $taskDefinition=$taskFolder.GetTask($taskName).Definition
    $taskReport['TaskDefinition']=[ordered]@{RunLevel=$taskDefinition.Principal.RunLevel;LogonType=$taskDefinition.Principal.LogonType;User=$taskDefinition.Principal.UserId;TriggerCount=$taskDefinition.Triggers.Count;ActionCount=$taskDefinition.Actions.Count;Arguments=$taskDefinition.Actions.Item(1).Arguments}
    if (@($taskFolder.GetTasks(0) | Where-Object Name -eq $taskName).Count -ne 1) { throw 'Duplicate startup entries' }
    # Exercise the registered action without logging off the user's session.
    $taskRunning=$taskFolder.GetTask($taskName).Run($null)
    Start-Sleep -Seconds 3
    $taskStarted=@(Get-Process 'lecoo-control-center' -ErrorAction SilentlyContinue)
    $taskReport['TaskLaunchPids']=@($taskStarted | ForEach-Object Id)
    if ($taskStarted.Count -ne 1) { throw 'Registered startup action did not launch one application' }
    $taskFolder.GetTask($taskName).Stop(0)
    Start-Sleep -Seconds 1
    foreach ($operation in @('disable','disable','status')) {
        $status=Startup $operation; $taskReport.Startup+=@{Operation=$operation;Status=$status}
        if ($status -ne 'Disabled') { throw 'Startup disable failed' }
    }
    $taskGuiReport=$ReportPath+'.gui.json'
    $taskCoolingTouched=$true
    $taskGui=New-Captured $Executable ('--smoke-test "'+$taskGuiReport+'"')
    $taskGuiErrors=$taskGui.StandardError.ReadToEndAsync(); $taskGuiOutput=$taskGui.StandardOutput.ReadToEndAsync()
    Start-Sleep -Seconds 2
    $taskReport.Gui+=Get-Content -LiteralPath $taskGuiReport -Raw | ConvertFrom-Json
    $null=Invoke-Captured $Executable '' # Must signal existing window and exit.
    Start-Sleep -Seconds 1
    $taskReport.Gui+=Get-Content -LiteralPath $taskGuiReport -Raw | ConvertFrom-Json
    while (!$taskGui.HasExited) {
        Start-Sleep -Milliseconds 500
        if (Test-Path -LiteralPath $taskGuiReport) { $taskReport.Gui+=Get-Content -LiteralPath $taskGuiReport -Raw | ConvertFrom-Json }
        if ($taskReport.Gui.Count -gt 40) { throw 'GUI smoke test timed out' }
    }
    if ($taskGui.ExitCode -ne 0) { throw ('GUI failed: '+$taskGuiErrors.Result) }
    if (!$taskReport.Gui[0].Hidden -or !$taskReport.Gui[0].TrayRegistered) { throw 'Tray hiding failed' }
    if ($taskReport.Gui[0].Backend -eq 'Win32' -and ($taskReport.Gui[0].ControlCount -ne 0 -or $taskReport.Gui[0].WindowsDestroyed -lt 1)) { throw 'Native window controls were not released on hide' }
    if (!($taskReport.Gui | Where-Object { $_.OpenRequests -ge 1 -and !$_.Hidden })) { throw 'Second launch failed to reopen existing window' }
    if (!($taskReport.Gui | Where-Object { $_.FanReady -and $_.FanTarget -eq 50 })) { throw 'GUI manual fan path was not exercised' }
    if (!($taskReport.Gui | Where-Object { $_.Hidden -and !$_.WindowVisible -and $_.FanTarget -eq 50 -and $_.Seconds -ge 12 })) { throw 'Hidden window did not keep manual-session heartbeats alive' }
    $taskReport['RpmAfterGuiExit']=Read-Fan
    $taskWorker=New-Captured $Executable '--fan-worker'
    $taskLine=$taskWorker.StandardOutput.ReadLineAsync()
    Drain-Worker 2 $true
    if (!($taskLines | Where-Object { $_ -eq 'READY' })) { throw ('Worker not ready: '+($taskLines -join ';')) }
    foreach ($percent in @(35,50,100)) {
        Send-Worker "P $percent"; Drain-Worker 6 $true
        $rpm=Read-Fan; $taskReport.Fan+=@{Target=$percent;Rpm=$rpm}
        if ($rpm -le 0) { throw 'Invalid measured RPM' }
    }
    if ($taskReport.Fan[2].Rpm -le $taskReport.Fan[0].Rpm) { throw 'Maximum fan request did not increase RPM' }
    Send-Worker 'P 34'; Drain-Worker 1 $true
    if (!($taskLines | Where-Object { $_ -like 'ERROR *35*100*' })) { throw 'Invalid fan value was not rejected' }
    Send-Worker 'P 50'; Drain-Worker 1 $true
    Drain-Worker 4.5 $false
    if (!($taskLines | Where-Object { $_ -like 'RECOVERY 101 *' })) { throw 'Heartbeat timeout did not recover automatic control' }
    $taskReport.Fan+=@{Target='AutoAfterHeartbeatTimeout';Rpm=(Read-Fan)}
    Send-Worker 'P 50'; Drain-Worker 1 $true
    $taskWorker.StandardInput.Close(); Drain-Worker 1 $false
    if (!$taskWorker.WaitForExit(10000) -or $taskWorker.ExitCode -ne 0) { throw 'EOF recovery worker failed' }
    if (!($taskLines | Where-Object { $_ -eq 'STOPPED' })) { throw 'EOF automatic restoration was not acknowledged' }
    $taskReport['WorkerOutput']=@($taskLines)
    $taskReport.Passed=$true
} catch { $taskReport.Error=$_.Exception.Message }
finally {
    if ($taskGui -and !$taskGui.HasExited) { $taskGui.Kill(); $taskGui.WaitForExit() }
    if ($taskWorker -and !$taskWorker.HasExited) { $taskWorker.StandardInput.Close(); $null=$taskWorker.WaitForExit(10000) }
    try {
        if ($taskInstance -and $taskCoolingTouched) {
            $recovery=Invoke-CimMethod -InputObject $taskInstance -MethodName SetFanControl -Arguments @{FanNumber=[byte]1;FanDuty=[byte]101}
            $taskReport['FinalAutoStatus']=$recovery.ResultStatus; $taskReport['FinalRpm']=Read-Fan; $taskReport['ModeAfter']=Read-Mode
            if ($recovery.ResultStatus -eq 255 -or $taskReport.FinalRpm -le 0 -or $taskReport.ModeAfter -ne $taskReport.ModeBefore) { throw 'Final recovery or unchanged power mode verification failed' }
            $taskReport.Restored=$true
        }
        if ($taskTouched) {
            if ($taskOriginalXml) { $null=$taskFolder.RegisterTask($taskName,$taskOriginalXml,6,$taskIdentity.User.Value,$null,3) }
            else { $null=Startup 'disable' }
        }
    } catch { $taskReport.Passed=$false; $taskReport.Error=($taskReport.Error+'; '+$_.Exception.Message).Trim('; ') }
    $taskReport['FinishedAt']=(Get-Date).ToString('o')
    $taskReport | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
}
Write-Output "Passed=$($taskReport.Passed), Report=$ReportPath"
if (!$taskReport.Passed) { exit 1 }
