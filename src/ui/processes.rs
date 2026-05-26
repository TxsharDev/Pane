//! Process table - sortable, filterable, with GPU columns and kill support.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::app::{App, SortColumn};
use super::{format_bytes, ACCENT, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let sort_marker = |col: SortColumn| -> &'static str {
        if app.sort_column == col {
            if app.sort_ascending { " \u{25b2}" } else { " \u{25bc}" }
        } else { "" }
    };

    let header = Row::new(vec![
        Cell::from(format!("PID{}", sort_marker(SortColumn::Pid)))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from(format!("Name{}", sort_marker(SortColumn::Name)))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from(format!("CPU%{}", sort_marker(SortColumn::Cpu)))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from(format!("Memory{}", sort_marker(SortColumn::Memory)))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from(format!("GPU%{}", sort_marker(SortColumn::GpuUtil)))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from(format!("VRAM{}", sort_marker(SortColumn::GpuVram)))
            .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let procs = app.sorted_processes();
    let visible_height = area.height.saturating_sub(4) as usize;
    let start = app.process_scroll.min(procs.len().saturating_sub(1));
    let end = (start + visible_height).min(procs.len());

    let rows: Vec<Row> = procs[start..end]
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = (start + i) == app.process_selected;
            let is_kill_target = app.confirm_kill == Some(p.pid);

            let row_style = if is_kill_target {
                Style::default().fg(Color::White).bg(Color::Red)
            } else if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cpu_color = if p.cpu_usage > 80.0 { Color::Red }
                else if p.cpu_usage > 40.0 { Color::Yellow }
                else { Color::White };

            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(format!("{:.1}", p.cpu_usage)).style(Style::default().fg(cpu_color)),
                Cell::from(format_bytes(p.memory_bytes)),
                Cell::from(
                    p.gpu_util.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "-".to_string()),
                ).style(Style::default().fg(Color::Cyan)),
                Cell::from(
                    p.gpu_vram.map(format_bytes).unwrap_or_else(|| "-".to_string()),
                ).style(Style::default().fg(Color::Cyan)),
            ]).style(row_style)
        })
        .collect();

    let title = if let Some(pid) = app.confirm_kill {
        format!(" Kill PID {}? (y/n) ", pid)
    } else if app.filter.is_empty() {
        format!(" Processes ({}) ", app.processes.len())
    } else {
        format!(" Processes \u{2014} filter: \"{}\" ({} matched) ", app.filter, procs.len())
    };

    let table = Table::new(rows, [
        Constraint::Length(8), Constraint::Percentage(30),
        Constraint::Length(8), Constraint::Length(12),
        Constraint::Length(8), Constraint::Length(12),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title).border_style(BORDER_STYLE));

    f.render_widget(table, area);
}
