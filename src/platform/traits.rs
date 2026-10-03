use crate::core::types::{FeatureKey, PowerMode};
use anyhow::Result;

/// 硬件控制统一接口
pub trait HardwareControl {
    /// 获取当前性能模式
    fn get_power_mode(&self) -> Result<PowerMode>;

    /// 设置性能模式（需要管理员权限）
    fn set_power_mode(&mut self, mode: PowerMode) -> Result<()>;

    /// 获取风扇转速（返回 None 表示不支持）
    fn get_fan_speed(&self, fan_num: u8) -> Result<Option<u32>>;

    /// 获取硬件温度（返回 None 表示不支持）
    /// temp_type: 0 或 1
    fn get_hw_temp(&self, temp_type: u8) -> Result<Option<u32>>;

    /// Temperature of the observed ACPI thermal zone, in Celsius.
    fn get_thermal_zone_temperature(&self) -> Result<Option<f32>> {
        Ok(None)
    }

    /// 获取功能值（返回 None 表示不支持）
    fn get_feature_value(&self, key: FeatureKey) -> Result<Option<u32>>;

    /// 检查是否有管理员权限
    fn is_elevated(&self) -> bool;
}
