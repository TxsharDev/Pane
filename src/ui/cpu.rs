//! CPU detail panel - total usage sparkline + per-core bar grid.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

use crate::app::App;
use super::{usage_color, ACCENT, DIM, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    // Total CPU sparkline
    let spark_data: Vec<u64> = app.cpu.total_history.data.iter().map(|v| *v as u64).collect();
    let title = format!(
        " {} \u{2014} {:.0}% ({}/{} cores) ",
        app.cpu.name, app.cpu.total_usage, app.cpu.physical_cores, app.cpu.logical_cores
    );
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
                .border_style(BORDER_STYLE),
        )
        .data(&spark_data)
        .max(100)
        .style(Style::default().fg(usage_color(app.cpu.total_usage)));
    f.render_widget(sparkline, chunks[0]);

    // Per-core bars
    if app.cpu.cores.is_empty() {
        return;
    }

    let cols = 4.min(app.cpu.cores.len());
    let rows = app.cpu.cores.len().div_ceil(cols);

    let row_constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(1)).collect();
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(chunks[1]);

    let col_constraints: Vec<Constraint> = (0..cols).map(|_| Constraint::Ratio(1, cols as u32)).collect();

    for row in 0..rows {
        if row >= row_chunks.len() { break; }
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints.clone())
            .split(row_chunks[row]);

        for col in 0..cols {
            let idx = row * cols + col;
            if idx >= app.cpu.cores.len() || col >= col_chunks.len() { break; }
            let core = &app.cpu.cores[idx];
            let bar_len = 20;
            let filled = ((core.usage / 100.0) * bar_len as f64) as usize;
            let bar = format!(
                "{}{}",
                "\u{2588}".repeat(filled),
                "\u{2591}".repeat(bar_len - filled)
            );

            let line = Line::from(vec![
                Span::styled(format!("C{:<2} ", idx), Style::default().fg(DIM)),
                Span::styled(bar, Style::default().fg(usage_color(core.usage))),
                Span::styled(format!(" {:>5.1}%", core.usage), Style::default().fg(Color::White)),
                Span::styled(format!(" {}MHz", core.freq_mhz), Style::default().fg(DIM)),
            ]);

            f.render_widget(Paragraph::new(line), col_chunks[col]);
        }
    }
}
