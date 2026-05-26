//! Windows Performance Counter (PDH) backend for per-process GPU metrics.
//!
//! This is exactly how Task Manager gets per-process GPU utilization and VRAM.
//! Vendor-agnostic (works with NVIDIA, AMD, Intel), no admin required.
//!
//! Counter paths:
//! - `\GPU Engine(pid_XXXX_*)\Utilization Percentage` - per-process per-engine GPU %
//! - `\GPU Process Memory(pid_XXXX_*)\Dedicated Usage` - per-process dedicated VRAM
//! - `\GPU Process Memory(pid_XXXX_*)\Shared Usage` - per-process shared GPU memory

#[cfg(target_os = "windows")]
mod inner {
    use std::collections::HashMap;
    use std::process::Command;

    #[derive(Debug, Clone, Default)]
    pub struct ProcessGpuUsage {
        #[allow(dead_code)]
        pub pid: u32,
        pub utilization: f64,
        pub dedicated_vram: u64,
        pub shared_vram: u64,
    }

    /// Hide console window when spawning child processes on Windows GUI apps.
    fn hidden_command(program: &str) -> Command {
        let mut cmd = Command::new(program);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd
    }

    pub struct PdhGpuCollector {
        data: HashMap<u32, ProcessGpuUsage>,
    }

    impl PdhGpuCollector {
        pub fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }

        pub fn refresh(&mut self) {
            let mut map: HashMap<u32, ProcessGpuUsage> = HashMap::new();

            // GPU Engine utilization
            if let Ok(output) = hidden_command("powershell")
                .args([
                    "-NoProfile", "-NonInteractive", "-Command",
                    "(Get-Counter '\\GPU Engine(*)\\Utilization Percentage').CounterSamples | ForEach-Object { $_.InstanceName + '=' + $_.CookedValue.ToString('F2') }"
                ])
                .output()
                && output.status.success()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Some((instance, value)) = line.split_once('=')
                        && let (Some(pid), Some(util)) = (extract_pid(instance), value.parse::<f64>().ok())
                    {
                        let entry = map.entry(pid).or_insert_with(|| ProcessGpuUsage { pid, ..Default::default() });
                        if util > entry.utilization {
                            entry.utilization = util;
                        }
                    }
                }
            }

            // GPU Process Memory
            if let Ok(output) = hidden_command("powershell")
                .args([
                    "-NoProfile", "-NonInteractive", "-Command",
                    "(Get-Counter '\\GPU Process Memory(*)\\Dedicated Usage','\\GPU Process Memory(*)\\Shared Usage').CounterSamples | ForEach-Object { $_.Path + '=' + $_.CookedValue.ToString('F0') }"
                ])
                .output()
                && output.status.success()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Some((path, value)) = line.split_once('=') {
                        let path_lower = path.to_lowercase();
                        if let Some(pid) = extract_pid_from_path(&path_lower) {
                            let bytes = value.trim().parse::<f64>().unwrap_or(0.0) as u64;
                            let entry = map.entry(pid).or_insert_with(|| ProcessGpuUsage { pid, ..Default::default() });
                            if path_lower.contains("dedicated usage") {
                                entry.dedicated_vram = bytes;
                            } else if path_lower.contains("shared usage") {
                                entry.shared_vram = bytes;
                            }
                        }
                    }
                }
            }

            self.data = map;
        }

        pub fn per_process(&self) -> &HashMap<u32, ProcessGpuUsage> {
            &self.data
        }
    }

    fn extract_pid(instance: &str) -> Option<u32> {
        let lower = instance.to_lowercase();
        if let Some(rest) = lower.strip_prefix("pid_") {
            rest.split('_').next()?.parse().ok()
        } else {
            None
        }
    }

    fn extract_pid_from_path(path: &str) -> Option<u32> {
        let start = path.find("pid_")? + 4;
        let rest = &path[start..];
        rest.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
    }
}

#[cfg(target_os = "windows")]
pub use inner::*;

#[cfg(not(target_os = "windows"))]
pub mod inner {
    use std::collections::HashMap;

    #[derive(Debug, Clone, Default)]
    pub struct ProcessGpuUsage {
        pub pid: u32,
        pub utilization: f64,
        pub dedicated_vram: u64,
        pub shared_vram: u64,
    }

    pub struct PdhGpuCollector {
        data: HashMap<u32, ProcessGpuUsage>,
    }

    impl PdhGpuCollector {
        pub fn new() -> Self {
            Self { data: HashMap::new() }
        }
        pub fn refresh(&mut self) {}
        pub fn per_process(&self) -> &HashMap<u32, ProcessGpuUsage> {
            &self.data
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub use inner::*;
