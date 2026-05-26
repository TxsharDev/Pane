//! Network detail panel - per-interface throughput and totals.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::app::App;
use super::{format_bytes, format_bytes_rate, ACCENT, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Interface").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("\u{2193} RX/s").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("\u{2191} TX/s").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Total RX").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Total TX").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app.networks.iter().map(|n| {
        Row::new(vec![
            Cell::from(n.name.clone()),
            Cell::from(format_bytes_rate(n.rx_bytes_sec)).style(Style::default().fg(Color::Green)),
            Cell::from(format_bytes_rate(n.tx_bytes_sec)).style(Style::default().fg(Color::Magenta)),
            Cell::from(format_bytes(n.total_rx)),
            Cell::from(format_bytes(n.total_tx)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(30), Constraint::Percentage(17),
        Constraint::Percentage(17), Constraint::Percentage(18),
        Constraint::Percentage(18),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Network ").border_style(BORDER_STYLE));

    f.render_widget(table, area);
}
