//! Experimental native sensor access for the locally verified Ryzen 7 8745H.
//!
//! Connects only to an already running WinRing0. Never installs, starts, stops,
//! or deletes a driver, and does not depend on the vendor's managed DLLs.
//! SMN sampling requires changing the PCI address selector, not a sensor setting.
use anyhow::{bail, Context, Result};
use windows::{
    core::w,
    Win32::{
        Foundation::{
            CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0,
        },
        Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE, OPEN_EXISTING},
        System::{
            Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
            IO::DeviceIoControl,
        },
    },
};

const fn ioctl(function: u32, access: u32) -> u32 {
    (40000 << 16) | (access << 14) | (function << 2)
}
const VERSION: u32 = ioctl(0x800, 0);
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
    }
}
