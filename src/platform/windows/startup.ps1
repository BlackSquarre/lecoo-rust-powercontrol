$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
try {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $sid = $identity.User.Value
    $taskName = 'LecooRustPowerControl-' + $sid
    $description = 'Lecoo Rust PowerControl: current-user opt-in elevated startup'
    $scheduler = New-Object -ComObject Schedule.Service
    $scheduler.Connect()
    $folder = $scheduler.GetFolder('\')
    $task = $null
    try { $task = $folder.GetTask($taskName) } catch {
        if ($_.Exception.HResult -ne -2147024894) { throw }
    }
    if ($task -and $task.Definition.RegistrationInfo.Description -ne $description) {
        throw 'The startup task name is occupied by an unrelated task; no changes were made.'
    }
    switch ($env:LECOO_STARTUP_OPERATION) {
        'enable' {
            $exe = [IO.Path]::GetFullPath($env:LECOO_STARTUP_EXE)
            if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Application executable is missing.' }
            $definition = $scheduler.NewTask(0)
            $definition.RegistrationInfo.Description = $description
            $definition.Principal.UserId = $sid
            $definition.Principal.LogonType = 3 # InteractiveToken: no saved password
            $definition.Principal.RunLevel = 1 # HighestAvailable
            $trigger = $definition.Triggers.Create(9) # Logon
            $trigger.UserId = $sid
            $trigger.Enabled = $true
            $action = $definition.Actions.Create(0) # Execute
            $action.Path = $exe
            $action.Arguments = '--minimized'
            $action.WorkingDirectory = Split-Path -Parent $exe
            $definition.Settings.Enabled = $true
            $definition.Settings.DisallowStartIfOnBatteries = $false
            $definition.Settings.StopIfGoingOnBatteries = $false
            $definition.Settings.ExecutionTimeLimit = 'PT0S'
            $definition.Settings.MultipleInstances = 2 # IgnoreNew
            $definition.Settings.AllowDemandStart = $true
            $null = $folder.RegisterTaskDefinition($taskName, $definition, 6, $sid, $null, 3)
            $task = $folder.GetTask($taskName)
        }
        'disable' {
            if ($task) { $folder.DeleteTask($taskName, 0) }
            try { $null = $folder.GetTask($taskName); throw 'Startup task still exists after deletion.' } catch {
                if ($_.Exception.HResult -ne -2147024894) { throw }
            }
            $task = $null
        }
        'status' { }
        default { throw 'Unknown startup operation.' }
    }
    if (!$task) { Write-Output 'DISABLED'; exit 0 }
    $definition = $task.Definition
    $principal = $definition.Principal
    function Resolve-Sid([string]$account) {
        if ($account -like 'S-1-*') { return $account }
        return ([Security.Principal.NTAccount]::new($account)).Translate([Security.Principal.SecurityIdentifier]).Value
    }
    $actions = $definition.Actions
    $triggers = $definition.Triggers
    $action = if ($actions.Count -eq 1) { $actions.Item(1) } else { $null }
    $trigger = if ($triggers.Count -eq 1) { $triggers.Item(1) } else { $null }
    $valid = $task.Enabled -and $definition.Settings.Enabled -and
        (Resolve-Sid $principal.UserId) -eq $sid -and $principal.LogonType -eq 3 -and $principal.RunLevel -eq 1 -and
        $actions.Count -eq 1 -and $action.Type -eq 0 -and
        $action.Path -eq $env:LECOO_STARTUP_EXE -and $action.Arguments -eq '--minimized' -and
        $action.WorkingDirectory -eq (Split-Path -Parent $env:LECOO_STARTUP_EXE) -and
        $triggers.Count -eq 1 -and $trigger.Type -eq 9 -and $trigger.Enabled -and (Resolve-Sid $trigger.UserId) -eq $sid -and
        $definition.Settings.ExecutionTimeLimit -eq 'PT0S' -and $definition.Settings.MultipleInstances -eq 2
    if ($valid) { Write-Output 'ENABLED' } else { Write-Output 'STALE' }
    if ($env:LECOO_STARTUP_OPERATION -eq 'enable' -and !$valid) { throw 'Startup registration verification failed.' }
} catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }
