//! Application state for Pane.
//!
//! All data flows through `App` - metric collectors write into it,
//! the UI reads from it. No direct coupling between collectors and renderers.

#![allow(dead_code)]

use std::time::{Duration, Instant};

/// UI panels - Dashboard is the default overview, others are detail views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Dashboard,
    Gpu,
    Cpu,
    Memory,
    Disk,
    Network,
    Processes,
    GpuControl,
    VramCalc,
    Snapshot,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Dashboard => Panel::Gpu,
            Panel::Gpu => Panel::Cpu,
            Panel::Cpu => Panel::Memory,
            Panel::Memory => Panel::Disk,
            Panel::Disk => Panel::Network,
            Panel::Network => Panel::Processes,
            Panel::Processes => Panel::GpuControl,
            Panel::GpuControl => Panel::VramCalc,
            Panel::VramCalc => Panel::Snapshot,
            Panel::Snapshot => Panel::Dashboard,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Panel::Dashboard => "Dashboard",
            Panel::Gpu => "GPU",
            Panel::Cpu => "CPU",
            Panel::Memory => "Memory",
            Panel::Disk => "Disk",
            Panel::Network => "Network",
            Panel::Processes => "Processes",
            Panel::GpuControl => "GPU Ctrl",
            Panel::VramCalc => "VRAM Calc",
            Panel::Snapshot => "Snapshot",
        }
    }
}

const HISTORY_LEN: usize = 200;

/// Ring buffer for sparkline/graph history. Stores the last N samples.
#[derive(Debug, Clone)]
pub struct History {
    pub data: Vec<f64>,
    capacity: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(HISTORY_LEN),
            capacity: HISTORY_LEN,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.data.len() >= self.capacity {
            self.data.remove(0);
        }
        self.data.push(value);
    }

    /// Get the latest value, or 0.0 if empty.
    pub fn last(&self) -> f64 {
        self.data.last().copied().unwrap_or(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct CpuCore {
    pub usage: f64,
    pub freq_mhz: u64,
    pub history: History,
}

#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub total_usage: f64,
    pub total_history: History,
    pub cores: Vec<CpuCore>,
    pub name: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
}

#[derive(Debug, Clone)]
pub struct MemMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub usage_history: History,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
}

#[derive(Debug, Clone)]
pub struct NetInterface {
    pub name: String,
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
    pub total_rx: u64,
    pub total_tx: u64,
}

#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub name: String,
    pub utilization: f64,
    pub utilization_history: History,
    pub vram_used: u64,
    pub vram_total: u64,
    pub vram_history: History,
    pub temp_core: Option<u32>,
    pub temp_hotspot: Option<u32>,
    pub temp_vram: Option<u32>,
    pub temp_history: History,
    pub power_watts: Option<f64>,
    pub power_limit: Option<f64>,
    pub power_history: History,
    pub fan_rpm: Option<u32>,
    pub clock_core_mhz: Option<u32>,
    pub clock_mem_mhz: Option<u32>,
    pub pcie_tx_bytes_sec: Option<u64>,
    pub pcie_rx_bytes_sec: Option<u64>,
    pub processes: Vec<GpuProcessInfo>,
}

/// A process using this GPU - PID, name, type, VRAM.
#[derive(Debug, Clone)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub name: String,
    pub used_gpu_memory: u64,
    pub kind: GpuProcessKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuProcessKind {
    Graphics,
    Compute,
}

impl GpuMetrics {
    pub fn vram_pct(&self) -> f64 {
        if self.vram_total > 0 {
            (self.vram_used as f64 / self.vram_total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// GPU control state - adjustable values for fan, power, clocks.
#[derive(Debug, Clone)]
pub struct GpuControl {
    pub fan_speed_pct: Option<u32>,    // None = auto, Some = manual override
    pub power_limit_watts: Option<f64>,
    pub clock_offset_mhz: i32,        // Core clock offset (+/-)
    pub mem_offset_mhz: i32,          // Memory clock offset (+/-)
    pub fan_auto: bool,
}

impl GpuControl {
    pub fn new() -> Self {
        Self {
            fan_speed_pct: None,
            power_limit_watts: None,
            clock_offset_mhz: 0,
            mem_offset_mhz: 0,
            fan_auto: true,
        }
    }
}

/// Which control row is selected in the GPU Control panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlRow {
    FanSpeed,
    PowerLimit,
    CoreClock,
    MemClock,
}

impl ControlRow {
    pub fn next(self) -> Self {
        match self {
            ControlRow::FanSpeed => ControlRow::PowerLimit,
            ControlRow::PowerLimit => ControlRow::CoreClock,
            ControlRow::CoreClock => ControlRow::MemClock,
            ControlRow::MemClock => ControlRow::FanSpeed,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            ControlRow::FanSpeed => ControlRow::MemClock,
            ControlRow::PowerLimit => ControlRow::FanSpeed,
            ControlRow::CoreClock => ControlRow::PowerLimit,
            ControlRow::MemClock => ControlRow::CoreClock,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
    GpuUtil,
    GpuVram,
}

impl SortColumn {
    pub fn next(self) -> Self {
        match self {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::GpuUtil,
            SortColumn::GpuUtil => SortColumn::GpuVram,
            SortColumn::GpuVram => SortColumn::Pid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_bytes: u64,
    pub gpu_util: Option<f64>,
    pub gpu_vram: Option<u64>,
}

#[derive(Clone)]
pub struct App {
    pub running: bool,
    pub active_panel: Panel,
    pub cpu: CpuMetrics,
    pub memory: MemMetrics,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetInterface>,
    pub gpus: Vec<GpuMetrics>,
    pub gpu_controls: Vec<GpuControl>,
    pub control_row: ControlRow,
    pub processes: Vec<ProcessInfo>,
    pub selected_gpu: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub process_scroll: usize,
    pub process_selected: usize,
    pub filter: String,
    pub filtering: bool,
    pub tick_rate: Duration,
    pub last_tick: Instant,
    pub confirm_kill: Option<u32>,
    pub status_msg: Option<(String, bool)>, // (message, is_error)
}

impl App {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            running: true,
            active_panel: Panel::Dashboard,
            cpu: CpuMetrics {
                total_usage: 0.0,
                total_history: History::new(),
                cores: Vec::new(),
                name: String::new(),
                physical_cores: 0,
                logical_cores: 0,
            },
            memory: MemMetrics {
                total_bytes: 0,
                used_bytes: 0,
                swap_total: 0,
                swap_used: 0,
                usage_history: History::new(),
            },
            disks: Vec::new(),
            networks: Vec::new(),
            gpus: Vec::new(),
            gpu_controls: Vec::new(),
            control_row: ControlRow::FanSpeed,
            processes: Vec::new(),
            selected_gpu: 0,
            sort_column: SortColumn::Cpu,
            sort_ascending: false,
            process_scroll: 0,
            process_selected: 0,
            filter: String::new(),
            filtering: false,
            tick_rate,
            last_tick: Instant::now(),
            confirm_kill: None,
            status_msg: None,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_panel(&mut self) {
        self.active_panel = self.active_panel.next();
    }

    pub fn cycle_sort(&mut self) {
        self.sort_column = self.sort_column.next();
    }

    pub fn sorted_processes(&self) -> Vec<&ProcessInfo> {
        let mut procs: Vec<&ProcessInfo> = self.processes.iter().collect();

        if !self.filter.is_empty() {
            let f = self.filter.to_lowercase();
            procs.retain(|p| p.name.to_lowercase().contains(&f));
        }

        procs.sort_by(|a, b| {
            let ord = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
                SortColumn::GpuUtil => {
                    let av = a.gpu_util.unwrap_or(0.0);
                    let bv = b.gpu_util.unwrap_or(0.0);
                    av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::GpuVram => {
                    let av = a.gpu_vram.unwrap_or(0);
                    let bv = b.gpu_vram.unwrap_or(0);
                    av.cmp(&bv)
                }
            };
            if self.sort_ascending { ord } else { ord.reverse() }
        });

        procs
    }
}
