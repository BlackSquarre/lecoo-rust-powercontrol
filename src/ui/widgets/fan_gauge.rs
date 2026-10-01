use egui::{Color32, Pos2, Sense, Stroke, Vec2};
use std::f32::consts::PI;

pub struct FanGauge {
    rpm: u32,
    animation_phase: f32,  // 0.0 - 1.0
}

impl FanGauge {
    pub fn new() -> Self {
        Self {
            rpm: 0,
            animation_phase: 0.0,
        }
    }

    pub fn set_rpm(&mut self, rpm: u32) {
        self.rpm = rpm;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();

        // 更新动画
        if self.rpm > 0 {
            self.animation_phase += 0.02;
            if self.animation_phase > 1.0 {
                self.animation_phase = 0.0;
            }
            ctx.request_repaint();  // 持续重绘动画
        }

        let size = Vec2::new(250.0, 250.0);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let _color = Color32::from_rgb(0, 204, 163);

            // 绘制多层脉冲圆环（扩散效果）
            if self.rpm > 0 {
                for i in 0..6 {
                    let phase_offset = i as f32 * 0.15;
                    let phase = (self.animation_phase + phase_offset) % 1.0;
                    let radius = 70.0 + phase * 50.0;
                    let alpha = ((1.0 - phase) * 200.0) as u8;

                    // 绘制虚线圆环（模拟叶片）
                    let segments = 12;
                    for seg in 0..segments {
                        let angle1 = seg as f32 * 2.0 * PI / segments as f32;
                        let angle2 = angle1 + PI / segments as f32 * 0.7; // 虚线效果

                        let p1 = center + Vec2::new(radius * angle1.cos(), radius * angle1.sin());
                        let p2 = center + Vec2::new(radius * angle2.cos(), radius * angle2.sin());

                        painter.line_segment(
                            [p1, p2],
                            Stroke::new(
                                5.0,
                                Color32::from_rgba_premultiplied(0, 204, 163, alpha),
                            ),
                        );
                    }
                }
            }

            // 内圈静态圆
            painter.circle_stroke(
                center,
                60.0,
                Stroke::new(2.0, Color32::from_gray(60)),
            );

            // 中心 RPM 数值
            painter.text(
                center - Vec2::new(0.0, 10.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", self.rpm),
                egui::FontId::proportional(48.0),
                Color32::WHITE,
            );

            // RPM 标签
            painter.text(
                center + Vec2::new(0.0, 25.0),
                egui::Align2::CENTER_CENTER,
                "RPM",
                egui::FontId::proportional(16.0),
                Color32::from_gray(160),
            );

            // 底部标签
            painter.text(
                center + Vec2::new(0.0, 140.0),
                egui::Align2::CENTER_TOP,
                "风扇转速",
                egui::FontId::proportional(14.0),
                Color32::from_gray(200),
            );
        }
    }
}

impl Default for FanGauge {
    fn default() -> Self {
        Self::new()
    }
}
