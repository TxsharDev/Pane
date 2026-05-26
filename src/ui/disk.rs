//! Disk detail panel - per-disk usage, read/write throughput.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::app::App;
use super::{format_bytes, format_bytes_rate, ACCENT, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Disk").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Mount").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Used").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Total").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Read").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Write").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app.disks.iter().map(|d| {
        let pct = if d.total_bytes > 0 { (d.used_bytes as f64 / d.total_bytes as f64) * 100.0 } else { 0.0 };
        let color = if pct > 90.0 { Color::Red } else if pct > 70.0 { Color::Yellow } else { Color::White };

        Row::new(vec![
            Cell::from(d.name.clone()),
            Cell::from(d.mount.clone()),
            Cell::from(format!("{} ({:.0}%)", format_bytes(d.used_bytes), pct)).style(Style::default().fg(color)),
            Cell::from(format_bytes(d.total_bytes)),
            Cell::from(format_bytes_rate(d.read_bytes_sec)).style(Style::default().fg(Color::Green)),
            Cell::from(format_bytes_rate(d.write_bytes_sec)).style(Style::default().fg(Color::Magenta)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(15), Constraint::Percentage(15),
        Constraint::Percentage(20), Constraint::Percentage(15),
        Constraint::Percentage(17), Constraint::Percentage(18),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Disks ").border_style(BORDER_STYLE));

    f.render_widget(table, area);
}
