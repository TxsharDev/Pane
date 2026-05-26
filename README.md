<p align="center">
  <h1 align="center">Pane</h1>
  <p align="center"><strong>A transparent window into your system.</strong></p>
  <p align="center">
    <a href="#install">Install</a> &bull;
    <a href="#why-pane">Why Pane</a> &bull;
    <a href="#features">Features</a> &bull;
    <a href="#screenshots">Screenshots</a> &bull;
    <a href="#build">Build</a>
  </p>
</p>

---

Your OS hides what your hardware is actually doing. Task Manager is a joke. htop doesn't know your GPU exists. HWiNFO is a spreadsheet from 2004.

**Pane cracks it open.** One binary. Every platform. Real metrics. No admin required.

Built by someone who writes CUDA inference engines for fun and got tired of alt-tabbing between 4 different monitoring tools to figure out why his GPU was stalling.

## Why Pane

Windows makes GPU monitoring deliberately opaque. WDDM abstracts away per-process VRAM. NVIDIA locks per-process utilization behind datacenter-only APIs. Task Manager shows you a percentage and calls it a day.

Pane goes around all of it:
- **Per-process GPU utilization** without admin elevation (via Windows Performance Counters, not the broken NVML path)
- **Per-process VRAM usage** — dedicated and shared, per engine (3D, video decode, encode, copy)
- **PCIe throughput** — actual TX/RX bandwidth in real-time, not theoretical max
- **Thermal intelligence** — core temp, hotspot temp, VRAM temp, throttle detection
- **Power draw** — real watts, not TDP guesses

On Linux, most of this data is accessible but scattered across `/sys/class/drm`, `/proc`, `nvidia-smi`, and 6 different tools. Pane unifies it.

On macOS, you get what Apple allows (which is more than you'd think).

One tool. One binary. Every platform. No excuses.

## Features

### GPU (the reason this exists)
- Per-process GPU engine utilization (3D, decode, encode, copy)
- Per-process VRAM allocation (dedicated + shared)
- Real-time PCIe bandwidth (TX/RX bytes/sec)
- Clock speeds (core, memory, video — current and boost)
- Temperature (core, hotspot, VRAM junction)
- Power draw (current watts, power limit, power cap)
- Fan speed (actual RPM, not "target percentage")
- Thermal throttle detection and alerting
- Multi-GPU support (heterogeneous — e.g. RTX 5090 + RTX 4090)
- NVIDIA (NVML + NVAPI) and AMD (ADLX) backends

### CPU
- Per-core utilization with frequency scaling
- Thread count, context switches
- Temperature per-core (where available)
- Process tree with CPU attribution

### Memory
- Physical and virtual memory pressure
- Per-process working set, private bytes, shared
- Commit charge and page fault rates
- Swap/pagefile utilization

### Disk
- Per-disk read/write throughput (bytes/sec)
- IOPS, queue depth, latency
- Per-process disk I/O attribution
- NVMe temperature and health (where exposed)

### Network
- Per-interface throughput (TX/RX)
- Per-process network usage
- Connection table (TCP/UDP active connections)

### UX
- Dense, information-rich TUI — no wasted space
- Sparkline graphs with history
- Sortable process table with GPU columns
- Responsive layout — adapts to terminal size
- Runs in any terminal (Windows Terminal, Alacritty, kitty, iTerm2, even cmd.exe)
- Single `.exe` / single binary — no installer, no dependencies, no runtime

## Install

### Download
Grab the latest release for your platform from [Releases](https://github.com/TxsharDev/pane/releases).

```bash
# Windows — just run it
pane.exe

# Linux / macOS
chmod +x pane
./pane
```

### Build from source
```bash
git clone https://github.com/TxsharDev/pane.git
cd pane
cargo build --release
# Binary at target/release/pane (.exe on Windows)
```

### Package managers (coming soon)
```bash
# cargo
cargo install pane

# scoop (Windows)
scoop install pane

# brew (macOS)
brew install pane
```

## Architecture

```
pane
├── src/
│   ├── main.rs              # Entry point, event loop, render loop
│   ├── app.rs               # Application state machine
│   ├── ui/                  # TUI layout and widgets (ratatui)
│   │   ├── gpu.rs           # GPU panel rendering
│   │   ├── cpu.rs           # CPU panel rendering
│   │   ├── memory.rs        # Memory panel rendering
│   │   ├── disk.rs          # Disk panel rendering
│   │   ├── network.rs       # Network panel rendering
│   │   └── processes.rs     # Process table with GPU columns
│   ├── metrics/             # Platform-abstracted data collection
│   │   ├── gpu/
│   │   │   ├── mod.rs       # GPU metrics trait
│   │   │   ├── nvml.rs      # NVIDIA backend (NVML + NVAPI)
│   │   │   ├── adlx.rs      # AMD backend (ADLX via FFI)
│   │   │   ├── pdh.rs       # Windows Performance Counters (per-process GPU)
│   │   │   └── sysfs.rs     # Linux /sys/class/drm fallback
│   │   ├── cpu.rs
│   │   ├── memory.rs
│   │   ├── disk.rs
│   │   └── network.rs
│   └── platform/            # OS-specific abstractions
│       ├── windows.rs
│       ├── linux.rs
│       └── macos.rs
├── Cargo.toml
├── CHANGELOG.md
└── README.md
```

### Tech Stack

| Layer | Choice | Why |
|-------|--------|-----|
| TUI framework | [ratatui](https://github.com/ratatui/ratatui) | 20k+ stars, immediate-mode, diff-based rendering |
| Terminal backend | [crossterm](https://github.com/crossterm-rs/crossterm) | Only real option with full Windows support |
| System metrics | [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | Battle-tested, cross-platform CPU/RAM/disk/net |
| NVIDIA GPU | [nvml-wrapper](https://github.com/Cldfire/nvml-wrapper) | Rust bindings for NVML — temp, power, clocks, PCIe |
| Windows GPU (per-process) | [windows-rs](https://github.com/microsoft/windows-rs) + PDH | Vendor-agnostic per-process GPU utilization |
| AMD GPU | ADLX via FFI | AMD's official monitoring SDK |
| Async runtime | [tokio](https://github.com/tokio-rs/tokio) | Background metric polling without blocking UI |

### How Pane gets GPU data on Windows (the hard part)

Windows hides GPU metrics behind WDDM. NVIDIA's NVML returns `NOT_AVAILABLE` for per-process VRAM on consumer GPUs (WDDM mode). Most tools give up here.

Pane doesn't:

1. **Per-process GPU utilization** — PDH `GPU Engine` counters. Same source as Task Manager, but we expose per-engine breakdown (3D vs decode vs encode vs copy). No admin needed.
2. **Per-process VRAM** — PDH `GPU Process Memory` counters. Dedicated and shared, per-process. No admin needed.
3. **Hardware metrics** — NVML for temp/power/clocks/PCIe (device-level, no admin). NVAPI undocumented calls for hotspot temp, VRAM temp, actual fan RPM.
4. **AMD path** — ADLX for device-level hardware metrics + same PDH counters for per-process data.

No admin elevation. No kernel driver. No "run as administrator" popup. It just works.

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Tab` | Cycle panels |
| `g` | Focus GPU panel |
| `c` | Focus CPU panel |
| `m` | Focus Memory panel |
| `d` | Focus Disk panel |
| `n` | Focus Network panel |
| `p` | Focus Process table |
| `s` | Cycle sort column in process table |
| `/` | Filter processes |
| `1-9` | Select GPU (multi-GPU systems) |
| `?` | Help |

## Roadmap

- [x] Project architecture
- [ ] Core metric collection (CPU, RAM, disk, net)
- [ ] NVIDIA GPU metrics via NVML
- [ ] Per-process GPU metrics via PDH (Windows)
- [ ] TUI layout and rendering
- [ ] Process table with GPU columns
- [ ] AMD GPU support via ADLX
- [ ] Linux GPU metrics (/sys/class/drm + NVML)
- [ ] macOS support (IOKit + Metal)
- [ ] Config file (refresh rate, layout, colors)
- [ ] Historical graphs with configurable time window
- [ ] Export metrics (JSON, CSV)
- [ ] Alert rules (temp > X, throttle detected, etc.)

## Why not just use...

| Tool | Problem |
|------|---------|
| **Task Manager** | Shows one GPU percentage. No per-engine breakdown. No PCIe bandwidth. No thermals. No power. |
| **HWiNFO** | Sensor dump in a GUI from 2004. No per-process GPU. Not a workflow tool. |
| **Process Explorer** | Last real update was a decade ago. No GPU awareness at all. |
| **btop** | Great on Linux. Windows support is a separate fork and an afterthought. No deep GPU. |
| **bottom** | Good Rust TUI monitor. GPU support is surface-level (basic NVIDIA only). |
| **nvidia-smi** | NVIDIA only. Text dump. No TUI. Per-process VRAM broken on consumer GPUs (WDDM). |
| **Afterburner** | Overlay for games. Not a system monitor. No per-process. Being discontinued. |

Pane replaces all of them.

## Contributing

Pane is open source under the MIT License.

If you want to contribute, the GPU metrics layer (`src/metrics/gpu/`) is where the interesting problems are. Per-process GPU attribution on Windows is a solved-but-poorly-documented problem — if you've worked with D3DKMT, PDH, NVML, or ADLX, your expertise is welcome.

```bash
# Run in dev mode
cargo run

# Run tests
cargo test

# Run with GPU features disabled (for CI / machines without GPUs)
cargo run --no-default-features
```

## License

MIT License. See [LICENSE](LICENSE).

---

<p align="center">
  <strong>Built by <a href="https://github.com/TxsharDev">Tushar Sharma</a></strong><br>
  <em>Because your OS shouldn't hide what your hardware is doing.</em>
</p>
