use crate::core::types::{is_valid_fan_reading, is_valid_temperature, FeatureKey, PowerMode};
use crate::platform::traits::HardwareControl;
use anyhow::{bail, Context, Result};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
use windows::{
    core::*, Win32::Foundation::*, Win32::Security::*, Win32::System::Com::*, Win32::System::Wmi::*,
};

const WMI_NAMESPACE: &str = "root\\WMI";
const WMI_CLASS: &str = "PowerSwitchInterface";
const INSTANCE_NAME: &str = "ACPI\\PNP0C14\\IP3POWERSWITCH_0";
const THERMAL_ZONE_INSTANCE: &str = "ACPI\\ThermalZone\\TZ01_0";

fn acpi_celsius(raw: u32) -> Option<f32> {
    let celsius = raw as f64 / 10.0 - 273.15;
    (raw != 0 && (0.0..=125.0).contains(&celsius)).then_some(celsius as f32)
}

fn cpu_brand() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::__cpuid;
        if __cpuid(0x80000000).eax < 0x80000004 {
            return String::new();
        }
        let mut bytes = [0u8; 48];
        for leaf in 0x80000002..=0x80000004 {
            let result = __cpuid(leaf);
            for (index, value) in [result.eax, result.ebx, result.ecx, result.edx]
                .into_iter()
                .enumerate()
            {
                let offset = (leaf - 0x80000002) as usize * 16 + index * 4;
                bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        String::from_utf8_lossy(&bytes)
            .trim_matches('\0')
            .trim()
            .to_owned()
    }
    #[cfg(not(target_arch = "x86_64"))]
    String::new()
}

pub struct WindowsHardwareControl {
    wmi_service: IWbemServices,
    instance_path: BSTR,
    // Fixed private method set; never cache mutable argument instances.
    input_signatures: RefCell<Vec<(&'static str, IWbemClassObject)>>,
    thermal_path: RefCell<Option<BSTR>>,
}

// COM 对象只在初始化它的 GUI 线程上使用。
impl WindowsHardwareControl {
    pub fn new() -> Result<Self> {
        unsafe {
            // 获取 WMI 定位器
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
                .context("创建 WbemLocator 失败")?;

            // 连接到 WMI 命名空间
            let namespace = BSTR::from(WMI_NAMESPACE);
            let wmi_service = locator
                .ConnectServer(
                    &namespace,
                    &BSTR::new(),
                    &BSTR::new(),
                    &BSTR::new(),
                    0,
                    &BSTR::new(),
                    None,
                )
                .context("连接 WMI 命名空间失败")?;

            // 设置代理安全级别
            // RPC_C_AUTHN_WINNT = 10, RPC_C_AUTHZ_NONE = 0
            CoSetProxyBlanket(
                &wmi_service,
                10, // RPC_C_AUTHN_WINNT
                0,  // RPC_C_AUTHZ_NONE
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            )
            .context("设置 WMI 代理安全级别失败")?;

            // 构造实例路径
            let instance_path = BSTR::from(format!(
                "{}.InstanceName=\"{}\"",
                WMI_CLASS,
                INSTANCE_NAME.replace('\\', "\\\\").replace('"', "\\\"")
            ));

            Ok(Self {
                wmi_service,
                instance_path,
                input_signatures: RefCell::new(Vec::new()),
                thermal_path: RefCell::new(None),
            })
        }
    }

    unsafe fn invoke_method(
        &self,
        method_name: &str,
        in_params: Option<&IWbemClassObject>,
    ) -> Result<IWbemClassObject> {
        let method_bstr = BSTR::from(method_name);

        let mut out_params: Option<IWbemClassObject> = None;

        self.wmi_service
            .ExecMethod(
                &self.instance_path,
                &method_bstr,
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                in_params,
                Some(&mut out_params),
                None,
            )
            .with_context(|| format!("调用 WMI 方法 {} 失败", method_name))?;

        let out_params =
            out_params.ok_or_else(|| anyhow::anyhow!("WMI 方法 {} 未返回结果", method_name))?;
        let mut return_value = VARIANT::default();
        // 此 ACPI 提供程序的原生 COM 输出可能省略 ReturnValue，CIM 会补出该字段。
        match out_params.Get(w!("ReturnValue"), 0, &mut return_value, None, None) {
            Ok(()) => {
                if !bool::try_from(&return_value).context("转换 WMI ReturnValue 失败")? {
                    bail!("WMI 方法 {} 返回失败", method_name);
                }
            }
            Err(error) if error.code().0 == WBEM_E_NOT_FOUND.0 => {}
            Err(error) => {
                return Err(error).context(format!("读取 {} ReturnValue 失败", method_name))
            }
        }
        Ok(out_params)
    }

    unsafe fn get_u32_from_result(
        &self,
        result: &IWbemClassObject,
        prop_name: PCWSTR,
    ) -> Result<u32> {
        let mut variant = VARIANT::default();

        result
            .Get(prop_name, 0, &mut variant, None, None)
            .with_context(|| {
                format!(
                    "获取属性 {} 失败",
                    prop_name.to_string().unwrap_or_default()
                )
            })?;

        u32::try_from(&variant).with_context(|| {
            format!(
                "转换属性 {} 为 u32 失败",
                prop_name.to_string().unwrap_or_default()
            )
        })
    }

    unsafe fn create_in_params(&self, method_name: &'static str) -> Result<IWbemClassObject> {
        {
            let signatures = self.input_signatures.borrow();
            if let Some((_, signature)) = signatures.iter().find(|(name, _)| *name == method_name) {
                return signature.SpawnInstance(0).context("创建输入参数实例失败");
            }
        }
        let class_bstr = BSTR::from(WMI_CLASS);

        let mut class_obj: Option<IWbemClassObject> = None;

        self.wmi_service
            .GetObject(
                &class_bstr,
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                Some(&mut class_obj),
                None,
            )
            .context("获取 WMI 类对象失败")?;

        let class_obj = class_obj.ok_or_else(|| anyhow::anyhow!("WMI 类对象为空"))?;

        let method_bstr = BSTR::from(method_name);
        let mut in_signature: Option<IWbemClassObject> = None;
        let mut out_signature: Option<IWbemClassObject> = None;

        class_obj
            .GetMethod(&method_bstr, 0, &mut in_signature, &mut out_signature)
            .with_context(|| format!("获取方法 {} 的输入参数失败", method_name))?;

        let in_signature = in_signature.ok_or_else(|| anyhow::anyhow!("方法签名为空"))?;

        let in_params_instance = in_signature
            .SpawnInstance(0)
            .context("创建输入参数实例失败")?;
        self.input_signatures
            .borrow_mut()
            .push((method_name, in_signature));

        Ok(in_params_instance)
    }

    unsafe fn set_u8_param(
        &self,
        params: &IWbemClassObject,
        name: PCWSTR,
        value: u8,
    ) -> Result<()> {
        let variant = VARIANT::from(value);

        let result = params.Put(name, 0, &variant, 0);

        result
            .with_context(|| format!("设置参数 {} 失败", name.to_string().unwrap_or_default()))?;

        Ok(())
    }
}

impl HardwareControl for WindowsHardwareControl {
    fn get_power_mode(&self) -> Result<PowerMode> {
        unsafe {
            let result = self.invoke_method("GetPowerMode", None)?;
            let mode_value = self.get_u32_from_result(&result, w!("CurrentPowerMode"))?;

            PowerMode::from_u32(mode_value)
                .ok_or_else(|| anyhow::anyhow!("无效的性能模式值: {}", mode_value))
        }
    }

    fn set_power_mode(&mut self, mode: PowerMode) -> Result<()> {
        if !self.is_elevated() {
            bail!("需要管理员权限才能切换性能模式");
        }
        unsafe {
            let in_params = self.create_in_params("SetPowerMode")?;
            self.set_u8_param(&in_params, w!("PowerMode"), mode as u8)?;

            let result = self.invoke_method("SetPowerMode", Some(&in_params))?;
            let status = self.get_u32_from_result(&result, w!("ResultStatus"))?;
            if status == 255 {
                bail!("模式切换被硬件拒绝，ResultStatus={}", status);
            }

            // ResultStatus 的其它值未有完整厂商定义，以实际读回为成功依据。
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let actual = self.get_power_mode().context("模式切换后读取验证失败")?;
                if actual == mode {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    bail!(
                        "模式切换验证失败：预期 {:?}，实际 {:?}，ResultStatus={}",
                        mode,
                        actual,
                        status
                    );
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    fn get_fan_speed(&self, fan_num: u8) -> Result<Option<u32>> {
        unsafe {
            let in_params = self.create_in_params("GetFanControl")?;
            self.set_u8_param(&in_params, w!("FanNumber"), fan_num)?;

            let result = self.invoke_method("GetFanControl", Some(&in_params))?;
            let packed = self.get_u32_from_result(&result, w!("FanDuty"))?;

            if !is_valid_fan_reading(packed) || packed == u32::MAX {
                return Ok(None);
            }
            // The official provider packs both tachometers into FanNumber=1.
            match fan_num {
                1 => Ok(Some(packed & 0xffff)),
                _ => Ok(None),
            }
        }
    }

    fn get_hw_temp(&self, temp_type: u8) -> Result<Option<u32>> {
        unsafe {
            let in_params = self.create_in_params("GetHwTemp")?;
            self.set_u8_param(&in_params, w!("HwTempType"), temp_type)?;

            let result = self.invoke_method("GetHwTemp", Some(&in_params))?;
            let temp = self.get_u32_from_result(&result, w!("Temp"))?;

            Ok(if is_valid_temperature(temp) {
                Some(temp)
            } else {
                None
            })
        }
    }

    fn get_feature_value(&self, key: FeatureKey) -> Result<Option<u32>> {
        unsafe {
            let in_params = self.create_in_params("GetFeatureValue")?;
            self.set_u8_param(&in_params, w!("Reserved1"), key as u8)?;
            for name in [w!("Reserved2"), w!("Reserved3"), w!("Reserved4")] {
                self.set_u8_param(&in_params, name, 0)?;
            }

            let result = self.invoke_method("GetFeatureValue", Some(&in_params))?;
            let value = self.get_u32_from_result(&result, w!("ResultStatus"))?;

            Ok(if value != 255 { Some(value) } else { None })
        }
    }

    fn is_elevated(&self) -> bool {
        self.is_elevated_impl().unwrap_or(false)
    }

    fn get_thermal_zone_temperature(&self) -> Result<Option<f32>> {
        unsafe {
            let cached_object = {
                let cached = self.thermal_path.borrow();
                let mut object = None;
                if let Some(path) = cached.as_ref() {
                    if self
                        .wmi_service
                        .GetObject(
                            path,
                            WBEM_FLAG_RETURN_WBEM_COMPLETE,
                            None,
                            Some(&mut object),
                            None,
                        )
                        .is_err()
                    {
                        object = None;
                    }
                }
                object
            };
            if let Some(object) = cached_object {
                return self
                    .get_u32_from_result(&object, w!("CurrentTemperature"))
                    .map(acpi_celsius);
            }
            // A disappeared/replaced provider must be rediscovered.
            self.thermal_path.borrow_mut().take();
            let instances = self
                .wmi_service
                .CreateInstanceEnum(
                    &BSTR::from("MSAcpi_ThermalZoneTemperature"),
                    WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                    None,
                )
                .context("枚举 ACPI 热区失败")?;
            loop {
                let mut objects = [None];
                let mut returned = 0;
                let status = instances.Next(2000, &mut objects, &mut returned);
                status.ok().context("读取 ACPI 热区失败")?;
                if status.0 == WBEM_S_TIMEDOUT.0 {
                    bail!("读取 ACPI 热区超时");
                }
                if returned == 0 {
                    return Ok(None);
                }
                let object = objects[0].take().context("ACPI 热区对象为空")?;
                let mut name = VARIANT::default();
                object.Get(w!("InstanceName"), 0, &mut name, None, None)?;
                let name = BSTR::try_from(&name)?.to_string();
                if name.eq_ignore_ascii_case(THERMAL_ZONE_INSTANCE) {
                    let mut path = VARIANT::default();
                    if object.Get(w!("__PATH"), 0, &mut path, None, None).is_ok() {
                        *self.thermal_path.borrow_mut() = BSTR::try_from(&path).ok();
                    }
                    let raw = self.get_u32_from_result(&object, w!("CurrentTemperature"))?;
                    return Ok(acpi_celsius(raw));
                }
            }
        }
    }
}

impl crate::core::cooling::CoolingIo for WindowsHardwareControl {
    fn write_fan(&mut self, target: crate::core::cooling::FanTarget) -> Result<u32> {
        target.validate()?;
        if !self.is_elevated() {
            bail!("风扇控制需要管理员权限");
        }
        if !cpu_brand().starts_with("AMD Ryzen 7 8745H ")
            || self.get_feature_value(FeatureKey::FanCount)? != Some(1)
        {
            bail!("仅开放已验证的 8745H 单风扇硬件");
        }
        if matches!(target, crate::core::cooling::FanTarget::Percent(35..=99)) {
            bail!("ACPI 热区尚未验证为 CPU 保护温度，手动风扇目标暂不可用");
        }
        unsafe {
            let input = self.create_in_params("SetFanControl")?;
            self.set_u8_param(&input, w!("FanNumber"), 1)?;
            self.set_u8_param(&input, w!("FanDuty"), target.encoded())?;
            let output = self.invoke_method("SetFanControl", Some(&input))?;
            self.get_u32_from_result(&output, w!("ResultStatus"))
        }
    }
    fn measured_rpm(&self) -> Result<u32> {
        self.get_fan_speed(1)?.context("风扇转速不可用")
    }
    fn temperature(&self) -> Result<f32> {
        self.get_thermal_zone_temperature()?
            .context("ACPI 热区温度不可用")
    }
}

impl WindowsHardwareControl {
    fn is_elevated_impl(&self) -> Result<bool> {
        unsafe {
            // GetCurrentProcess 返回伪句柄 -1
            let process_handle = HANDLE(-1isize as *mut _);
            let mut token: HANDLE = HANDLE::default();

            if windows::Win32::System::Threading::OpenProcessToken(
                process_handle,
                TOKEN_QUERY,
                &mut token,
            )
            .is_err()
            {
                return Ok(false);
            }

            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0u32;

            let success = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );

            let _ = CloseHandle(token);

            Ok(success.is_ok() && elevation.TokenIsElevated != 0)
        }
    }
}

#[cfg(test)]
mod thermal_zone_tests {
    use super::acpi_celsius;

    #[test]
    fn converts_tenths_kelvin_and_rejects_invalid_values() {
        assert!((acpi_celsius(3112).unwrap() - 38.05).abs() < 0.001);
        for raw in [0, 2731, 2147483647, u32::MAX, 5000] {
            assert!(acpi_celsius(raw).is_none());
        }
    }
}
