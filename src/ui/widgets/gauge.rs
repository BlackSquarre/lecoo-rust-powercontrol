use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::f32::consts::PI;

pub fn circular_gauge(
    ui: &mut egui::Ui,
    label: &str,
    value: f32,
    max: f32,
    unit: &str,
    color: Color32,
) {
    let size = Vec2::new(150.0, 150.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();
        let radius = rect.width().min(rect.height()) / 2.0 - 10.0;
        let stroke_width = 8.0;

        // 背景圆环（暗灰）
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(stroke_width, Color32::from_gray(40)),
        );

        // 进度圆环
        let progress = (value / max).clamp(0.0, 1.0);
        if progress > 0.0 {
            draw_arc(
                painter,
                center,
                radius,
                stroke_width,
                -PI / 2.0,  // 从顶部开始
                -PI / 2.0 + progress * 2.0 * PI,  // 顺时针
                color,
            );
        }

        // 中心数值
        let value_text = format!("{:.0}", value);
        painter.text(
            center - Vec2::new(0.0, 5.0),
            egui::Align2::CENTER_CENTER,
            value_text,
            egui::FontId::proportional(32.0),
            Color32::WHITE,
        );

        // 单位
        painter.text(
            center + Vec2::new(0.0, 20.0),
            egui::Align2::CENTER_CENTER,
            unit,
            egui::FontId::proportional(14.0),
            Color32::from_gray(180),
        );

        // 底部标签
        painter.text(
            center + Vec2::new(0.0, radius + 20.0),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(14.0),
            Color32::from_gray(200),
        );
    }
}

fn draw_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    width: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color32,
) {
    let segments = 32;
    let angle_range = end_angle - start_angle;

    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let angle1 = start_angle + t1 * angle_range;
        let angle2 = start_angle + t2 * angle_range;

        let p1 = center + Vec2::new(radius * angle1.cos(), radius * angle1.sin());
        let p2 = center + Vec2::new(radius * angle2.cos(), radius * angle2.sin());

        painter.line_segment([p1, p2], Stroke::new(width, color));
    }
}

pub fn disk_gauge(ui: &mut egui::Ui, used_gb: f32, total_gb: f32, color: Color32) {
    let usage = if total_gb > 0.0 {
        (used_gb / total_gb * 100.0).min(100.0)
    } else {
        0.0
    };

    let label = format!("{}G/{}G", used_gb as u64, total_gb as u64);

    circular_gauge(ui, "磁盘使用率", usage, 100.0, "%", color);

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(label)
            .size(12.0)
            .color(Color32::from_gray(160))
    );
}

pub fn memory_gauge(ui: &mut egui::Ui, used_gb: f32, total_gb: f32, color: Color32) {
    let usage = if total_gb > 0.0 {
        (used_gb / total_gb * 100.0).min(100.0)
    } else {
        0.0
    };

    let label = format!("{:.1}G/{:.1}G", used_gb, total_gb);

    circular_gauge(ui, "内存使用率", usage, 100.0, "%", color);

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(label)
            .size(12.0)
            .color(Color32::from_gray(160))
    );
}
