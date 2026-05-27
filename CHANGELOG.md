# Changelog

All notable changes to Pane will be documented here.

## [4.0.1] - 2026-05-26

### Fixed
- **Windows executable icon** - embedded .ico resource via winresource so the icon shows on taskbar pins, file explorer, and downloaded .exe (was missing, only the runtime window icon worked)

## [4.0.0] - 2026-05-26 - First Public Release

### Added
- **Custom app icon** - logo.png embedded as window/taskbar icon
- **Airstrike Bold font** - custom branding font for PANE logo in sidebar and loading screen
- "SYSTEM MONITOR" subtitle under sidebar logo

### Changed
- **Color palette overhaul** - sky blue accent (#38BDF8), richer emerald/amber/rose/violet, deeper blacks, new card_bg layer for depth
- **Font sizing** - body 14px, headings 20px, monospace 13px, bigger stat card values (26px monospace)
- **Sidebar redesign** - centered logo, full-width theme toggle button, full-width nav labels (removed badge abbreviations), cleaner spacing
- **Loading screen** - 80px Airstrike font, centered branding
- Section headers enlarged (15px, thicker accent bar)
- Button padding increased for better touch targets

### Fixed
- All clippy warnings resolved (removed unused control_card closure, send_gpu_command method, icon method)
- Collapsed nested if statements in PDH collector and GPU control
- Zero warnings on release build

## [3.7.0] - 2026-05-25

### Added
- **GPU Control admin UX** - yellow warning banner when not running as admin, explains how to elevate
- Power limit slider disabled when not elevated (shows "Needs admin" badge)
- Fan/clock controls show "Requires NVAPI (coming soon)" instead of fake sliders
- Apply button disabled and shows hover tooltip when not admin
- NVML badge on power limit when elevated and functional

### Changed
- **README completely rewritten** - reflects GUI app, accurate feature list, platform support matrix, PDH accuracy notes, honest cross-platform scope
- GPU Control Apply button renamed to "Apply Power Limit" for clarity
- Unused control card helper removed

## [3.6.0] - 2026-05-25

### Added
- **Window size persistence** - window dimensions auto-saved to config on resize, restored on startup

### Fixed
- **No more console window flashing** - PDH collector and taskkill/net commands now use CREATE_NO_WINDOW flag to suppress child process console windows
- **No more console window** - `windows_subsystem = "windows"` applied unconditionally, Pane launches as a pure GUI app
- Window size saved with 10px threshold to avoid config spam on minor resizes

### Removed
- System tray support (disabled due to Win32 message loop conflicts with eframe/winit - will revisit in future version)

## [3.4.0] - 2026-05-25

### Added
- **VRAM Headroom Calculator** - shows what LLM models fit in your available VRAM across GPUs, with quant sizes (Q4/Q5/Q8/FP16), KV cache estimates, max context predictions, and split-GPU indicators
- **Performance Snapshot Exporter** - one-click button generates clean text report of all system metrics (GPU, CPU, RAM, disk, processes), copy to clipboard or save to desktop. Ready for Reddit/GitHub/Discord
- 9 popular models in VRAM calculator (Llama 8B-405B, Qwen 14B-80B, Mixtral, DeepSeek V3)

## [3.3.0] - 2026-05-25

### Added
- **Config file persistence** - saves theme, refresh rate, selected GPU, window size, sidebar width, default panel to `%APPDATA%/pane/config.json` (Windows) or `~/.config/pane/config.json` (Linux/Mac)
- Settings auto-saved on theme change, loaded on startup
- Configurable refresh rate (default 500ms)

### Dependencies
- Added `serde`, `serde_json`, `dirs` for config persistence

## [3.2.0] - 2026-05-25

### Added
- **Real GPU power limit control** - Apply button sends SetPowerManagementLimit to GPU via NVML with success/error feedback
- **GPU command channel** - UI thread sends control commands to background metric thread safely
- **Per-process GPU metrics via PDH** - Windows Performance Counters fill in GPU% and VRAM columns in process table (vendor-agnostic, no admin)
- **Cross-platform CI** - GitHub Actions workflow builds for Windows x64, Linux x64, macOS x64 + ARM64 with auto-release on tags

### Changed
- GPU Control Apply button now functional (power limit only - fan/clocks noted as requiring NVAPI)
- Background thread processes GPU commands before each metric collection cycle

## [3.1.0] - 2026-05-25

### Added
- **GPU process table** - shows all processes using the selected GPU directly on the GPU panel (PID, name, type GFX/CMP, VRAM usage), sorted by VRAM descending
- **Process web search** - `?` button per process row opens Google search ("what is [name] Windows process") in default browser
- **Graceful close** - "Close" button sends WM_CLOSE/SIGTERM before resorting to force kill
- **Kill error feedback** - red banner with actual error message when kill fails (e.g. "Access denied - run Pane as administrator")
- **Success feedback** - green banner confirming "PID closed" or "PID killed" after successful action
- **Admin detection** - warning shown in kill confirmation when not running elevated
- **Status message system** - dismissible banners for action feedback across both GPU and Processes panels

### Fixed
- Kill confirmation no longer disappears on metric refresh (confirm_kill state preserved across updates)
- GPU process names resolved via sysinfo instead of showing raw PIDs
- NVML UsedGpuMemory enum properly handled (Used vs Unavailable)

## [3.0.0] - 2026-05-25

### Added
- **Native GUI** - egui/eframe GPU-accelerated window replaces TUI as default (TUI still available as `pane-tui`)
- **Dark / Light / System theme** - toggle in sidebar, full palette swap (text, backgrounds, charts, borders all adapt)
- **Real-time charts** - filled area graphs with grid lines, Y-axis labels, and max value indicators
- **Click-to-copy** - click any metric value to copy to clipboard, hover for tooltip with full precision
- **Copy confirmation** - tooltip shows "Copied!" on click
- **Sidebar navigation** - clean badge labels (GP, CP, ME, etc.), GPU selector, theme toggle, footer with author link
- **Loading screen** - spinner with branding while first metrics are collected
- **Background metric thread** - collection runs off the UI thread, no jank or frame drops
- **Process sort pills** - rounded pill buttons with accent highlight, ascending/descending toggle per column
- **Process kill button** - per-row `x` button with hover tooltip and red confirmation banner

### Changed
- Process filter redesigned with placeholder text and fixed-width input
- Chart rendering uses filled polygons with semi-transparent area under the line
- All panels use dynamic palette (`theme::p()`) instead of hardcoded dark constants
- Sort indicators changed from Unicode arrows to `^` / `v` ASCII for font compatibility
- GPU Control sliders are native egui sliders with suffix labels
- Sidebar icons replaced with monospace text badges for cross-platform font compatibility
- Binary size: ~5 MB (GUI) vs ~1 MB (TUI)

### Fixed
- Light mode text/icon contrast - all text properly dark on light backgrounds
- All clippy warnings resolved (zero warnings on release build)
- Chart grid lines and Y-axis labels respect active theme colors
- Process table kill button no longer uses broken Unicode glyph

## [2.0.0] - 2026-05-25

### Added
- **Dashboard overview** - all metrics at a glance: GPU cards, CPU, RAM, disk, network, top processes in one view
- **GPU Control panel** - fan speed, power limit, core/memory clock offset with interactive sliders
- **Braille sparklines** - 8x vertical resolution graphs across all panels
- **Process kill** - select process, press `k`, confirm with `y`
- **Process selection** - arrow key navigation with highlighted row
- **Dual GPU support** - both GPUs detected and displayed (RTX 5090 + RTX 4090)
- **GPU history tracking** - VRAM, temperature, and power draw histories
- **Panel jump keys** - `h` dashboard, `g` gpu, `c` cpu, `m` memory, `d` disk, `n` network, `p` processes, `x` gpu control
- **Kill confirmation** - red highlight + y/n prompt before killing a process

### Changed
- Default panel is now Dashboard (was GPU)
- UI overhauled: consistent dark borders, color gradients (green/yellow/red), Unicode sort arrows
- History buffer increased from 120 to 200 samples
- Status bar shows context-sensitive keybindings per panel

### Fixed
- All clippy warnings resolved
- Proper iterator usage for slider rendering
- `div_ceil` used instead of manual ceiling division

## [1.0.0] - 2026-05-25

### Added
- Project initialized
- Core architecture designed: ratatui + crossterm + sysinfo + nvml-wrapper + windows-rs
- Cross-platform foundation (Windows, Linux, macOS)
- Deep GPU metrics pipeline design (PDH, NVML, NVAPI, ADLX)
- README with full project vision, architecture, and roadmap
- MIT License

---

*Pane follows [Semantic Versioning](https://semver.org/).*
