/// 性能模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PowerMode {
    Balance = 0,
    Performance = 1,
    Quiet = 2,
}

impl PowerMode {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(PowerMode::Balance),
            1 => Some(PowerMode::Performance),
            2 => Some(PowerMode::Quiet),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PowerMode::Balance => "均衡模式",
            PowerMode::Performance => "性能模式",
            PowerMode::Quiet => "安静模式",
        }
    }

    pub fn name_en(&self) -> &'static str {
        match self {
            PowerMode::Balance => "Balance",
            PowerMode::Performance => "Performance",
            PowerMode::Quiet => "Quiet",
        }
    }
}

/// 功能查询键
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FeatureKey {
    OsdStatus = 1,
    ModeCount = 2,
    FanCount = 3,
    GpuCapability = 4,
    CpuTurbo = 5,  // 测试显示不支持（返回255）
    LightMode = 7, // 测试显示不支持（返回255）
}

/// 风扇控制模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanMode {
    Auto,    // 自动模式
    Custom,  // 自定义模式
    Maximum, // 最大模式
}

impl FanMode {
    pub fn name(&self) -> &'static str {
        match self {
            FanMode::Auto => "自动",
            FanMode::Custom => "自定义",
            FanMode::Maximum => "最大",
        }
    }
}

/// 哨兵值常量
pub mod sentinel {
    pub const I32_MAX: i32 = 2147483647;
    pub const U32_MAX: u32 = 2147483647;
    pub const U8_MAX: u8 = 255;
}

/// 检查是否为有效的风扇读数
pub fn is_valid_fan_reading(duty: u32) -> bool {
    duty != sentinel::U32_MAX
}

/// 检查是否为有效的温度读数
pub fn is_valid_temperature(temp: u32) -> bool {
    temp != sentinel::U32_MAX
}

/// 检查是否为有效的功能值
pub fn is_valid_feature_value(status: u8) -> bool {
    status != sentinel::U8_MAX
}

/// 系统状态快照
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: std::time::Instant,
    pub power_mode: Option<PowerMode>,
    pub power_mode_error: Option<String>,
    pub fan_speed: Option<u32>,
    pub cpu_temp: Option<f32>,
    pub cpu_package_power: Option<f64>,
    pub disk_usage: f32,
    pub disk_total: u64,
    pub disk_used: u64,
    pub mem_usage: f32,
    pub mem_total: u64,
    pub mem_used: u64,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            timestamp: std::time::Instant::now(),
            power_mode: None,
            power_mode_error: None,
            fan_speed: None,
            cpu_temp: None,
            cpu_package_power: None,
            disk_usage: 0.0,
            disk_total: 0,
            disk_used: 0,
            mem_usage: 0.0,
            mem_total: 0,
            mem_used: 0,
        }
    }
}
