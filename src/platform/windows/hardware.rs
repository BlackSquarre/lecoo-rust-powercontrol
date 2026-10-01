use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::System::Wmi::*,
    Win32::Security::*,
    Win32::Foundation::*,
};
use anyhow::{Result, Context, bail};
use std::time::{Duration, Instant};
use crate::platform::traits::HardwareControl;
use crate::core::types::{PowerMode, FeatureKey, is_valid_fan_reading, is_valid_temperature};

const WMI_NAMESPACE: &str = "root\\WMI";
const WMI_CLASS: &str = "PowerSwitchInterface";
const INSTANCE_NAME: &str = "ACPI\\PNP0C14\\IP3POWERSWITCH_0";

pub struct WindowsHardwareControl {
    wmi_service: IWbemServices,
    instance_path: BSTR,
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
            let wmi_service = locator.ConnectServer(
                &namespace,
                &BSTR::new(),
                &BSTR::new(),
                &BSTR::new(),
                0,
                &BSTR::new(),
                None,
            ).context("连接 WMI 命名空间失败")?;

            // 设置代理安全级别
            // RPC_C_AUTHN_WINNT = 10, RPC_C_AUTHZ_NONE = 0
            CoSetProxyBlanket(
                &wmi_service,
                10,  // RPC_C_AUTHN_WINNT
                0,   // RPC_C_AUTHZ_NONE
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            ).context("设置 WMI 代理安全级别失败")?;

            // 构造实例路径
            let instance_path = BSTR::from(format!(
                "{}.InstanceName=\"{}\"",
                WMI_CLASS,
                INSTANCE_NAME.replace('\\', "\\\\").replace('"', "\\\"")
            ));

            Ok(Self {
                wmi_service,
                instance_path,
            })
        }
    }

    unsafe fn invoke_method(&self, method_name: &str, in_params: Option<&IWbemClassObject>) -> Result<IWbemClassObject> {
        let method_bstr = BSTR::from(method_name);

        let mut out_params: Option<IWbemClassObject> = None;

        self.wmi_service.ExecMethod(
            &self.instance_path,
            &method_bstr,
            WBEM_FLAG_RETURN_WBEM_COMPLETE,
            None,
            in_params,
            Some(&mut out_params),
            None,
        ).context(format!("调用 WMI 方法 {} 失败", method_name))?;

        let out_params = out_params.ok_or_else(|| anyhow::anyhow!("WMI 方法 {} 未返回结果", method_name))?;
        let mut return_value = VARIANT::default();
        // 此 ACPI 提供程序的原生 COM 输出可能省略 ReturnValue，CIM 会补出该字段。
        match out_params.Get(&BSTR::from("ReturnValue"), 0, &mut return_value, None, None) {
            Ok(()) => {
                if !bool::try_from(&return_value).context("转换 WMI ReturnValue 失败")? {
                    bail!("WMI 方法 {} 返回失败", method_name);
                }
            }
            Err(error) if error.code().0 == WBEM_E_NOT_FOUND.0 => {}
            Err(error) => return Err(error).context(format!("读取 {} ReturnValue 失败", method_name)),
        }
        Ok(out_params)
    }

    unsafe fn get_u32_from_result(&self, result: &IWbemClassObject, prop_name: &str) -> Result<u32> {
        let prop_bstr = BSTR::from(prop_name);
        let mut variant = VARIANT::default();

        result.Get(
            &prop_bstr,
            0,
            &mut variant,
            None,
            None,
        ).context(format!("获取属性 {} 失败", prop_name))?;

        u32::try_from(&variant).context(format!("转换属性 {} 为 u32 失败", prop_name))
    }

    unsafe fn create_in_params(&self, method_name: &str) -> Result<IWbemClassObject> {
        let class_bstr = BSTR::from(WMI_CLASS);

        let mut class_obj: Option<IWbemClassObject> = None;

        self.wmi_service.GetObject(
            &class_bstr,
            WBEM_FLAG_RETURN_WBEM_COMPLETE,
            None,
            Some(&mut class_obj),
            None,
        ).context("获取 WMI 类对象失败")?;

        let class_obj = class_obj.ok_or_else(|| anyhow::anyhow!("WMI 类对象为空"))?;

        let method_bstr = BSTR::from(method_name);
        let mut in_signature: Option<IWbemClassObject> = None;
        let mut out_signature: Option<IWbemClassObject> = None;

        class_obj.GetMethod(
            &method_bstr,
            0,
            &mut in_signature,
            &mut out_signature,
        ).context(format!("获取方法 {} 的输入参数失败", method_name))?;

        let in_signature = in_signature.ok_or_else(|| anyhow::anyhow!("方法签名为空"))?;

        let in_params_instance = in_signature.SpawnInstance(0)
            .context("创建输入参数实例失败")?;

        Ok(in_params_instance)
    }

    unsafe fn set_u8_param(&self, params: &IWbemClassObject, name: &str, value: u8) -> Result<()> {
        let prop_bstr = BSTR::from(name);

        let variant = VARIANT::from(value);

        let result = params.Put(
            &prop_bstr,
            0,
            &variant,
            0,
        );

        result.context(format!("设置参数 {} 失败", name))?;

        Ok(())
    }
}

impl HardwareControl for WindowsHardwareControl {
    fn get_power_mode(&self) -> Result<PowerMode> {
        unsafe {
            let result = self.invoke_method("GetPowerMode", None)?;
            let mode_value = self.get_u32_from_result(&result, "CurrentPowerMode")?;

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
            self.set_u8_param(&in_params, "PowerMode", mode as u8)?;

            let result = self.invoke_method("SetPowerMode", Some(&in_params))?;
            let status = self.get_u32_from_result(&result, "ResultStatus")?;
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
                    bail!("模式切换验证失败：预期 {:?}，实际 {:?}，ResultStatus={}", mode, actual, status);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    fn get_fan_speed(&self, fan_num: u8) -> Result<Option<u32>> {
        unsafe {
            let in_params = self.create_in_params("GetFanControl")?;
            self.set_u8_param(&in_params, "FanNumber", fan_num)?;

            let result = self.invoke_method("GetFanControl", Some(&in_params))?;
            let rpm = self.get_u32_from_result(&result, "FanDuty")?;

            Ok(if is_valid_fan_reading(rpm) { Some(rpm) } else { None })
        }
    }

    fn get_hw_temp(&self, temp_type: u8) -> Result<Option<u32>> {
        unsafe {
            let in_params = self.create_in_params("GetHwTemp")?;
            self.set_u8_param(&in_params, "HwTempType", temp_type)?;

            let result = self.invoke_method("GetHwTemp", Some(&in_params))?;
            let temp = self.get_u32_from_result(&result, "Temp")?;

            Ok(if is_valid_temperature(temp) { Some(temp) } else { None })
        }
    }

    fn get_feature_value(&self, key: FeatureKey) -> Result<Option<u32>> {
        unsafe {
            let in_params = self.create_in_params("GetFeatureValue")?;
            self.set_u8_param(&in_params, "Reserved1", key as u8)?;
            for name in ["Reserved2", "Reserved3", "Reserved4"] {
                self.set_u8_param(&in_params, name, 0)?;
            }

            let result = self.invoke_method("GetFeatureValue", Some(&in_params))?;
            let value = self.get_u32_from_result(&result, "ResultStatus")?;

            Ok(if value != 255 { Some(value) } else { None })
        }
    }

    fn is_elevated(&self) -> bool {
        self.is_elevated_impl().unwrap_or(false)
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
            ).is_err() {
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
