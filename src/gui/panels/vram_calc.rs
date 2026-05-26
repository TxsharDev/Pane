//! VRAM Headroom Calculator - shows what models fit in available VRAM.
//!
//! Estimates based on common quantization sizes and context length overhead.
//! This is what r/LocalLLaMA users desperately want in a monitoring tool.

use eframe::egui;
use crate::app::App;
use crate::gui::{theme, widgets};

/// Known model sizes at various quantization levels (in GB).
/// Format: (name, param_count, q4_gb, q5_gb, q8_gb, fp16_gb)
const MODELS: &[(&str, &str, f64, f64, f64, f64)] = &[
    ("Llama 3.1 8B",    "8B",   4.7,   5.5,   8.5,   16.0),
    ("Mistral 7B",      "7B",   4.4,   5.1,   7.7,   14.5),
    ("Qwen 2.5 14B",    "14B",  8.2,   9.8,   14.8,  28.0),
    ("Llama 3.1 70B",   "70B",  40.0,  48.0,  72.0,  140.0),
    ("Qwen 2.5 72B",    "72B",  41.0,  49.0,  74.0,  144.0),
    ("Mixtral 8x7B",    "47B",  26.0,  31.0,  48.0,  94.0),
    ("Llama 3.1 405B",  "405B", 228.0, 274.0, 415.0, 810.0),
    ("DeepSeek V3",     "671B", 377.0, 453.0, 688.0, 1342.0),
    ("Qwen3 Coder 80B", "80B",  45.0,  54.0,  82.0,  160.0),
];

/// Estimate KV cache size in GB for a given context length and model size.
fn kv_cache_gb(context_len: usize, num_layers: usize, head_dim: usize, num_kv_heads: usize) -> f64 {
    // KV cache = 2 * layers * kv_heads * head_dim * context * 2bytes (FP16)
    let bytes = 2.0 * num_layers as f64 * num_kv_heads as f64 * head_dim as f64 * context_len as f64 * 2.0;
    bytes / (1024.0 * 1024.0 * 1024.0)
}

/// Rough KV cache estimate based on param count string.
fn estimate_kv_cache(params: &str, context: usize) -> f64 {
    // Rough heuristic: bigger models have more layers/heads
    match params {
        "7B" | "8B" => kv_cache_gb(context, 32, 128, 8),
        "14B" => kv_cache_gb(context, 40, 128, 8),
        "47B" => kv_cache_gb(context, 32, 128, 8), // MoE
        "70B" | "72B" | "80B" => kv_cache_gb(context, 80, 128, 8),
        "405B" => kv_cache_gb(context, 126, 128, 8),
        "671B" => kv_cache_gb(context, 61, 128, 8), // MoE, fewer layers
        _ => 0.5, // fallback
    }
}

pub fn draw(ui: &mut egui::Ui, app: &App) {
    let p = theme::p();

    egui::ScrollArea::vertical().show(ui, |ui| {
        widgets::section_header(ui, "VRAM Headroom Calculator");

        // Show available VRAM per GPU
        let mut total_free: u64 = 0;
        for (i, gpu) in app.gpus.iter().enumerate() {
            let free = gpu.vram_total.saturating_sub(gpu.vram_used);
            total_free += free;
            ui.horizontal(|ui| {
                let short = gpu.name.replace("NVIDIA GeForce ", "");
                ui.label(egui::RichText::new(format!("GPU {}: {}", i, short)).size(12.0).color(p.text));
                ui.label(egui::RichText::new(format!(
                    "{} free / {} total",
                    widgets::format_bytes(free),
                    widgets::format_bytes(gpu.vram_total)
                )).size(12.0).color(if free > gpu.vram_total / 4 { p.green } else { p.yellow }));
            });
        }

        if app.gpus.len() > 1 {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Combined free:").size(12.0).color(p.dim));
                ui.label(egui::RichText::new(widgets::format_bytes(total_free)).size(14.0).color(p.accent).strong());
            });
        }

        ui.add_space(12.0);
        widgets::section_header(ui, "What fits?");
        ui.label(egui::RichText::new("Based on current free VRAM (single GPU: largest card)").size(10.0).color(p.dim));
        ui.add_space(4.0);

        let largest_free = app.gpus.iter().map(|g| g.vram_total.saturating_sub(g.vram_used)).max().unwrap_or(0);
        let largest_free_gb = largest_free as f64 / (1024.0 * 1024.0 * 1024.0);
        let combined_free_gb = total_free as f64 / (1024.0 * 1024.0 * 1024.0);

        // Model table
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::exact(140.0))  // Model
            .column(egui_extras::Column::exact(55.0))   // Q4
            .column(egui_extras::Column::exact(55.0))   // Q5
            .column(egui_extras::Column::exact(55.0))   // Q8
            .column(egui_extras::Column::exact(55.0))   // FP16
            .column(egui_extras::Column::remainder())    // Verdict
            .header(22.0, |mut header| {
                header.col(|ui| { ui.label(egui::RichText::new("Model").size(11.0).color(p.accent).strong()); });
                header.col(|ui| { ui.label(egui::RichText::new("Q4").size(11.0).color(p.accent).strong()); });
                header.col(|ui| { ui.label(egui::RichText::new("Q5").size(11.0).color(p.accent).strong()); });
                header.col(|ui| { ui.label(egui::RichText::new("Q8").size(11.0).color(p.accent).strong()); });
                header.col(|ui| { ui.label(egui::RichText::new("FP16").size(11.0).color(p.accent).strong()); });
                header.col(|ui| { ui.label(egui::RichText::new("Status").size(11.0).color(p.accent).strong()); });
            })
            .body(|body| {
                body.rows(22.0, MODELS.len(), |mut row| {
                    let (name, params, q4, q5, q8, fp16) = MODELS[row.index()];
                    let kv_4k = estimate_kv_cache(params, 4096);

                    row.col(|ui| { ui.label(egui::RichText::new(name).size(11.0)); });

                    // Q4
                    row.col(|ui| {
                        let fits_single = q4 + kv_4k < largest_free_gb;
                        let color = if fits_single { p.green } else { p.red };
                        ui.label(egui::RichText::new(format!("{:.0}G", q4)).size(11.0).color(color));
                    });
                    // Q5
                    row.col(|ui| {
                        let fits = q5 + kv_4k < largest_free_gb;
                        ui.label(egui::RichText::new(format!("{:.0}G", q5)).size(11.0).color(if fits { p.green } else { p.red }));
                    });
                    // Q8
                    row.col(|ui| {
                        let fits = q8 + kv_4k < largest_free_gb;
                        ui.label(egui::RichText::new(format!("{:.0}G", q8)).size(11.0).color(if fits { p.green } else { p.red }));
                    });
                    // FP16
                    row.col(|ui| {
                        let fits = fp16 + kv_4k < largest_free_gb;
                        ui.label(egui::RichText::new(format!("{:.0}G", fp16)).size(11.0).color(if fits { p.green } else { p.red }));
                    });

                    // Verdict
                    row.col(|ui| {
                        let best_fit = if q4 + kv_4k < largest_free_gb {
                            let headroom = largest_free_gb - q4 - kv_4k;
                            let max_ctx = if headroom > 4.0 { "32k+" } else if headroom > 1.0 { "8k" } else { "4k" };
                            format!("Q4 fits ({}ctx)", max_ctx)
                        } else if q4 + kv_4k < combined_free_gb && app.gpus.len() > 1 {
                            "Q4 fits (split)".into()
                        } else {
                            "Too large".into()
                        };

                        let color = if best_fit.contains("fits") { p.green } else { p.dim };
                        ui.label(egui::RichText::new(best_fit).size(10.0).color(color));
                    });
                });
            });

        ui.add_space(12.0);
        ui.label(egui::RichText::new("Sizes include model weights only. KV cache adds ~0.1-2GB depending on context length. Actual usage varies by runtime.").size(10.0).color(p.dim));
    });
}
