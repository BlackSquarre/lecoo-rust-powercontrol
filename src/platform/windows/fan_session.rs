use super::{wake::UiWake, ComGuard, WindowsHardwareControl};
use crate::{
    core::cooling::{CoolingController, FanTarget},
    platform::HardwareControl,
};
use anyhow::{bail, Context, Result};
use std::{
    io::{BufRead, BufReader, Write},
    os::windows::process::CommandExt,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
use windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0},
        System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
    },
};

struct FanLock(HANDLE);
impl FanLock {
    fn acquire() -> Result<Self> {
        let handle =
            unsafe { CreateMutexW(None, false, w!("Global\\LecooRustPowerControl.FanSession")) }?;
        let result = unsafe { WaitForSingleObject(handle, 0) };
        if result != WAIT_OBJECT_0 && result != WAIT_ABANDONED {
            unsafe {
                let _ = CloseHandle(handle);
            }
            bail!("另一个风扇控制会话正在运行");
        }
        Ok(Self(handle))
    }
}
impl Drop for FanLock {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

fn emit(line: impl AsRef<str>) {
    // A closed parent pipe must not panic before automatic recovery.
    let mut output = std::io::stdout().lock();
    let _ = writeln!(output, "{}", line.as_ref().replace(['\r', '\n'], " "));
    let _ = output.flush();
}

pub fn worker() -> Result<()> {
    let _lock = FanLock::acquire()?;
    let _com = ComGuard::new()?;
    let hw = WindowsHardwareControl::new()?;
    if !hw.is_elevated() {
        bail!("风扇守护进程需要管理员权限");
    }
    let mut control = CoolingController::new(hw);
    // Opening a session establishes firmware automatic control; no manual preference is persisted.
    let receipt = control.restore_auto()?;
    emit(format!("AUTO {} {}", receipt.status, receipt.rpm));
    // restore_auto already validates a nonzero RPM.
    emit("READY");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            match line {
                Ok(line) => {
                    if sender.send(line).is_err() {
                        return;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = sender.send("QUIT".to_owned());
    });
    let mut heartbeat = Instant::now();
    let mut sample_at = Instant::now() - Duration::from_secs(1);
    loop {
        // Sleep until the next protection deadline or an incoming command.
        let remaining = Duration::from_millis(500).saturating_sub(sample_at.elapsed());
        match receiver.recv_timeout(remaining) {
            Ok(line) if line == "HB" => {
                heartbeat = Instant::now();
            }
            Ok(line) if line == "QUIT" => {
                let receipt = control.restore_auto()?;
                emit(format!("AUTO {} {}", receipt.status, receipt.rpm));
                emit("STOPPED");
                return Ok(());
            }
            Ok(line) => {
                let requested = if line == "AUTO" {
                    Ok(FanTarget::Auto)
                } else if let Some(value) = line.strip_prefix("P ") {
                    value
                        .parse::<u8>()
                        .context("非法风扇目标")
                        .and_then(FanTarget::percent)
                } else {
                    Err(anyhow::anyhow!("未知风扇命令"))
                };
                match requested.and_then(|target| control.apply(target)) {
                    Ok(receipt) => {
                        heartbeat = Instant::now();
                        emit(format!(
                            "OK {} {} {}",
                            receipt.target.encoded(),
                            receipt.status,
                            receipt.rpm
                        ));
                    }
                    Err(error) => {
                        emit(format!("ERROR {error:#}"));
                        if !control.is_armed() && control.target() == FanTarget::Auto {
                            emit(format!("RECOVERY 101 {error:#}"));
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                control.restore_auto()?;
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if sample_at.elapsed() >= Duration::from_millis(500) {
            sample_at = Instant::now();
            if let Err(error) = control.guard(heartbeat.elapsed()) {
                let target = if control.is_armed() && control.target() == FanTarget::Auto {
                    255
                } else {
                    control.target().encoded()
                };
                emit(format!("RECOVERY {target} {error:#}"));
            }
            // The GUI reads its own visible telemetry. guard consumes exactly one
            // protection sample; do not repeat RPM/temperature queries for unused DATA.
        }
    }
}

pub struct FanClient {
    child: Child,
    input: Option<ChildStdin>,
    messages: Receiver<String>,
    heartbeat: Instant,
    stopping_at: Option<Instant>,
    stop_warned: bool,
}
impl FanClient {
    pub fn start(wake: UiWake) -> Result<Self> {
        let child = Command::new(std::env::current_exe()?)
            .arg("--fan-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(0x08000000)
            .spawn()
            .context("无法启动风扇守护进程")?;
        Self::from_child(child, wake)
    }
    fn from_child(mut child: Child, wake: UiWake) -> Result<Self> {
        let input = child.stdin.take();
        let output = child.stdout.take().context("风扇守护输出不可用")?;
        let (sender, messages) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let Ok(line) = line else {
                    break;
                };
                if sender.send(line).is_err() {
                    return;
                }
                wake.notify();
            }
            let _ = sender.send("DIED 风扇守护进程已退出".to_owned());
            wake.notify();
        });
        Ok(Self {
            child,
            input,
            messages,
            heartbeat: Instant::now() - Duration::from_secs(1),
            stopping_at: None,
            stop_warned: false,
        })
    }
    pub fn send(&mut self, target: FanTarget) -> Result<()> {
        target.validate()?;
        let command = match target {
            FanTarget::Auto => "AUTO".to_owned(),
            FanTarget::Percent(value) => format!("P {value}"),
        };
        self.write(&command)
    }
    fn write(&mut self, command: &str) -> Result<()> {
        let input = self.input.as_mut().context("风扇会话已关闭")?;
        writeln!(input, "{command}")?;
        input.flush()?;
        Ok(())
    }
    pub fn tick(&mut self) -> Result<()> {
        if self.stopping_at.is_none() && self.heartbeat.elapsed() >= Duration::from_millis(500) {
            self.write("HB")?;
            self.heartbeat = Instant::now();
        }
        Ok(())
    }
    pub fn next_message(&self) -> Option<String> {
        self.messages.try_recv().ok()
    }
    pub fn begin_stop(&mut self) -> Result<()> {
        if self.stopping_at.is_some() {
            return Ok(());
        }
        self.stopping_at = Some(Instant::now());
        // Close stdin too: even if QUIT fails the worker sees EOF and recovers.
        let result = self.write("QUIT");
        self.input.take();
        result
    }
    pub fn poll_stop(&mut self) -> Result<bool> {
        if let Some(status) = self.child.try_wait()? {
            if !status.success() {
                bail!("风扇守护退出失败；自动恢复可能未完成");
            }
            return Ok(true);
        }
        Ok(false)
    }
    pub fn take_stop_timeout(&mut self) -> bool {
        if !self.stop_warned
            && self
                .stopping_at
                .is_some_and(|at| at.elapsed() >= Duration::from_secs(5))
        {
            self.stop_warned = true;
            return true;
        }
        false
    }
    pub fn stop(&mut self) -> Result<()> {
        let _ = self.begin_stop();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if self.poll_stop()? {
                return Ok(());
            }
            if Instant::now() >= deadline {
                bail!("风扇恢复仍在后台进行，请检查散热状态");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
impl Drop for FanClient {
    fn drop(&mut self) {
        self.input.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::HWND;

    #[test]
    fn asynchronous_shutdown_reaps_success_and_detects_failed_exit() {
        for exit in [0, 7] {
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "platform::windows::fan_session::tests::mock_worker_process",
                    "--nocapture",
                ])
                .env("LECOO_TEST_WORKER_EXIT", exit.to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .creation_flags(0x08000000)
                .spawn()
                .unwrap();
            let mut client = FanClient::from_child(child, UiWake::new(HWND::default())).unwrap();
            client.begin_stop().unwrap();
            assert!(
                !client.poll_stop().unwrap(),
                "stop must return before the delayed worker exits"
            );
            client.tick().unwrap(); // No heartbeat write to the intentionally closed pipe.
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                match client.poll_stop() {
                    Ok(true) => {
                        assert_eq!(exit, 0);
                        break;
                    }
                    Err(_) => {
                        assert_ne!(exit, 0);
                        break;
                    }
                    Ok(false) => assert!(Instant::now() < deadline, "mock worker did not exit"),
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
    #[test]
    #[ignore = "child-process entry point for the asynchronous shutdown test"]
    fn mock_worker_process() {
        let Ok(exit) = std::env::var("LECOO_TEST_WORKER_EXIT") else {
            return;
        };
        let mut command = String::new();
        std::io::stdin().read_line(&mut command).unwrap();
        assert_eq!(command.trim(), "QUIT");
        std::thread::sleep(Duration::from_millis(1000));
        let exit: i32 = exit.parse().unwrap();
        if exit != 0 {
            std::process::exit(exit);
        }
    }
}
