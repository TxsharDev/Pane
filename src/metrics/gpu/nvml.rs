//! NVIDIA GPU metrics via NVML (NVIDIA Management Library).
//!
//! Provides device-level metrics: utilization, VRAM, temperature, power,
//! clocks, PCIe throughput, and fan speed. Works on both Windows and Linux.
//!
//! Note: On Windows (WDDM mode), per-process VRAM from NVML returns
//! NOT_AVAILABLE. Per-process GPU data comes from PDH instead (see pdh.rs).

use std::sync::OnceLock;

use crate::app::{GpuMetrics, GpuProcessInfo, GpuProcessKind, History};
use super::GpuBackend;

/// Lazily initialized NVML instance - lives for the process lifetime.
static NVML: OnceLock<nvml_wrapper::Nvml> = OnceLock::new();

pub struct NvmlBackend {
    devices: Vec<NvmlDevice>,
}

struct NvmlDevice {
    index: u32,
    metrics: GpuMetrics,
}

impl NvmlBackend {
    pub fn try_new() -> Option<Self> {
        let nvml = match nvml_wrapper::Nvml::init() {
            Ok(n) => {
                // Store in OnceLock for later use
                let _ = NVML.set(n);
                NVML.get()?
            }
            Err(_) => return None,
        };

        let count = nvml.device_count().ok()?;
        if count == 0 {
            return None;
        }

        let mut devices = Vec::new();
        for i in 0..count {
            let handle = match nvml.device_by_index(i) {
                Ok(h) => h,
                Err(_) => continue,
            };
            let name = handle.name().unwrap_or_else(|_| format!("GPU {}", i));
            let vram_total = handle.memory_info().map(|m| m.total).unwrap_or(0);

            devices.push(NvmlDevice {
                index: i,
                metrics: GpuMetrics {
                    name,
                    utilization: 0.0,
                    utilization_history: History::new(),
                    vram_used: 0,
                    vram_total,
                    vram_history: History::new(),
                    temp_core: None,
                    temp_hotspot: None,
                    temp_vram: None,
                    temp_history: History::new(),
                    power_watts: None,
                    power_limit: None,
                    power_history: History::new(),
                    fan_rpm: None,
                    clock_core_mhz: None,
                    clock_mem_mhz: None,
                    pcie_tx_bytes_sec: None,
                    pcie_rx_bytes_sec: None,
                    processes: Vec::new(),
                },
            });
        }

        if devices.is_empty() {
            return None;
        }

        Some(Self { devices })
    }
}

impl GpuBackend for NvmlBackend {
    fn refresh(&mut self) {
        let nvml = match NVML.get() {
            Some(n) => n,
            None => return,
        };

        for dev in &mut self.devices {
            let h = match nvml.device_by_index(dev.index) {
                Ok(h) => h,
                Err(_) => continue,
            };

            // Utilization (% of time GPU was busy)
            if let Ok(util) = h.utilization_rates() {
                dev.metrics.utilization = util.gpu as f64;
                dev.metrics.utilization_history.push(util.gpu as f64);
            }

            // VRAM (device-level, always works)
            if let Ok(mem) = h.memory_info() {
                dev.metrics.vram_used = mem.used;
                dev.metrics.vram_total = mem.total;
            }

            // Temperature
            dev.metrics.temp_core = h
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .ok();

            // Power (NVML returns milliwatts)
            dev.metrics.power_watts = h.power_usage().ok().map(|mw| mw as f64 / 1000.0);
            dev.metrics.power_limit = h
                .power_management_limit()
                .ok()
                .map(|mw| mw as f64 / 1000.0);

            // Clocks
            dev.metrics.clock_core_mhz = h
                .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                .ok();
            dev.metrics.clock_mem_mhz = h
                .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
                .ok();

            // PCIe throughput (NVML returns KB/s, we want bytes/sec)
            dev.metrics.pcie_tx_bytes_sec = h
                .pcie_throughput(nvml_wrapper::enum_wrappers::device::PcieUtilCounter::Send)
                .ok()
                .map(|kb| kb as u64 * 1024);
            dev.metrics.pcie_rx_bytes_sec = h
                .pcie_throughput(nvml_wrapper::enum_wrappers::device::PcieUtilCounter::Receive)
                .ok()
                .map(|kb| kb as u64 * 1024);

            // Fan speed (NVML gives percentage, not RPM)
            dev.metrics.fan_rpm = h.fan_speed(0).ok();

            // Per-GPU processes
            let mut gpu_procs = Vec::new();

            // Graphics processes
            if let Ok(procs) = h.running_graphics_processes() {
                for p in procs {
                    gpu_procs.push(GpuProcessInfo {
                        pid: p.pid,
                        name: process_name(p.pid),
                        used_gpu_memory: extract_gpu_mem(p.used_gpu_memory),
                        kind: GpuProcessKind::Graphics,
                    });
                }
            }

            // Compute processes
            if let Ok(procs) = h.running_compute_processes() {
                for p in procs {
                    // Avoid duplicates if same PID is in both lists
                    if !gpu_procs.iter().any(|gp| gp.pid == p.pid) {
                        gpu_procs.push(GpuProcessInfo {
                            pid: p.pid,
                            name: process_name(p.pid),
                            used_gpu_memory: extract_gpu_mem(p.used_gpu_memory),
                            kind: GpuProcessKind::Compute,
                        });
                    }
                }
            }

            // Sort by VRAM usage descending
            gpu_procs.sort_by_key(|p| std::cmp::Reverse(p.used_gpu_memory));
            dev.metrics.processes = gpu_procs;
        }
    }

    fn metrics(&self) -> Vec<GpuMetrics> {
        self.devices.iter().map(|d| d.metrics.clone()).collect()
    }

    fn set_power_limit(&mut self, gpu_index: usize, watts: f64) -> Result<(), String> {
        let nvml = NVML.get().ok_or("NVML not initialized")?;
        let dev = self.devices.get(gpu_index).ok_or("Invalid GPU index")?;
        let mut handle = nvml.device_by_index(dev.index).map_err(|e| format!("Device error: {}", e))?;

        let milliwatts = (watts * 1000.0) as u32;
        handle.set_power_management_limit(milliwatts)
            .map_err(|e| format!("Failed to set power limit: {} (requires admin)", e))
    }
}

/// Extract GPU memory usage from NVML's enum type.
fn extract_gpu_mem(mem: nvml_wrapper::enums::device::UsedGpuMemory) -> u64 {
    match mem {
        nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => bytes,
        nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0,
    }
}

/// Resolve PID to process name via sysinfo.
fn process_name(pid: u32) -> String {
    use sysinfo::{Pid, System};
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
    sys.process(Pid::from_u32(pid))
        .map(|p| p.name().to_string_lossy().to_string())
        .unwrap_or_else(|| format!("PID {}", pid))
}
