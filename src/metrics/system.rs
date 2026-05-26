//! System metric collection via the `sysinfo` crate.
//!
//! Collects CPU, memory, disk, and network metrics at each tick.
//! Disk and network rates are computed as deltas between samples.
//! Process list includes PID, name, CPU%, and memory - GPU columns
//! are populated separately by the GPU backend (PDH on Windows).

use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use std::collections::HashMap;
use std::time::Instant;

use crate::app::{CpuCore, CpuMetrics, DiskInfo, History, MemMetrics, NetInterface, ProcessInfo};

/// Wraps `sysinfo` and tracks deltas for rate-based metrics (disk IO, network throughput).
pub struct SystemCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    prev_net_rx: HashMap<String, u64>,
    prev_net_tx: HashMap<String, u64>,
    prev_disk_read: HashMap<String, u64>,
    prev_disk_write: HashMap<String, u64>,
    last_sample: Instant,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        Self {
            sys,
            disks,
            networks,
            prev_net_rx: HashMap::new(),
            prev_net_tx: HashMap::new(),
            prev_disk_read: HashMap::new(),
            prev_disk_write: HashMap::new(),
            last_sample: Instant::now(),
        }
    }

    /// Refresh all system metrics. Call once per tick.
    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        self.disks.refresh(false);
        self.networks.refresh(false);
    }

    /// Update CPU metrics and push to history.
    pub fn cpu(&self, prev: &mut CpuMetrics) {
        let cpus = self.sys.cpus();
        let total: f64 = cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cpus.len() as f64;

        prev.total_usage = total;
        prev.total_history.push(total);
        prev.name = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
        prev.physical_cores = System::physical_core_count().unwrap_or(0);
        prev.logical_cores = cpus.len();

        // Grow or shrink cores vec to match actual core count
        while prev.cores.len() < cpus.len() {
            prev.cores.push(CpuCore {
                usage: 0.0,
                freq_mhz: 0,
                history: History::new(),
            });
        }
        prev.cores.truncate(cpus.len());

        for (i, cpu) in cpus.iter().enumerate() {
            prev.cores[i].usage = cpu.cpu_usage() as f64;
            prev.cores[i].freq_mhz = cpu.frequency();
            prev.cores[i].history.push(cpu.cpu_usage() as f64);
        }
    }

    /// Update memory metrics and push to history.
    pub fn memory(&self, prev: &mut MemMetrics) {
        prev.total_bytes = self.sys.total_memory();
        prev.used_bytes = self.sys.used_memory();
        prev.swap_total = self.sys.total_swap();
        prev.swap_used = self.sys.used_swap();
        let usage_pct = if prev.total_bytes > 0 {
            (prev.used_bytes as f64 / prev.total_bytes as f64) * 100.0
        } else {
            0.0
        };
        prev.usage_history.push(usage_pct);
    }

    /// Compute per-disk usage and IO rates (bytes/sec since last sample).
    pub fn disks(&mut self) -> Vec<DiskInfo> {
        let elapsed = self.last_sample.elapsed().as_secs_f64().max(0.1);

        let mut result = Vec::new();
        for disk in self.disks.list() {
            let name = disk.name().to_string_lossy().to_string();
            let mount = disk.mount_point().to_string_lossy().to_string();

            let read_now = disk.usage().read_bytes;
            let write_now = disk.usage().written_bytes;

            let prev_r = self.prev_disk_read.get(&name).copied().unwrap_or(read_now);
            let prev_w = self.prev_disk_write.get(&name).copied().unwrap_or(write_now);

            let read_sec = ((read_now.saturating_sub(prev_r)) as f64 / elapsed) as u64;
            let write_sec = ((write_now.saturating_sub(prev_w)) as f64 / elapsed) as u64;

            self.prev_disk_read.insert(name.clone(), read_now);
            self.prev_disk_write.insert(name.clone(), write_now);

            result.push(DiskInfo {
                name,
                mount,
                total_bytes: disk.total_space(),
                used_bytes: disk.total_space() - disk.available_space(),
                read_bytes_sec: read_sec,
                write_bytes_sec: write_sec,
            });
        }
        result
    }

    /// Compute per-interface network rates (bytes/sec since last sample).
    pub fn networks(&mut self) -> Vec<NetInterface> {
        let elapsed = self.last_sample.elapsed().as_secs_f64().max(0.1);

        let mut result = Vec::new();
        for (name, data) in self.networks.list() {
            let rx_now = data.total_received();
            let tx_now = data.total_transmitted();

            let prev_rx = self.prev_net_rx.get(name).copied().unwrap_or(rx_now);
            let prev_tx = self.prev_net_tx.get(name).copied().unwrap_or(tx_now);

            let rx_sec = ((rx_now.saturating_sub(prev_rx)) as f64 / elapsed) as u64;
            let tx_sec = ((tx_now.saturating_sub(prev_tx)) as f64 / elapsed) as u64;

            self.prev_net_rx.insert(name.clone(), rx_now);
            self.prev_net_tx.insert(name.clone(), tx_now);

            result.push(NetInterface {
                name: name.clone(),
                rx_bytes_sec: rx_sec,
                tx_bytes_sec: tx_sec,
                total_rx: rx_now,
                total_tx: tx_now,
            });
        }
        result
    }

    /// Snapshot all running processes. GPU columns are None - filled by GPU backend.
    pub fn processes(&self) -> Vec<ProcessInfo> {
        self.sys
            .processes()
            .iter()
            .map(|(pid, proc_)| ProcessInfo {
                pid: pid.as_u32(),
                name: proc_.name().to_string_lossy().to_string(),
                cpu_usage: proc_.cpu_usage() as f64,
                memory_bytes: proc_.memory(),
                gpu_util: None,
                gpu_vram: None,
            })
            .collect()
    }

    /// Mark the end of a sample interval (for rate calculations).
    pub fn mark_tick(&mut self) {
        self.last_sample = Instant::now();
    }
}
