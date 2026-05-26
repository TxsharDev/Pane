//! Dashboard - everything at a glance.
//!
//! Layout:
//! ┌─────────────────────────────────────────────────────────┐
//! │  GPU 0: RTX 5090   ▁▂▃▅▇█▇▅▃  45%   45C   180W       │
//! │  GPU 1: RTX 4090   ▁▁▁▁▁▁▁▁▁   0%   38C    12W       │
//! ├────────────────────────────┬────────────────────────────┤
//! │  CPU  ▁▂▅▇▅▃▂▁   23%      │  RAM  ████░░░░  42.1/64GB │
//! ├────────────────────────────┼────────────────────────────┤
//! │  Disk C: 1.2 GB/s R        │  Net: 12.4 MB/s ↓         │
//! ├────────────────────────────┴────────────────────────────┤
//! │  Top processes by CPU/GPU                               │
//! └─────────────────────────────────────────────────────────┘

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Cell, Table};

use crate::app::App;
use super::{format_bytes, format_bytes_rate, usage_color, braille_sparkline, ACCENT, DIM, BORDER_STYLE};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // Vertical split: GPU cards | CPU+RAM row | Disk+Net row | Top procs
    let gpu_rows = app.gpus.len().max(1) as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(gpu_rows + 2),  // GPU cards
            Constraint::Length(5),              // CPU + RAM
            Constraint::Length(5),              // Disk + Net
            Constraint::Min(6),                // Top processes
        ])
        .split(area);

    draw_gpu_summary(f, app, chunks[0]);
    draw_cpu_ram_row(f, app, chunks[1]);
    draw_disk_net_row(f, app, chunks[2]);
    draw_top_processes(f, app, chunks[3]);
}

fn draw_gpu_summary(f: &mut Frame, app: &App, area: Rect) {
    if app.gpus.is_empty() {
        let msg = Paragraph::new(Line::from(vec![
            Span::styled(" No GPU detected", Style::default().fg(DIM)),
        ]))
        .block(Block::default().borders(Borders::ALL).title(" GPU ").border_style(BORDER_STYLE));
        f.render_widget(msg, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (i, gpu) in app.gpus.iter().enumerate() {
        let spark = braille_sparkline(&gpu.utilization_history.data, 20);
        let sel = if i == app.selected_gpu { ">" } else { " " };
        let temp_str = gpu.temp_core.map(|t| format!("{}C", t)).unwrap_or_else(|| "--".into());
        let power_str = gpu.power_watts.map(|w| format!("{:.0}W", w)).unwrap_or_else(|| "--".into());
        let vram_str = format!("{}/{}", format_bytes(gpu.vram_used), format_bytes(gpu.vram_total));

        lines.push(Line::from(vec![
            Span::styled(format!("{} GPU {}: ", sel, i), Style::default().fg(if i == app.selected_gpu { ACCENT } else { DIM })),
            Span::styled(format!("{:<22}", gpu.name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", spark), Style::default().fg(usage_color(gpu.utilization))),
            Span::styled(format!("{:>5.1}%", gpu.utilization), Style::default().fg(usage_color(gpu.utilization)).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {:<6}", temp_str), Style::default().fg(
                gpu.temp_core.map(|t| if t > 80 { Color::Red } else if t > 65 { Color::Yellow } else { Color::Green }).unwrap_or(DIM)
            )),
            Span::styled(format!("  {:<6}", power_str), Style::default().fg(Color::White)),
            Span::styled(format!("  {}", vram_str), Style::default().fg(usage_color(gpu.vram_pct()))),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" GPU ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .border_style(BORDER_STYLE);
    let widget = Paragraph::new(lines).block(block);
    f.render_widget(widget, area);
}

fn draw_cpu_ram_row(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // CPU card
    let spark = braille_sparkline(&app.cpu.total_history.data, 25);
    let cpu_lines = vec![
        Line::from(vec![
            Span::styled(&app.cpu.name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(format!("{} ", spark), Style::default().fg(usage_color(app.cpu.total_usage))),
            Span::styled(format!("{:.1}%", app.cpu.total_usage), Style::default().fg(usage_color(app.cpu.total_usage)).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}/{}c", app.cpu.physical_cores, app.cpu.logical_cores), Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Avg freq: {} MHz", app.cpu.cores.first().map(|c| c.freq_mhz).unwrap_or(0)),
                Style::default().fg(DIM),
            ),
        ]),
    ];
    let cpu_block = Block::default()
        .borders(Borders::ALL)
        .title(" CPU ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .border_style(BORDER_STYLE);
    f.render_widget(Paragraph::new(cpu_lines).block(cpu_block), cols[0]);

    // RAM card
    let mem_pct = if app.memory.total_bytes > 0 {
        (app.memory.used_bytes as f64 / app.memory.total_bytes as f64) * 100.0
    } else { 0.0 };
    let bar_width = 30usize;
    let filled = ((mem_pct / 100.0) * bar_width as f64) as usize;
    let bar = format!("{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(bar_width.saturating_sub(filled)),
    );
    let ram_lines = vec![
        Line::from(vec![
            Span::styled(bar, Style::default().fg(usage_color(mem_pct))),
            Span::styled(format!(" {:.1}%", mem_pct), Style::default().fg(usage_color(mem_pct)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} / {}", format_bytes(app.memory.used_bytes), format_bytes(app.memory.total_bytes)),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Swap: {} / {}", format_bytes(app.memory.swap_used), format_bytes(app.memory.swap_total)),
                Style::default().fg(DIM),
            ),
        ]),
    ];
    let ram_block = Block::default()
        .borders(Borders::ALL)
        .title(" Memory ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .border_style(BORDER_STYLE);
    f.render_widget(Paragraph::new(ram_lines).block(ram_block), cols[1]);
}

fn draw_disk_net_row(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Disk summary - show busiest disk
    let mut disk_lines = Vec::new();
    for d in app.disks.iter().take(3) {
        let pct = if d.total_bytes > 0 { (d.used_bytes as f64 / d.total_bytes as f64) * 100.0 } else { 0.0 };
        disk_lines.push(Line::from(vec![
            Span::styled(format!("{:<8}", d.mount), Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.0}% ", pct), Style::default().fg(usage_color(pct))),
            Span::styled(format!("R:{:<10} W:{}", format_bytes_rate(d.read_bytes_sec), format_bytes_rate(d.write_bytes_sec)), Style::default().fg(DIM)),
        ]));
    }
    let disk_block = Block::default()
        .borders(Borders::ALL)
        .title(" Disk ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .border_style(BORDER_STYLE);
    f.render_widget(Paragraph::new(disk_lines).block(disk_block), cols[0]);

    // Network summary
    let mut net_lines = Vec::new();
    for n in app.networks.iter().filter(|n| n.rx_bytes_sec > 0 || n.tx_bytes_sec > 0).take(3) {
        net_lines.push(Line::from(vec![
            Span::styled(format!("{:<15}", n.name), Style::default().fg(Color::White)),
            Span::styled(format!("\u{2193}{:<10}", format_bytes_rate(n.rx_bytes_sec)), Style::default().fg(Color::Green)),
            Span::styled(format!(" \u{2191}{}", format_bytes_rate(n.tx_bytes_sec)), Style::default().fg(Color::Magenta)),
        ]));
    }
    if net_lines.is_empty() {
        net_lines.push(Line::from(Span::styled("No active interfaces", Style::default().fg(DIM))));
    }
    let net_block = Block::default()
        .borders(Borders::ALL)
        .title(" Network ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .border_style(BORDER_STYLE);
    f.render_widget(Paragraph::new(net_lines).block(net_block), cols[1]);
}

fn draw_top_processes(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("PID").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("CPU%").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Memory").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    // Show top 10 by CPU usage
    let mut procs = app.sorted_processes();
    procs.truncate(10);

    let rows: Vec<Row> = procs.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:.1}", p.cpu_usage))
                .style(Style::default().fg(usage_color(p.cpu_usage))),
            Cell::from(format_bytes(p.memory_bytes)),
        ])
    }).collect();

    let table = Table::new(
        rows,
        [Constraint::Length(8), Constraint::Percentage(40), Constraint::Length(8), Constraint::Length(12)],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Top Processes ")
            .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
            .border_style(BORDER_STYLE),
    );
    f.render_widget(table, area);
}
