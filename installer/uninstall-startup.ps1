param([Parameter(Mandatory=$true)][string]$Executable)
$ErrorActionPreference = 'Stop'
$taskExecutable = [IO.Path]::GetFullPath($Executable)
foreach ($task in Get-ScheduledTask -TaskName 'LecooRustPowerControl-*' -ErrorAction SilentlyContinue) {
    $taskMatches = @($task.Actions | Where-Object {
        $_.Execute -and [string]::Equals($_.Execute.Trim('"'), $taskExecutable, [StringComparison]::OrdinalIgnoreCase)
    })
    if ($taskMatches.Count -gt 0) {
        Unregister-ScheduledTask -TaskName $task.TaskName -TaskPath $task.TaskPath -Confirm:$false
    }
}
