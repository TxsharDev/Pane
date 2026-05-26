//! GPU detail panel - deep metrics for the selected GPU.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

use crate::app::App;
use super::{format_bytes, format_bytes_rate, usage_color, ACCENT, DIM, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.gpus.is_empty() {
        let msg = Paragraph::new("No GPU detected. Install NVIDIA drivers with NVML support.")
            .style(Style::default().fg(DIM))
            .block(Block::default().borders(Borders::ALL).title(" GPU ").border_style(BORDER_STYLE));
        f.render_widget(msg, area);
        return;
    }

    let gpu = &app.gpus[app.selected_gpu.min(app.gpus.len() - 1)];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // utilization sparkline
            Constraint::Min(0),    // details grid
        ])
        .split(area);

    // Utilization sparkline
    let spark_data: Vec<u64> = gpu.utilization_history.data.iter().map(|v| *v as u64).collect();
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} \u{2014} {:.0}% ", gpu.name, gpu.utilization))
                .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
                .border_style(BORDER_STYLE),
        )
        .data(&spark_data)
        .max(100)
        .style(Style::default().fg(usage_color(gpu.utilization)));
    f.render_widget(sparkline, chunks[0]);

    // Details grid
    let detail_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left: VRAM, Power, Clocks
    let vram_pct = gpu.vram_pct();

    let mut left_lines = vec![
        Line::from(vec![
            Span::styled("VRAM     ", Style::default().fg(DIM)),
            Span::styled(
                format!("{} / {}  ({:.0}%)", format_bytes(gpu.vram_used), format_bytes(gpu.vram_total), vram_pct),
                Style::default().fg(usage_color(vram_pct)),
            ),
        ]),
    ];

    if let Some(watts) = gpu.power_watts {
        let limit_str = gpu.power_limit.map(|l| format!(" / {:.0}W", l)).unwrap_or_default();
        left_lines.push(Line::from(vec![
            Span::styled("Power    ", Style::default().fg(DIM)),
            Span::styled(format!("{:.0}W{}", watts, limit_str), Style::default().fg(Color::White)),
        ]));
    }

    if let Some(core) = gpu.clock_core_mhz {
        let mem_str = gpu.clock_mem_mhz.map(|m| format!(" / {} MHz mem", m)).unwrap_or_default();
        left_lines.push(Line::from(vec![
            Span::styled("Clocks   ", Style::default().fg(DIM)),
            Span::styled(format!("{} MHz core{}", core, mem_str), Style::default().fg(Color::White)),
        ]));
    }

    let left = Paragraph::new(left_lines)
        .block(Block::default().borders(Borders::ALL).title(" Hardware ").border_style(BORDER_STYLE));
    f.render_widget(left, detail_chunks[0]);

    // Right: Temps, Fan, PCIe
    let mut right_lines = Vec::new();

    if let Some(temp) = gpu.temp_core {
        let temp_color = if temp > 85 { Color::Red } else if temp > 70 { Color::Yellow } else { Color::Green };
        let mut temp_str = format!("{}C core", temp);
        if let Some(hs) = gpu.temp_hotspot {
            temp_str.push_str(&format!(" / {}C hotspot", hs));
        }
        right_lines.push(Line::from(vec![
            Span::styled("Temp     ", Style::default().fg(DIM)),
            Span::styled(temp_str, Style::default().fg(temp_color)),
        ]));
    }

    if let Some(fan) = gpu.fan_rpm {
        right_lines.push(Line::from(vec![
            Span::styled("Fan      ", Style::default().fg(DIM)),
            Span::styled(format!("{}%", fan), Style::default().fg(Color::White)),
        ]));
    }

    if let (Some(tx), Some(rx)) = (gpu.pcie_tx_bytes_sec, gpu.pcie_rx_bytes_sec) {
        right_lines.push(Line::from(vec![
            Span::styled("PCIe TX  ", Style::default().fg(DIM)),
            Span::styled(format_bytes_rate(tx), Style::default().fg(Color::Cyan)),
        ]));
        right_lines.push(Line::from(vec![
            Span::styled("PCIe RX  ", Style::default().fg(DIM)),
            Span::styled(format_bytes_rate(rx), Style::default().fg(Color::Cyan)),
        ]));
    }

    let right = Paragraph::new(right_lines)
        .block(Block::default().borders(Borders::ALL).title(" Thermals & IO ").border_style(BORDER_STYLE));
    f.render_widget(right, detail_chunks[1]);
}
