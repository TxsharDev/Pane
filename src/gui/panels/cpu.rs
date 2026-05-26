//! CPU detail panel - total usage chart + per-core bars.

use eframe::egui;
use crate::app::App;
use crate::gui::{theme, widgets};

pub fn draw(ui: &mut egui::Ui, app: &App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let cpu_name = if app.cpu.name.len() > 45 {
            format!("{}...", &app.cpu.name[..42])
        } else {
            app.cpu.name.clone()
        };

        widgets::section_header(ui, &cpu_name);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{} physical / {} logical cores", app.cpu.physical_cores, app.cpu.logical_cores)).size(11.0).color(theme::p().dim));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format!("{:.1}%", app.cpu.total_usage)).size(18.0).color(theme::usage_color(app.cpu.total_usage)).strong());
            });
        });

        widgets::chart(ui, &app.cpu.total_history.data, theme::usage_color(app.cpu.total_usage), 100.0, "Total Usage", "%", Some(100.0));

        ui.add_space(12.0);
        widgets::section_header(ui, "Per Core");

        let cols = 4.min(app.cpu.cores.len()).max(1);
        let rows = app.cpu.cores.len().div_ceil(cols);

        for row in 0..rows {
            #[allow(clippy::needless_range_loop)]
            ui.columns(cols, |col_ui| {
                for col in 0..cols {
                    let idx = row * cols + col;
                    if idx >= app.cpu.cores.len() { continue; }
                    let core = &app.cpu.cores[idx];

                    col_ui[col].horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("C{}", idx)).size(10.0).color(theme::p().dim).monospace());
                        ui.label(egui::RichText::new(format!("{:.0}%", core.usage)).size(11.0).color(theme::usage_color(core.usage)).strong());
                        ui.label(egui::RichText::new(format!("{}MHz", core.freq_mhz)).size(9.0).color(theme::p().dim));
                    });
                    widgets::progress_bar(&mut col_ui[col], core.usage / 100.0, theme::usage_color(core.usage), 4.0);
                    col_ui[col].add_space(3.0);
                }
            });
        }
    });
}
