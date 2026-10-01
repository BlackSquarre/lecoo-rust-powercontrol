# 官方 UI 设计参考

**来源**: 原厂控制中心截图分析  
**日期**: 2026-10-02  
**用途**: UI 设计和功能规划参考

---

## 🎨 UI 布局分析

### 主界面（截图 1）

**左侧导航栏**（垂直排列，4个按钮）:
1. 🎛️ **模式设置** - 主页面（默认选中）
2. 🌀 **风扇控制** - 风扇调节页面
3. ⚙️ **功能设定** - 高级设置页面
4. 📋 **常规信息** - 系统信息页面

**顶部标题栏**:
- Lecoo 品牌 Logo（橙色）
- 三个按钮：最小化、关闭

**右侧状态区**（3个圆形仪表盘）:
- **CPU温度**: 50°C（环形进度条，青绿色）
- **磁盘使用率**: 25% (246G/952G)
- **内存使用率**: 91% (8.0G/8.8G)

**中央主区域**:

1. **性能模式选择**（顶部3个按钮）:
   - 安静模式（灰色，未选中）
   - 均衡模式（灰色，未选中）
   - 性能模式（红色渐变，已选中）

2. **雷达图**（五边形）:
   - 温度（顶部）
   - 静音（右上）
   - 性能（右下）
   - 功耗（左下）
   - 续航（左上）
   - 填充颜色：青绿色渐变

3. **风扇转速仪表**（底部大圆形）:
   - 显示：2075 RPM
   - 环形动画条（青绿色脉冲效果）
   - 标签：风扇转速

---

### 风扇控制页面（截图 2）

**三种风扇模式**:

1. **自动模式**（A 图标）
   - 青绿色高亮（当前选中）
   - 系统自动调节风扇

2. **自定义模式**（扳手图标）
   - 灰色未选中
   - 允许手动调节

3. **最大模式**（三层风扇图标）
   - 灰色未选中
   - 风扇全速运行

**风扇转速显示**:
- 中央大圆形：2101 RPM
- 环形动画条（与主页一致）

**手动调节滑块**:
- 底部横向滑块
- 当前显示：0%
- 说明：可能只在"自定义"模式下可用

---

### 功能设定页面（截图 3）

**设置项**:
- **热键图标提示**
  - 开关按钮（ON 状态，青绿色圆形按钮）
  - 说明文字：
    > "打开功能，按下热键，会提示功能图标；关闭功能，按下热键，不会提示功能图标。"

**界面特点**:
- 简洁设计
- 只有一个主要设置项
- 符合前期测试结果（大部分功能不支持）

---

### 常规信息页面（截图 4）

**系统信息显示**（左右两列布局）:

| 项目 | 值 |
|-----|-----|
| **系统** | Microsoft Windows 10 IoT 企业版 LTSC |
| **处理器** | AMD Ryzen 7 8745H w/ Radeon 780M Graphics<br>3801 MHz, 8 Core(s), 16 Logical Processor(s) |
| **主板** | MINI PRO |
| **BIOS版本/日期** | American Megatrends International, LLC<br>MINI PRO-AHP 1.09, 2024/11/12 |
| **显示器** | 3456x2168 200% |
| **OSD版本** | 2.4 |
| **磁盘信息** | 使用率: 38%, 可用/共享: 295 G/476 G |
| **内存信息** | 使用率: 91%, 已用/共享: 8.0 G/8.8 G |

---

## 🎨 设计语言总结

### 配色方案

| 元素 | 颜色 | 用途 |
|-----|------|------|
| **主题色** | 青绿色 (#00CCA3 系列) | 高亮、选中状态、进度条 |
| **强调色** | 橙色 (#FF9800 系列) | Logo、品牌元素 |
| **警告色** | 红色渐变 | 性能模式按钮选中状态 |
| **背景色** | 深黑色 (#0A0A0A) + 碳纤维纹理 | 主背景 |
| **卡片背景** | 半透明深灰 (#1A1A1A 80%) | 按钮、区域背景 |
| **文字主色** | 白色 (#FFFFFF) | 主要文字 |
| **文字副色** | 浅灰色 (#CCCCCC) | 辅助信息 |

### 组件样式

**按钮**:
- 圆角矩形（约 8px 圆角）
- 未选中：深灰背景 + 灰色文字
- 选中：渐变背景（青绿色/红色）+ 白色文字
- Hover 效果：轻微发光

**圆形仪表盘**:
- 外圈：环形进度条（青绿色，动态宽度）
- 内圈：深色背景 + 中心数值
- 数值：大号白色 + 单位（小号灰色）
- 下方标签：灰色文字

**风扇转速仪表**:
- 中央：数值 + "RPM"
- 外圈：脉冲式动画条（类似声波扩散）
- 配色：青绿色渐变

**雷达图**:
- 五边形轮廓（灰色细线）
- 填充区域：青绿色渐变（半透明）
- 顶点标签：白色文字

### 动画效果

观察到的动画:
1. ✨ 风扇转速环形动画（脉冲扩散效果）
2. ✨ 按钮 hover 发光
3. ✨ 模式切换时的渐变过渡
4. ✨ 数值变化时的平滑过渡

---

## 💡 对 Rust 实现的指导

### 1. egui 主题配置

```rust
// src/ui/theme.rs
pub fn apply_lecoo_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // 配色
    let primary = egui::Color32::from_rgb(0, 204, 163);      // 青绿色
    let accent = egui::Color32::from_rgb(255, 152, 0);       // 橙色
    let bg_dark = egui::Color32::from_rgb(10, 10, 10);       // 深黑
    let bg_card = egui::Color32::from_rgba_premultiplied(26, 26, 26, 200);
    
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::WHITE);
    style.visuals.window_fill = bg_dark;
    style.visuals.panel_fill = bg_dark;
    style.visuals.widgets.inactive.bg_fill = bg_card;
    style.visuals.widgets.active.bg_fill = primary;
    
    // 圆角
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    
    ctx.set_style(style);
}
```

### 2. 布局结构

```rust
// src/ui/app.rs
impl eframe::App for ControlCenterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(150.0)
            .show(ctx, |ui| {
                // 4个导航按钮（垂直布局）
                self.render_nav_button(ui, "模式设置", Page::Mode);
                self.render_nav_button(ui, "风扇控制", Page::Fan);
                self.render_nav_button(ui, "功能设定", Page::Settings);
                self.render_nav_button(ui, "常规信息", Page::Info);
            });
        
        egui::SidePanel::right("status")
            .resizable(false)
            .exact_width(200.0)
            .show(ctx, |ui| {
                // 3个圆形仪表盘（垂直排列）
                self.render_gauge(ui, "CPU温度", self.cpu_temp, "°C");
                self.render_gauge(ui, "磁盘使用率", self.disk_usage, "%");
                self.render_gauge(ui, "内存使用率", self.mem_usage, "%");
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Mode => self.render_mode_page(ui),
                Page::Fan => self.render_fan_page(ui),
                Page::Settings => self.render_settings_page(ui),
                Page::Info => self.render_info_page(ui),
            }
        });
    }
}
```

### 3. 圆形仪表盘实现

```rust
// src/ui/widgets/gauge.rs
pub fn circular_gauge(
    ui: &mut egui::Ui,
    label: &str,
    value: f32,
    max: f32,
    unit: &str,
) {
    let size = egui::vec2(150.0, 150.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();
        let radius = rect.width().min(rect.height()) / 2.0 - 10.0;
        let stroke_width = 8.0;
        
        // 背景圆环（暗灰）
        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(stroke_width, egui::Color32::from_gray(40)),
        );
        
        // 进度圆环（青绿色）
        let progress = value / max;
        let arc_angle = progress * std::f32::consts::TAU;
        
        painter.add(egui::epaint::PathShape::convex_polygon(
            arc_points(center, radius, -std::f32::consts::FRAC_PI_2, arc_angle),
            egui::Color32::from_rgb(0, 204, 163),
            egui::Stroke::new(stroke_width, egui::Color32::from_rgb(0, 204, 163)),
        ));
        
        // 中心文字
        let text = format!("{:.0}{}", value, unit);
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(32.0),
            egui::Color32::WHITE,
        );
        
        // 底部标签
        let label_pos = center + egui::vec2(0.0, radius + 20.0);
        painter.text(
            label_pos,
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(14.0),
            egui::Color32::GRAY,
        );
    }
}
```

### 4. 风扇转速脉冲动画

```rust
// src/ui/widgets/fan_gauge.rs
pub struct FanGauge {
    rpm: u32,
    animation_phase: f32,  // 0.0 - 1.0
}

impl FanGauge {
    pub fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 更新动画
        self.animation_phase += 0.02;
        if self.animation_phase > 1.0 {
            self.animation_phase = 0.0;
        }
        ctx.request_repaint();  // 持续重绘
        
        let size = egui::vec2(250.0, 250.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            
            // 绘制多层脉冲圆环
            for i in 0..5 {
                let phase_offset = i as f32 * 0.2;
                let phase = (self.animation_phase + phase_offset) % 1.0;
                let radius = 80.0 + phase * 40.0;
                let alpha = ((1.0 - phase) * 255.0) as u8;
                
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(
                        4.0,
                        egui::Color32::from_rgba_premultiplied(0, 204, 163, alpha),
                    ),
                );
            }
            
            // 中心 RPM 数值
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{}", self.rpm),
                egui::FontId::proportional(48.0),
                egui::Color32::WHITE,
            );
            
            painter.text(
                center + egui::vec2(0.0, 30.0),
                egui::Align2::CENTER_TOP,
                "RPM",
                egui::FontId::proportional(16.0),
                egui::Color32::GRAY,
            );
        }
    }
}
```

### 5. 雷达图实现

```rust
// src/ui/widgets/radar_chart.rs
pub fn radar_chart(ui: &mut egui::Ui, values: &[f32; 5]) {
    let labels = ["温度", "静音", "性能", "功耗", "续航"];
    let size = egui::vec2(300.0, 300.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();
        let max_radius = rect.width().min(rect.height()) / 2.0 - 40.0;
        
        // 绘制五边形背景网格
        for scale in &[0.25, 0.5, 0.75, 1.0] {
            let points = pentagon_points(center, max_radius * scale);
            painter.add(egui::epaint::Shape::closed_line(
                points,
                egui::Stroke::new(1.0, egui::Color32::from_gray(40)),
            ));
        }
        
        // 绘制数据区域（填充）
        let data_points: Vec<egui::Pos2> = values.iter().enumerate().map(|(i, &v)| {
            let angle = i as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
            let r = max_radius * v;
            center + egui::vec2(r * angle.cos(), r * angle.sin())
        }).collect();
        
        painter.add(egui::epaint::Shape::convex_polygon(
            data_points,
            egui::Color32::from_rgba_premultiplied(0, 204, 163, 80),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 204, 163)),
        ));
        
        // 绘制标签
        for (i, label) in labels.iter().enumerate() {
            let angle = i as f32 * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
            let label_pos = center + egui::vec2(
                (max_radius + 25.0) * angle.cos(),
                (max_radius + 25.0) * angle.sin(),
            );
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
        }
    }
}
```

---

## 📊 功能映射

| UI 元素 | WMI 接口 | 实现优先级 |
|--------|---------|----------|
| 性能模式按钮 | GetPowerMode / SetPowerMode | P0 (MVP) |
| 风扇转速显示 | GetFanControl(1) | P0 (MVP) |
| CPU 温度 | GetHwTemp(1) | P0 (MVP) |
| 磁盘/内存使用率 | sysinfo crate | P0 (MVP) |
| 风扇模式选择 | 未测试（SetFanControl） | P1 (Phase 2) |
| 热键图标提示 | GetFeatureValue(1) / SetFeatureValue | P2 (Phase 3) |
| 雷达图 | 综合计算指标 | P1 (Phase 2) |
| 系统信息 | sysinfo + WMI | P1 (Phase 2) |

---

## ✅ 新增设计决策

基于官方 UI 分析，建议调整：

1. **窗口大小**: 
   - 原计划：800x600
   - 建议：1000x700（容纳左右侧边栏）

2. **导航结构**:
   - 采用左侧垂直导航（官方风格）
   - 右侧固定状态面板

3. **配色方案**:
   - 主题色：青绿色 #00CCA3（而非默认 egui 蓝色）
   - 强调色：橙色 #FF9800
   - 背景：深黑 + 碳纤维纹理

4. **动画优先级**:
   - P0: 风扇转速脉冲动画（视觉焦点）
   - P1: 按钮 hover 效果
   - P2: 模式切换过渡

5. **字体**:
   - 数值：48px（风扇 RPM）、32px（仪表盘）
   - 标签：14-16px
   - 建议使用 Microsoft YaHei UI 或 Noto Sans CJK

---

## 🎯 更新实现步骤

### Phase 1: MVP（基于官方 UI）

1. ✅ 三栏布局（左导航 + 中内容 + 右状态）
2. ✅ 性能模式切换（3 个按钮）
3. ✅ 风扇转速仪表盘（带脉冲动画）
4. ✅ 右侧 3 个圆形仪表盘
5. ✅ 应用 Lecoo 配色主题

### Phase 2: 完整功能

6. ⬜ 风扇控制页面（3 种模式 + 滑块）
7. ⬜ 雷达图（5 项指标）
8. ⬜ 功能设定页面
9. ⬜ 常规信息页面

### Phase 3: 抛光

10. ⬜ 碳纤维纹理背景
11. ⬜ 完整动画效果
12. ⬜ 图标资源

---

**最后更新**: 2026-10-02  
**参考来源**: 官方控制中心 v2.4 截图
