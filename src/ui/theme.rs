use egui::Color32;
use std::sync::Arc;

/// Lecoo 品牌配色
pub struct LecooColors {
    pub primary: Color32,        // 青绿色
    pub accent: Color32,         // 橙色
    pub bg_dark: Color32,        // 深黑背景
    pub bg_card: Color32,        // 卡片背景
    pub text_primary: Color32,   // 主文字
    pub text_secondary: Color32, // 副文字
    pub mode_quiet: Color32,     // 安静模式渐变起点
    pub mode_balance: Color32,   // 均衡模式
    pub mode_performance: Color32, // 性能模式渐变起点
}

impl Default for LecooColors {
    fn default() -> Self {
        Self {
            primary: Color32::from_rgb(0, 204, 163),      // #00CCA3
            accent: Color32::from_rgb(255, 152, 0),       // #FF9800
            bg_dark: Color32::from_rgb(10, 10, 10),       // #0A0A0A
            bg_card: Color32::from_rgba_premultiplied(26, 26, 26, 200),
            text_primary: Color32::WHITE,
            text_secondary: Color32::from_rgb(204, 204, 204),
            mode_quiet: Color32::from_rgb(100, 200, 255),
            mode_balance: Color32::from_rgb(100, 200, 100),
            mode_performance: Color32::from_rgb(255, 80, 80),
        }
    }
}

pub fn apply_lecoo_theme(ctx: &egui::Context) {
    // 加载中文字体
    let mut fonts = egui::FontDefinitions::default();

    // 尝试从 Windows 系统加载微软雅黑字体
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        fonts.font_data.insert(
            "microsoft_yahei".to_owned(),
            Arc::new(egui::FontData::from_owned(font_data)),
        );

        // 将微软雅黑设置为默认字体
        fonts.families.get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "microsoft_yahei".to_owned());

        fonts.families.get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "microsoft_yahei".to_owned());

        ctx.set_fonts(fonts);
    }

    let colors = LecooColors::default();
    let mut style = (*ctx.style()).clone();

    // 基础设置
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(colors.text_primary);
    style.visuals.window_fill = colors.bg_dark;
    style.visuals.panel_fill = colors.bg_dark;

    // 组件样式
    style.visuals.widgets.noninteractive.bg_fill = colors.bg_card;
    style.visuals.widgets.inactive.bg_fill = colors.bg_card;
    style.visuals.widgets.hovered.bg_fill = colors.primary;
    style.visuals.widgets.active.bg_fill = colors.primary;

    // 圆角
    let rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.noninteractive.rounding = rounding;
    style.visuals.widgets.inactive.rounding = rounding;
    style.visuals.widgets.hovered.rounding = rounding;
    style.visuals.widgets.active.rounding = rounding;

    // 描边
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_gray(60));
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0_f32, colors.primary);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, colors.primary);

    // 间距
    style.spacing.item_spacing = egui::vec2(12.0, 12.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);

    ctx.set_style(style);
}
