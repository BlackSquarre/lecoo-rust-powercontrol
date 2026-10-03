use super::{ComGuard, WindowsHardwareControl};
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
    match control.measured_rpm() {
        Ok(_) => emit("READY"),
        Err(error) => emit(format!("UNAVAILABLE {error:#}")),
    }
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
        match receiver.recv_timeout(Duration::from_millis(100)) {
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
                    Err(error) => emit(format!("ERROR {error:#}")),
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
                emit(format!("RECOVERY {} {error:#}", control.target().encoded()));
            }
            match control.measured_rpm() {
                Ok(rpm) => {
                    let temp = control
                        .sample()
                        .ok()
                        .map(|(temp, _)| format!("{temp:.3}"))
                        .unwrap_or_else(|| "NA".to_owned());
                    emit(format!("DATA {temp} {rpm} {}", control.target().encoded()));
                }
                Err(error) => emit(format!("UNAVAILABLE {error:#}")),
            }
        }
    }
}

pub struct FanClient {
    child: Child,
    input: Option<ChildStdin>,
    messages: Receiver<String>,
    heartbeat: Instant,
}
impl FanClient {
    pub fn start() -> Result<Self> {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("--fan-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(0x08000000)
            .spawn()
            .context("无法启动风扇守护进程")?;
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
            }
            let _ = sender.send("DIED 风扇守护进程已退出".to_owned());
        });
        Ok(Self {
            child,
            input,
            messages,
            heartbeat: Instant::now() - Duration::from_secs(1),
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
    pub fn tick(&mut self) -> Result<Vec<String>> {
        if self.heartbeat.elapsed() >= Duration::from_millis(500) {
            self.write("HB")?;
            self.heartbeat = Instant::now();
        }
        Ok(self.messages.try_iter().collect())
    }
    pub fn stop(&mut self) -> Result<()> {
        // Close stdin too: even if QUIT fails the worker sees EOF and recovers.
        let _ = self.write("QUIT");
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait()? {
                if !status.success() {
                    bail!("风扇守护退出失败；自动恢复可能未完成");
                }
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
