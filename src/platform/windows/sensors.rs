//! Experimental native sensor access for the locally verified Ryzen 7 8745H.
//!
//! Connects only to an already running WinRing0. Never installs, starts, stops,
//! or deletes a driver, and does not depend on the vendor's managed DLLs.
//! SMN sampling requires changing the PCI address selector, not a sensor setting.
use anyhow::{bail, Context, Result};
use std::time::{Duration, Instant};
use windows::{
    core::w,
    Win32::{
        Foundation::{
            CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0,
        },
        Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE, OPEN_EXISTING},
        System::{
            Threading::{
                CreateMutexW, GetCurrentThread, ReleaseMutex, SetThreadAffinityMask,
                WaitForSingleObject,
            },
            IO::DeviceIoControl,
        },
    },
};

const fn ioctl(function: u32, access: u32) -> u32 {
    (40000 << 16) | (access << 14) | (function << 2)
}
const VERSION: u32 = ioctl(0x800, 0);
const READ_MSR: u32 = ioctl(0x821, 0);
const AMD_RAPL_POWER_UNIT: u32 = 0xc001_0299;
const AMD_PACKAGE_ENERGY: u32 = 0xc001_029b;
const READ_PCI: u32 = ioctl(0x851, 1);
const WRITE_PCI: u32 = ioctl(0x852, 2);
const THM_TCON_TEMP: u32 = 0x59800;

struct OwnedHandle(HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

struct PciLock(OwnedHandle);
impl PciLock {
    fn acquire() -> Result<Self> {
        let handle = OwnedHandle(
            unsafe { CreateMutexW(None, false, w!("Global\\Access_PCI")) }
                .context("Cannot open the shared PCI mutex; sampling cancelled")?,
        );
        let result = unsafe { WaitForSingleObject(handle.0, 100) };
        if result == WAIT_ABANDONED {
            // We own the abandoned mutex; release it even though sampling is rejected.
            let _guard = Self(handle);
            bail!("Shared PCI mutex was abandoned; sampling cancelled");
        }
        if result != WAIT_OBJECT_0 {
            bail!("Shared PCI mutex unavailable ({result:?}); sampling cancelled");
        }
        Ok(Self(handle))
    }
}
impl Drop for PciLock {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.0 .0);
        }
    }
}

/// A source-labelled temperature with its unmodified register value.
#[derive(Debug, Clone, Copy)]
pub struct CpuTemperatureReading {
    pub raw_register: u32,
    pub celsius: f32,
}

/// Package energy telemetry, not a configured TDP or power limit.
#[derive(Debug, Clone, Copy)]
pub struct CpuPackageEnergyReading {
    pub raw_unit_register: u64,
    pub counter: u32,
    pub sampled_at: Instant,
}

impl CpuPackageEnergyReading {
    /// Average watts between two samples of the same package. No blocking wait.
    pub fn watts_since(&self, previous: &Self) -> Result<f64> {
        let elapsed = self
            .sampled_at
            .checked_duration_since(previous.sampled_at)
            .context("Package energy samples are out of order")?;
        package_watts(
            previous.counter,
            self.counter,
            previous.raw_unit_register,
            self.raw_unit_register,
            elapsed,
        )
    }
}

fn package_watts(
    previous: u32,
    current: u32,
    previous_unit: u64,
    unit: u64,
    elapsed: Duration,
) -> Result<f64> {
    if unit == 0 || unit == u64::MAX || previous_unit != unit {
        bail!("Invalid or changed AMD energy unit register");
    }
    // Keep the interval short enough that multiple 32-bit wraps cannot be missed.
    if elapsed < Duration::from_millis(100) || elapsed > Duration::from_secs(5) {
        bail!("Package power requires a sampling interval between 100 ms and 5 s");
    }
    let delta = current.wrapping_sub(previous);
    if delta == 0 {
        bail!("Package energy counter did not advance");
    }
    let exponent = ((unit >> 8) & 0x1f) as i32;
    let watts = f64::from(delta) * 2f64.powi(-exponent) / elapsed.as_secs_f64();
    if !watts.is_finite() || !(0.0..=250.0).contains(&watts) {
        bail!("Implausible Ryzen 7 8745H package power {watts} W");
    }
    Ok(watts)
}

/// Converts AMD THM_TCON_TEMP using the algorithm confirmed in the local DLL.
/// The validation bounds reject invalid data; they are not thermal control limits.
pub fn decode_package_temperature(raw: u32) -> Result<f32> {
    if raw == 0 || raw == u32::MAX || raw == 0x7fff_ffff {
        bail!("Invalid temperature register: 0x{raw:08x}");
    }
    let mut value = ((raw >> 21) & 0x7ff) as f32 / 8.0;
    if raw & 0x80000 != 0 {
        value -= 49.0;
    }
    if !(0.0..=125.0).contains(&value) {
        bail!("Implausible package temperature {value} C (raw=0x{raw:08x})");
    }
    Ok(value)
}

#[cfg(target_arch = "x86_64")]
pub fn cpu_brand() -> String {
    use std::arch::x86_64::__cpuid;
    let mut bytes = Vec::with_capacity(48);
    {
        if __cpuid(0x80000000).eax < 0x80000004 {
            return String::new();
        }
        for leaf in 0x80000002..=0x80000004 {
            let registers = __cpuid(leaf);
            for value in [registers.eax, registers.ebx, registers.ecx, registers.edx] {
                bytes.extend(value.to_le_bytes());
            }
        }
    }
    String::from_utf8_lossy(&bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_owned()
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cpu_brand() -> String {
    String::new()
}

pub struct ExistingWinRing0 {
    device: OwnedHandle,
    version: u32,
}

impl ExistingWinRing0 {
    pub fn open() -> Result<Self> {
        if !cpu_brand().starts_with("AMD Ryzen 7 8745H ") {
            bail!("Native sampling is only validated for AMD Ryzen 7 8745H");
        }
        let device = OwnedHandle(unsafe {
            CreateFileW(w!("\\\\.\\WinRing0_1_2_0"), GENERIC_READ.0 | GENERIC_WRITE.0,
                FILE_SHARE_MODE(0), None, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, None)
        }.context("Cannot open existing WinRing0; administrator access and a running driver are required")?);
        let mut driver = Self { device, version: 0 };
        driver.version = driver.read_u32(VERSION, &[])?;
        if driver.version != 0x01020005 {
            bail!(
                "Unexpected driver version 0x{:08x}; sampling cancelled",
                driver.version
            );
        }
        Ok(driver)
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    // Only the two known read-only AMD telemetry MSRs are exposed publicly below.
    fn read_msr(&self, register: u32) -> Result<u64> {
        let mut value = 0u64;
        let mut returned = 0u32;
        unsafe {
            DeviceIoControl(
                self.device.0,
                READ_MSR,
                Some((&register as *const u32).cast()),
                4,
                Some((&mut value as *mut u64).cast()),
                8,
                Some(&mut returned),
                None,
            )
        }
        .with_context(|| format!("Cannot read AMD MSR 0x{register:08x}"))?;
        if returned != 8 {
            bail!("Unexpected MSR output length: {returned}");
        }
        Ok(value)
    }

    /// Reads energy on logical CPU 0 of the locally validated single-package CPU.
    /// Does not write MSRs, change power limits, or load another monitoring stack.
    pub fn read_package_energy(&self) -> Result<CpuPackageEnergyReading> {
        let thread = unsafe { GetCurrentThread() };
        let previous_affinity = unsafe { SetThreadAffinityMask(thread, 1) };
        if previous_affinity == 0 {
            return Err(windows::core::Error::from_win32())
                .context("Cannot pin package energy sampling to CPU 0");
        }
        let reading = (|| {
            let raw_unit_register = self.read_msr(AMD_RAPL_POWER_UNIT)?;
            let raw_energy = self.read_msr(AMD_PACKAGE_ENERGY)?;
            Ok(CpuPackageEnergyReading {
                raw_unit_register,
                counter: raw_energy as u32,
                sampled_at: Instant::now(),
            })
        })();
        if unsafe { SetThreadAffinityMask(thread, previous_affinity) } == 0 {
            return Err(windows::core::Error::from_win32())
                .context("Cannot restore sampling thread affinity");
        }
        reading
    }

    // Private transport: the public API cannot perform arbitrary register writes.
    fn read_u32(&self, code: u32, input: &[u32]) -> Result<u32> {
        let mut value = 0u32;
        let mut returned = 0u32;
        unsafe {
            DeviceIoControl(
                self.device.0,
                code,
                if input.is_empty() {
                    None
                } else {
                    Some(input.as_ptr().cast())
                },
                std::mem::size_of_val(input) as u32,
                Some((&mut value as *mut u32).cast()),
                4,
                Some(&mut returned),
                None,
            )
        }
        .context(format!("WinRing0 read failed (IOCTL=0x{code:08x})"))?;
        if returned != 4 {
            bail!("Unexpected WinRing0 output length: {returned}");
        }
        Ok(value)
    }

    fn select_smn(&self, address: u32) -> Result<()> {
        // PCI BDF 0:0:0, offset 0x60 is the SMN address selector.
        let input = [0u32, 0x60, address];
        let mut returned = 0;
        unsafe {
            DeviceIoControl(
                self.device.0,
                WRITE_PCI,
                Some(input.as_ptr().cast()),
                12,
                None,
                0,
                Some(&mut returned),
                None,
            )
        }
        .context("Cannot select/restore the SMN address")
    }

    pub fn read_package_temperature(&self) -> Result<CpuTemperatureReading> {
        let _lock = PciLock::acquire()?;
        let vendor = self.read_u32(READ_PCI, &[0, 0])? & 0xffff;
        if vendor != 0x1022 {
            bail!("PCI 0:0:0 is not AMD: 0x{vendor:04x}");
        }
        let previous_selector = self.read_u32(READ_PCI, &[0, 0x60])?;
        let reading = (|| {
            self.select_smn(THM_TCON_TEMP)?;
            let raw_register = self.read_u32(READ_PCI, &[0, 0x64])?;
            Ok(CpuTemperatureReading {
                raw_register,
                celsius: decode_package_temperature(raw_register)?,
            })
        })();
        // Attempt restoration after every attempted selector change, including failed IOCTLs.
        let restored = self.select_smn(previous_selector).and_then(|()| {
            let actual = self.read_u32(READ_PCI, &[0, 0x60])?;
            if actual != previous_selector {
                bail!("Selector readback mismatch: expected 0x{previous_selector:08x}, got 0x{actual:08x}");
            }
            Ok(())
        });
        match (reading, restored) {
            (Ok(reading), Ok(())) => Ok(reading),
            (Err(error), Ok(())) => Err(error),
            (result, Err(error)) => {
                bail!("SMN selector restoration failed: {error:#}; sampling result: {result:?}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temperature_encoding_and_range_bit() {
        assert_eq!(decode_package_temperature(400 << 21).unwrap(), 50.0);
        assert_eq!(
            decode_package_temperature((792 << 21) | 0x80000).unwrap(),
            50.0
        );
        assert_eq!(decode_package_temperature(401 << 21).unwrap(), 50.125);
    }
    #[test]
    fn rejects_invalid_or_implausible_registers() {
        for raw in [0, u32::MAX, 0x7fff_ffff, 2047 << 21, (100 << 21) | 0x80000] {
            assert!(decode_package_temperature(raw).is_err(), "0x{raw:08x}");
        }
    }
    #[test]
    fn protocol_codes_match_winring0_header() {
        assert_eq!(VERSION, 0x9c402000);
        assert_eq!(READ_PCI, 0x9c406144);
        assert_eq!(WRITE_PCI, 0x9c40a148);
        assert_eq!(READ_MSR, 0x9c402084);
    }
    #[test]
    fn package_power_uses_energy_unit_time_and_full_counter_wrap() {
        let unit = 16 << 8;
        assert_eq!(
            package_watts(0, 655_360, unit, unit, Duration::from_millis(500)).unwrap(),
            20.0
        );
        assert_eq!(
            package_watts(
                u32::MAX - 32_767,
                32_768,
                unit,
                unit,
                Duration::from_secs(1)
            )
            .unwrap(),
            1.0
        );
    }
    #[test]
    fn package_power_rejects_invalid_stale_or_ambiguous_samples() {
        let unit = 16 << 8;
        for (before, after, old_unit, new_unit, time) in [
            (7, 7, unit, unit, Duration::from_secs(1)),
            (0, 655_360, unit, 0, Duration::from_secs(1)),
            (0, 655_360, unit, unit + 256, Duration::from_secs(1)),
            (0, 655_360, unit, unit, Duration::from_millis(99)),
            (0, 655_360, unit, unit, Duration::from_secs(6)),
            (0, u32::MAX, unit, unit, Duration::from_secs(1)),
        ] {
            assert!(package_watts(before, after, old_unit, new_unit, time).is_err());
        }
    }
}
