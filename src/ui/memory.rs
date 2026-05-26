//! Memory detail panel - RAM and swap gauges with usage history.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Sparkline};

use crate::app::App;
use super::{format_bytes, usage_color, ACCENT, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Usage sparkline
    let spark_data: Vec<u64> = app.memory.usage_history.data.iter().map(|v| *v as u64).collect();
    let usage_pct = if app.memory.total_bytes > 0 {
        (app.memory.used_bytes as f64 / app.memory.total_bytes as f64) * 100.0
    } else { 0.0 };

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Memory \u{2014} {:.0}% ", usage_pct))
                .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
                .border_style(BORDER_STYLE),
        )
        .data(&spark_data)
        .max(100)
        .style(Style::default().fg(usage_color(usage_pct)));
    f.render_widget(sparkline, chunks[0]);

    // RAM gauge
    let ram_label = format!("{} / {}", format_bytes(app.memory.used_bytes), format_bytes(app.memory.total_bytes));
    let ram_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" RAM ").border_style(BORDER_STYLE))
        .gauge_style(Style::default().fg(usage_color(usage_pct)))
        .ratio((usage_pct / 100.0).min(1.0))
        .label(ram_label);
    f.render_widget(ram_gauge, chunks[1]);

    // Swap gauge
    let swap_pct = if app.memory.swap_total > 0 {
        (app.memory.swap_used as f64 / app.memory.swap_total as f64) * 100.0
    } else { 0.0 };
    let swap_label = format!("{} / {}", format_bytes(app.memory.swap_used), format_bytes(app.memory.swap_total));
    let swap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Swap ").border_style(BORDER_STYLE))
        .gauge_style(Style::default().fg(usage_color(swap_pct)))
        .ratio((swap_pct / 100.0).min(1.0))
        .label(swap_label);
    f.render_widget(swap_gauge, chunks[2]);
}
