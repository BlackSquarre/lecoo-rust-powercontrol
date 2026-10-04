//! Native Task Scheduler operations. No PowerShell process or persistent COM worker.
use super::ComGuard;
use anyhow::{bail, Context, Result};
use std::path::Path;
use windows::{
    core::{Interface, BSTR, HRESULT, PWSTR, VARIANT},
    Win32::{
        Foundation::{CloseHandle, LocalFree, ERROR_FILE_NOT_FOUND, HANDLE, HLOCAL, VARIANT_BOOL},
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, LookupAccountNameW,
            TokenUser, PSID, SID_NAME_USE, TOKEN_QUERY, TOKEN_USER,
        },
        System::{
            Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
            TaskScheduler::*,
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    },
};

const DESCRIPTION: &str = "Lecoo Rust PowerControl: current-user opt-in elevated startup";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStatus {
    Disabled,
    Enabled,
    Stale,
}

struct Token(HANDLE);
impl Drop for Token {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
unsafe fn sid_string(sid: PSID) -> Result<String> {
    let mut value = PWSTR::null();
    ConvertSidToStringSidW(sid, &mut value)?;
    let result = value.to_string();
    let _ = LocalFree(HLOCAL(value.0.cast()));
    Ok(result?)
}
fn current_sid() -> Result<String> {
    unsafe {
        let mut handle = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle)?;
        let token = Token(handle);
        let mut bytes = 0;
        let _ = GetTokenInformation(token.0, TokenUser, None, 0, &mut bytes);
        if bytes == 0 {
            bail!("无法读取当前用户 SID");
        }
        // TOKEN_USER contains a pointer; use aligned storage, not a byte buffer.
        let mut buffer = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            bytes,
            &mut bytes,
        )?;
        sid_string((*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid)
    }
}
fn account_sid(account: &str) -> Result<String> {
    if account.starts_with("S-1-") {
        return Ok(account.to_owned());
    }
    unsafe {
        let account = windows::core::HSTRING::from(account);
        let (mut bytes, mut domain_size) = (0, 0);
        let mut use_kind = SID_NAME_USE::default();
        let _ = LookupAccountNameW(
            None,
            &account,
            PSID::default(),
            &mut bytes,
            PWSTR::null(),
            &mut domain_size,
            &mut use_kind,
        );
        if bytes == 0 {
            bail!("无法解析启动任务用户 SID");
        }
        let mut sid = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
        let mut domain = vec![0u16; domain_size as usize];
        LookupAccountNameW(
            None,
            &account,
            PSID(sid.as_mut_ptr().cast()),
            &mut bytes,
            PWSTR(domain.as_mut_ptr()),
            &mut domain_size,
            &mut use_kind,
        )?;
        sid_string(PSID(sid.as_mut_ptr().cast()))
    }
}
fn text(getter: impl FnOnce(*mut BSTR) -> windows::core::Result<()>) -> Result<String> {
    let mut value = BSTR::new();
    getter(&mut value)?;
    Ok(value.to_string())
}
unsafe fn task(folder: &ITaskFolder, name: &BSTR) -> Result<Option<IRegisteredTask>> {
    match folder.GetTask(name) {
        Ok(task) => {
            let info = task.Definition()?.RegistrationInfo()?;
            if text(|value| info.Description(value))? != DESCRIPTION {
                bail!(
                    "The startup task name is occupied by an unrelated task; no changes were made."
                );
            }
            Ok(Some(task))
        }
        Err(error) if error.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) => Ok(None),
        Err(error) => Err(error).context("无法读取登录启动任务"),
    }
}
unsafe fn definition(
    service: &ITaskService,
    sid: &str,
    executable: &Path,
) -> Result<ITaskDefinition> {
    let definition = service.NewTask(0)?;
    definition
        .RegistrationInfo()?
        .SetDescription(&BSTR::from(DESCRIPTION))?;
    let principal = definition.Principal()?;
    principal.SetUserId(&BSTR::from(sid))?;
    principal.SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN)?;
    principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;
    let trigger: ILogonTrigger = definition.Triggers()?.Create(TASK_TRIGGER_LOGON)?.cast()?;
    trigger.SetUserId(&BSTR::from(sid))?;
    trigger.SetEnabled(VARIANT_BOOL(-1))?;
    let action: IExecAction = definition.Actions()?.Create(TASK_ACTION_EXEC)?.cast()?;
    action.SetPath(&BSTR::from(executable.to_string_lossy().as_ref()))?;
    action.SetArguments(&BSTR::from("--minimized"))?;
    action.SetWorkingDirectory(&BSTR::from(
        executable
            .parent()
            .context("Application directory missing")?
            .to_string_lossy()
            .as_ref(),
    ))?;
    let settings = definition.Settings()?;
    settings.SetEnabled(VARIANT_BOOL(-1))?;
    settings.SetDisallowStartIfOnBatteries(VARIANT_BOOL(0))?;
    settings.SetStopIfGoingOnBatteries(VARIANT_BOOL(0))?;
    settings.SetExecutionTimeLimit(&BSTR::from("PT0S"))?;
    settings.SetMultipleInstances(TASK_INSTANCES_IGNORE_NEW)?;
    settings.SetAllowDemandStart(VARIANT_BOOL(-1))?;
    Ok(definition)
}
unsafe fn valid_definition(
    definition: &ITaskDefinition,
    sid: &str,
    executable: &Path,
) -> Result<bool> {
    let principal = definition.Principal()?;
    let (mut logon, mut run_level) = (TASK_LOGON_TYPE::default(), TASK_RUNLEVEL_TYPE::default());
    principal.LogonType(&mut logon)?;
    principal.RunLevel(&mut run_level)?;
    if account_sid(&text(|value| principal.UserId(value))?)? != sid
        || logon != TASK_LOGON_INTERACTIVE_TOKEN
        || run_level != TASK_RUNLEVEL_HIGHEST
    {
        return Ok(false);
    }
    let actions = definition.Actions()?;
    let triggers = definition.Triggers()?;
    let (mut action_count, mut trigger_count) = (0, 0);
    actions.Count(&mut action_count)?;
    triggers.Count(&mut trigger_count)?;
    if action_count != 1 || trigger_count != 1 {
        return Ok(false);
    }
    let action = actions.get_Item(1)?;
    let trigger = triggers.get_Item(1)?;
    let (mut action_type, mut trigger_type) =
        (TASK_ACTION_TYPE::default(), TASK_TRIGGER_TYPE2::default());
    action.Type(&mut action_type)?;
    trigger.Type(&mut trigger_type)?;
    if action_type != TASK_ACTION_EXEC || trigger_type != TASK_TRIGGER_LOGON {
        return Ok(false);
    }
    let action: IExecAction = action.cast()?;
    let trigger: ILogonTrigger = trigger.cast()?;
    let mut trigger_enabled = VARIANT_BOOL::default();
    trigger.Enabled(&mut trigger_enabled)?;
    if trigger_enabled.0 == 0 || account_sid(&text(|value| trigger.UserId(value))?)? != sid {
        return Ok(false);
    }
    if !text(|value| action.Path(value))?.eq_ignore_ascii_case(&executable.to_string_lossy())
        || text(|value| action.Arguments(value))? != "--minimized"
        || !text(|value| action.WorkingDirectory(value))?.eq_ignore_ascii_case(
            &executable
                .parent()
                .context("Application directory missing")?
                .to_string_lossy(),
        )
    {
        return Ok(false);
    }
    let settings = definition.Settings()?;
    let (mut enabled, mut instances) = (VARIANT_BOOL::default(), TASK_INSTANCES_POLICY::default());
    settings.Enabled(&mut enabled)?;
    settings.MultipleInstances(&mut instances)?;
    Ok(enabled.0 != 0
        && instances == TASK_INSTANCES_IGNORE_NEW
        && text(|value| settings.ExecutionTimeLimit(value))? == "PT0S")
}

/// Task Scheduler remains the source of truth; initialize COM in the calling thread.
pub fn configure(operation: &str, executable: &Path) -> Result<StartupStatus> {
    if !["status", "enable", "disable"].contains(&operation) {
        bail!("Invalid startup operation");
    }
    let executable = std::path::absolute(executable)?;
    let _com = ComGuard::new()?;
    let sid = current_sid()?;
    let name = BSTR::from(format!("LecooRustPowerControl-{sid}"));
    unsafe {
        let service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        let empty = VARIANT::default();
        service.Connect(&empty, &empty, &empty, &empty)?;
        let folder = service.GetFolder(&BSTR::from("\\"))?;
        let mut registered = task(&folder, &name)?;
        match operation {
            "enable" => {
                if !executable.is_file() {
                    bail!("Application executable is missing.");
                }
                let definition = definition(&service, &sid, &executable)?;
                folder.RegisterTaskDefinition(
                    &name,
                    &definition,
                    TASK_CREATE_OR_UPDATE.0,
                    &VARIANT::from(sid.as_str()),
                    &empty,
                    TASK_LOGON_INTERACTIVE_TOKEN,
                    &empty,
                )?;
                registered = task(&folder, &name)?;
            }
            "disable" => {
                if registered.is_some() {
                    folder.DeleteTask(&name, 0)?;
                }
                if task(&folder, &name)?.is_some() {
                    bail!("Startup task still exists after deletion.");
                }
                registered = None;
            }
            _ => {}
        }
        let status = match registered {
            None => StartupStatus::Disabled,
            Some(task)
                if task.Enabled()?.0 != 0
                    && valid_definition(&task.Definition()?, &sid, &executable)? =>
            {
                StartupStatus::Enabled
            }
            Some(_) => StartupStatus::Stale,
        };
        if operation == "enable" && status != StartupStatus::Enabled {
            bail!("Startup registration verification failed.");
        }
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_definition_checks_identity_arguments_and_triggers_without_registration() {
        let _com = ComGuard::new().unwrap();
        let sid = current_sid().unwrap();
        let executable = std::env::current_exe().unwrap();
        unsafe {
            let service: ITaskService =
                CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER).unwrap();
            let definition = definition(&service, &sid, &executable).unwrap();
            assert!(valid_definition(&definition, &sid, &executable).unwrap());
            let action: IExecAction = definition
                .Actions()
                .unwrap()
                .get_Item(1)
                .unwrap()
                .cast()
                .unwrap();
            action.SetArguments(&BSTR::from("--unexpected")).unwrap();
            assert!(!valid_definition(&definition, &sid, &executable).unwrap());
            action.SetArguments(&BSTR::from("--minimized")).unwrap();
            definition
                .Triggers()
                .unwrap()
                .Create(TASK_TRIGGER_TIME)
                .unwrap();
            assert!(!valid_definition(&definition, &sid, &executable).unwrap());
        }
    }
    #[test]
    fn read_only_status_and_repeated_com_apartments() {
        let executable = std::env::current_exe().unwrap();
        for _ in 0..3 {
            configure("status", &executable).unwrap();
        }
        std::thread::spawn(move || configure("status", &executable).unwrap())
            .join()
            .unwrap();
    }
}
