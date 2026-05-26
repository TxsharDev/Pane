//! Memory detail panel - RAM + swap with charts.

use eframe::egui;
use crate::app::App;
use crate::gui::{theme, widgets};

pub fn draw(ui: &mut egui::Ui, app: &App) {
    let mem_pct = if app.memory.total_bytes > 0 {
        (app.memory.used_bytes as f64 / app.memory.total_bytes as f64) * 100.0
    } else { 0.0 };

    egui::ScrollArea::vertical().show(ui, |ui| {
        widgets::section_header(ui, "Physical Memory");

        widgets::stat_card(
            ui,
            &format!("{} / {}", widgets::format_bytes(app.memory.used_bytes), widgets::format_bytes(app.memory.total_bytes)),
            &format!("{:.1}%", mem_pct),
            Some(mem_pct),
            theme::usage_color(mem_pct),
        );

        ui.add_space(4.0);
        widgets::chart(ui, &app.memory.usage_history.data, theme::usage_color(mem_pct), 120.0, "Usage History", "%", Some(100.0));

        ui.add_space(16.0);
        widgets::section_header(ui, "Swap / Pagefile");

        let swap_pct = if app.memory.swap_total > 0 {
            (app.memory.swap_used as f64 / app.memory.swap_total as f64) * 100.0
        } else { 0.0 };

        widgets::stat_card(
            ui,
            &format!("{} / {}", widgets::format_bytes(app.memory.swap_used), widgets::format_bytes(app.memory.swap_total)),
            &format!("{:.1}%", swap_pct),
            Some(swap_pct),
            theme::usage_color(swap_pct),
        );
    });
}
