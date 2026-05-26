//! TUI rendering for Pane.
//!
//! Each panel is a separate module. The `draw` function dispatches to the
//! active panel. Shared helpers (colors, formatters, braille graphs) live here.

pub mod dashboard;
pub mod gpu;
pub mod gpu_control;
pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod processes;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Tabs};

use crate::app::{App, Panel};

// ── Theme ──────────────────────────────────────────────────────────────────

pub const ACCENT: Color = Color::Cyan;
pub const DIM: Color = Color::DarkGray;
pub const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);

// ── Braille sparkline ──────────────────────────────────────────────────────

/// Braille-character sparkline - 8x vertical resolution vs normal block chars.
/// Each braille character encodes 2 columns x 4 rows of dots.
/// We use single-column mode: each char = one data point, 8 vertical levels.
pub fn braille_sparkline(data: &[f64], width: usize) -> String {
    if data.is_empty() {
        return " ".repeat(width);
    }

    // Braille patterns for 0-8 dots filled from bottom
    // ⠀ ⡀ ⡄ ⡆ ⡇ ⣇ ⣧ ⣷ ⣿
    const BRAILLE: [char; 9] = ['⠀', '⡀', '⡄', '⡆', '⡇', '⣇', '⣧', '⣷', '⣿'];

    let start = if data.len() > width { data.len() - width } else { 0 };
    let slice = &data[start..];

    let max = slice.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1.0);

    let mut result = String::with_capacity(width * 3);
    for &val in slice {
        let level = ((val / max) * 8.0).round() as usize;
        result.push(BRAILLE[level.min(8)]);
    }

    // Pad to width if not enough data
    while result.chars().count() < width {
        result.insert(0, BRAILLE[0]);
    }

    result
}

// ── Layout & dispatch ──────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(0),   // content
            Constraint::Length(1), // status bar
        ])
        .split(size);

    draw_tabs(f, app, outer[0]);
    draw_panel(f, app, outer[1]);
    draw_status_bar(f, app, outer[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let panels = [
        Panel::Dashboard, Panel::Gpu, Panel::Cpu, Panel::Memory,
        Panel::Disk, Panel::Network, Panel::Processes, Panel::GpuControl,
    ];

    let titles: Vec<Line> = panels
        .iter()
        .map(|p| {
            let style = if *p == app.active_panel {
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(DIM)
            };
            Line::from(Span::styled(p.label(), style))
        })
        .collect();

    let idx = panels.iter().position(|p| *p == app.active_panel).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title(" Pane ")
                .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
                .border_style(BORDER_STYLE),
        )
        .select(idx)
        .highlight_style(Style::default().fg(ACCENT));

    f.render_widget(tabs, area);
}

fn draw_panel(f: &mut Frame, app: &App, area: Rect) {
    match app.active_panel {
        Panel::Dashboard => dashboard::draw(f, app, area),
        Panel::Gpu => gpu::draw(f, app, area),
        Panel::Cpu => cpu::draw(f, app, area),
        Panel::Memory => memory::draw(f, app, area),
        Panel::Disk => disk::draw(f, app, area),
        Panel::Network => network::draw(f, app, area),
        Panel::Processes => processes::draw(f, app, area),
        Panel::GpuControl => gpu_control::draw(f, app, area),
        Panel::VramCalc | Panel::Snapshot => {} // GUI-only panels
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let keys = if app.filtering {
        "Type to filter | Enter: apply | Esc: cancel"
    } else if app.confirm_kill.is_some() {
        "y: confirm kill | n: cancel"
    } else {
        match app.active_panel {
            Panel::GpuControl => "Tab: panels | \u{2190}/\u{2192}: adjust | \u{2191}/\u{2193}: select | q: quit",
            Panel::Processes => "Tab: panels | s: sort | /: filter | k: kill | \u{2191}/\u{2193}: scroll | q: quit",
            _ => "Tab: panels | h/g/c/m/d/n/p/x: jump | s: sort | /: filter | q: quit",
        }
    };

    let status = Line::from(vec![
        Span::styled(" Pane v2.0.0 ", Style::default().fg(Color::Black).bg(ACCENT)),
        Span::styled(" ", Style::default()),
        Span::styled(keys, Style::default().fg(DIM)),
    ]);

    f.render_widget(ratatui::widgets::Paragraph::new(status), area);
}

// ── Shared helpers ─────────────────────────────────────────────────────────

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_bytes_rate(bytes_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_sec))
}

pub fn usage_color(pct: f64) -> Color {
    if pct > 90.0 {
        Color::Red
    } else if pct > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
