//! Isolated allocation experiments. No hardware control is performed.
use anyhow::{bail, Result};
use eframe::egui;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use windows::Win32::System::{
    ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
    Threading::GetCurrentProcess,
};

fn sample(stage: &str) -> Result<()> {
    let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
    counters.cb = std::mem::size_of_val(&counters) as u32;
    unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
            counters.cb,
        )?;
    }
    println!(
        "MEM stage={stage} working_set_mib={:.3} private_commit_mib={:.3}",
        counters.WorkingSetSize as f64 / 1048576.0,
        counters.PrivateUsage as f64 / 1048576.0
    );
    Ok(())
}

struct GraphicsProbe {
    start: Instant,
    sampled: bool,
}
impl eframe::App for GraphicsProbe {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("OpenGL / egui baseline. No Chinese font or system scan.");
        });
        ctx.request_repaint_after(Duration::from_millis(200));
        if !self.sampled && self.start.elapsed() >= Duration::from_secs(5) {
            self.sampled = true;
            let _ = sample("graphics_idle");
        }
        if self.start.elapsed() >= Duration::from_secs(6) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn run() -> Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_default();
    sample("entry")?;
    match mode.as_str() {
        "system-full" | "system-memory" => {
            let mut system = if mode == "system-full" {
                sysinfo::System::new_all()
            } else {
                sysinfo::System::new()
            };
            system.refresh_memory();
            let disks = sysinfo::Disks::new_with_refreshed_list();
            println!(
                "SYSTEM processes={} cpus={} disks={} memory_total={}",
                system.processes().len(),
                system.cpus().len(),
                disks.list().len(),
                system.total_memory()
            );
            sample("system_ready")?;
            std::thread::sleep(Duration::from_secs(2));
            sample("system_idle")?;
            drop((system, disks));
            sample("system_dropped")?;
        }
        "font-owned" | "font-borrowed" => {
            let ctx = egui::Context::default();
            sample("context")?;
            let bytes = std::fs::read("C:/Windows/Fonts/msyh.ttc")?;
            println!("FONT bytes={}", bytes.len());
            sample("font_read")?;
            let mut fonts = egui::FontDefinitions::default();
            let data = if mode == "font-owned" {
                egui::FontData::from_owned(bytes)
            } else {
                // Experiment only: process-lifetime immutable storage avoids FontVec's full copy.
                egui::FontData::from_static(Box::leak(bytes.into_boxed_slice()))
            };
            fonts.font_data.insert("yahei".into(), Arc::new(data));
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .get_mut(&family)
                    .unwrap()
                    .insert(0, "yahei".into());
            }
            ctx.set_fonts(fonts);
            sample("font_assigned")?;
            let frame = ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 600.0))), ..Default::default() }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| { ui.heading("Lecoo CPU温度 风扇控制"); ui.label("模式设置 安静模式 均衡模式 性能模式 功能设定 常规信息 自动 最大 手动目标 不可用"); });
            });
            println!(
                "ATLAS size={:?}",
                ctx.fonts(|fonts| fonts.font_image_size())
            );
            sample("font_rendered")?;
            std::thread::sleep(Duration::from_secs(2));
            sample("font_idle")?;
            drop((frame, ctx));
            sample("font_dropped")?;
        }
        "graphics" => {
            eframe::run_native(
                "Memory baseline",
                eframe::NativeOptions {
                    viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
                    ..Default::default()
                },
                Box::new(|_| {
                    let _ = sample("graphics_context");
                    Ok(Box::new(GraphicsProbe {
                        start: Instant::now(),
                        sampled: false,
                    }))
                }),
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            sample("graphics_closed")?;
        }
        _ => {
            bail!("Usage: memory_probe system-full|system-memory|font-owned|font-borrowed|graphics")
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
