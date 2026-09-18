# DLSS 5 STUDIO ⚡ v1.0.5

> **A blisteringly fast, low-memory utility built in pure native Rust to enable and unlock DLSS, Neural Reconstruction, and 4x Frame Generation across your PC games while preserving pristine graphical fidelity.**
>
> _Supports **GeForce RTX GPUs (20, 30, and 40-Series)** for DLSS upscaling and OptiScaler Pre-SR, **4x Multi-Frame Generation unlocking for RTX 40-Series**, and **experimental RTX 30-Series Frame Generation** in compatible x64/D3D12 games with native DLSS-G._

[![Version](https://img.shields.io/badge/version-1.0.5-orange.svg)](#)
[![Platform](<https://img.shields.io/badge/platform-Windows%2010%20%7C%2011%20(64--bit)-blue.svg>)](#)
[![Language](https://img.shields.io/badge/language-100%25%20Pure%20Rust-red.svg)](#)
[![i18n](https://img.shields.io/badge/i18n-14%20Languages-yellow.svg)](#)
[![Memory](https://img.shields.io/badge/RAM%20usage-~20%20MB-green.svg)](#)
[![Binary Size](https://img.shields.io/badge/Portable%20Exe-5.9%20MB-success.svg)](#)
[![License](https://img.shields.io/badge/license-MIT-purple.svg)](https://github.com/bookamp/dlss-studio/blob/main/LICENSE)

**DLSS 5 STUDIO** is a ground-up pure Rust desktop utility inspired by the sleek UI design and layout of the original [DLSS 5 Swapper](https://github.com/rakanki911/DLSS5-Swapper). It manages DLSS upgrades, Neural Reconstruction, and OptiScaler Pre-SR across game libraries, with **3x/4x Multi-Frame Generation unlocking for GeForce RTX 40-Series** and an **experimental DLSSG SM86 backend for compatible RTX 30-Series games**.

Built with [Dioxus](https://dioxuslabs.com/) and direct Win32 APIs, it eliminates heavy web-wrapper and Electron stacks—launching in under 200ms and consuming under 20 MB of RAM.

<p align="center">
  <img src="assets/preview-dashboard.jpg" alt="DLSS 5 Studio Dashboard Overview" width="850">
</p>

---

## 🌟 Key Features

### 1. ⚡ Frame Generation for RTX 30 & RTX 40

- **Bypass RTX 50-Series Driver Locks**: NVIDIA officially restricts 3x and 4x Multi-Frame Generation in drivers to RTX 50-Series hardware. DLSS 5 STUDIO unlocks 3x and 4x multipliers on GeForce RTX 40-Series GPUs.
- **Native DLSS-G Interception**: Leverages RenoDX hook add-ons (`renodx-mfgunlock.addon64`) to intercept Streamline Frame Generation contracts on games with native Frame Generation code (`sl.dlss_g.dll`, `nvngx_dlssg.dll`).
- **Honest Hardware & API Gating**: Automatically detects whether a game's engine has native Frame Generation or only DLSS Super Resolution (e.g. _Baldur's Gate 3_), and strictly gates MFG availability on DirectX 11 executables (`bg3_dx11.exe`) where Streamline Frame Generation is unsupported.
- **Experimental RTX 30 Backend**: Opt-in DLSSG SM86 on Windows x64/D3D12 games with original DLSS-G integration. Choose a 2x, 3x, or 4x ceiling; the game controls the actual Frame Generation request. Native, Feeder, and OptiScaler routes share the same eligibility checks.
- **Verified External Runtime**: Downloads a pinned SM86 version with SHA-256 verification and upstream notices. Installation checks all four proxy slots and rejects conflicts before changing game files. See [RTX 30 compatibility and setup](#-rtx-30-frame-generation).

<p align="center">
  <img src="assets/preview-cyberpunk-mfg.png" alt="Cyberpunk 2077 4x Multi-Frame Generation Unlock" width="620">
</p>

### 2. 🔬 OptiScaler DLSS-NR & Pre-SR Multipass

- **OptiScaler Neural Reconstruction Pipeline**: Route graphics through OptiScaler's open-source multi-vendor wrapper (`nvngx.dll` / `dxgi.dll`).
- **Pre-SR Multipass Clarity**: Enables multi-pass neural reconstruction (`1x`, `2x`, or `3x` passes) for dramatic clarity, sharpness, and temporal stability enhancements.
- **GPU-Aware Frame Generation**: Uses standalone RTXMFG on compatible RTX 40 configurations. On eligible RTX 30 configurations, SM86 supplies external Frame Generation while OptiScaler retains the game's Streamline stack; this combination still requires real-game validation.

<p align="center">
  <img src="assets/preview-bg3-presr.png" alt="Baldur's Gate 3 OptiScaler Pre-SR Multipass Clarity" width="620">
</p>

### 3. 🎯 Flexible Rendering Backends & Routes

- **ReShade Backend**:
  - **`Native DLSS (RenoDX)`**: For DirectX 12 games with native DLSS pipelines. Hooks into D3D12 NGX vtables and enables 4x MFG unlock.
  - **`DLSS 5 Feeder`**: Dedicated frame interception route for non-DLSS titles or games running on DirectX 11, Vulkan, OpenGL, or legacy pre-DirectX 10 APIs (DirectX 8 and DirectX 9 via automated dgVoodoo 2 translation with 32-bit LAA memory support) (`dlss5-feed.addon64`, `DLSS5_Feed.fx`, `vort_Motion.fx`).
- **OptiScaler Backend**:
  - **`OptiScaler DLSS-NR`**: Full neural reconstruction with Pre-SR multipass. Automatically restricted on titles lacking native depth and motion vectors.
- **Seamless Cross-Route Hot-Swapping**: Switch freely between ReShade (Native/Feeder) and OptiScaler with a single click. DLSS 5 STUDIO automatically unregisters Vulkan implicit layers, removes conflicting proxy DLLs, and deploys the new payload while carrying forward the original vanilla game backups.

### 4. 🚀 Universal Multi-Store Game Scanner

Scans and organizes your games automatically without manual configuration:

- **Steam**: Resolves library roots from `SteamPath` registry and `libraryfolders.vdf`, parses `appmanifest_<id>.acf`, and downloads official 600x900 vertical box art from Steam CDN.
- **Xbox Game Pass / Microsoft Store**: Queries `GamingServices` package repository and scans `XboxGames` drive roots. Parses GDK `MicrosoftGame.config` and `AppxManifest.xml` to bypass launcher wrappers (`gamelaunchhelper.exe`), resolves authentic 64-bit executables, and extracts high-resolution logos directly from package assets.
- **Epic Games Store**: Discovers installed titles by parsing `%PROGRAMDATA%\Epic\...\Manifests\*.item` manifests.
- **GOG Galaxy**: Inspects `GOG.com\Games` registry trees and `goggame-*.info` playtasks.
- **Custom Folders & Manual Executables**: Add any custom game folder or executable with instant automatic Steam CDN box art resolution, smart nested directory climbing (e.g. `bin/x64` auto-resolving to the authentic parent title), fuzzy title boundary splitting, on-demand artwork refresh, and persistent caching.

### 5. 🛡️ Bulletproof Backup, Rollback & Process Safety

- **Atomic Rollback Journals**: Every modification automatically creates a snapshot in `_DLSS5_Backup/originals/` before touching any game files.
- **Vanilla Backup Continuity**: Switching between routes carries forward the genuine unmodded game files through arbitrary successive swaps.
- **SM86 Ownership Checks**: Journals the backend, proxies, configuration, and notices. Failed SM86 installs attempt rollback; restore and backend switches refuse to delete proxies replaced by another mod after a completed installation.
- **Restore Originals**: Restores authentic vanilla binaries with a single click and archives the backup manifest.
- **Clean Untracked Mods**: Purges leftover proxy DLLs (`dxgi.dll`, `OptiScaler.dll`, `ReShade64.dll`, `.addon64`) up to 4 directory levels deep without risking original game files.
- **Process Guarding**: Inspects running processes via native Win32 `Toolhelp32` snapshots, blocking mod deployment or restoration if the game is running.
- **Anti-Cheat Detection**: Detects EasyAntiCheat, BattlEye, and Vanguard, warning you before touching protected titles.

### 6. 🌍 Multilingual Localization (14 Languages)

- **14 Supported Languages**: English, Deutsch (German), Español (Spanish), Français (French), Italiano (Italian), Português (Portuguese), Русский (Russian), 简体中文 (Simplified Chinese), 日本語 (Japanese), 한국어 (Korean), Polski (Polish), Türkçe (Turkish), العربية (Arabic with full RTL layout support), and हिन्दी (Hindi).
- **Reactive Dynamic Switching**: Instantly switch languages anytime from the header selector or Settings. All views, sheets, specs, action badges, and tooltips update in real-time with zero app restart.
- **Structured Activity Logging Engine**: Activity log entries use tokenized templates (`@{key|...}`), allowing the in-app terminal to dynamically translate logs into the selected language while keeping on-disk diagnostics (`dlss-studio.log`) in standard English for seamless GitHub issue reporting.
- **Localized History & Tooltips**: Fully translated modification history tables, dynamic change counts (`0 replaced, 5 added`), action badges, and localized play button tooltips (`Launch {game}`).
- **Experimental UI**: The new Frame Generation backend labels, SM86 instructions, and unsupported-state explanations currently use English.

### 7. 🪶 100% Pure Rust Performance

| Metric                   | Traditional Web / Electron Apps | **DLSS 5 STUDIO**            | Advantage                      |
| :----------------------- | :------------------------------ | :--------------------------- | :----------------------------- |
| **Idle Memory (RAM)**    | 350 MB – 600 MB                 | **~20 MB**                   | **95% less RAM**               |
| **Executable Size**      | 120 MB – 250 MB                 | **5.9 MB**                   | **97% smaller**                |
| **Startup Time**         | 2.5s – 6.0s                     | **< 200ms**                  | **Instantaneous**              |
| **Window Dragging**      | Emulated / CSS Drag Regions     | **Native Win32 `HTCAPTION`** | Fluid Tao window management    |
| **Runtime Dependencies** | Node.js, Chromium, PowerShell   | **None (Pure Win32)**        | Standalone portable executable |

---

## 💻 System Requirements

- **Operating System**: Windows 10 (1903+) or Windows 11 (64-bit)
- **Graphics Card**:
  - Any DirectX 11, DirectX 12, or Vulkan compatible GPU.
  - _For DLSS Super Resolution_: NVIDIA GeForce RTX 20/30/40/50-Series.
  - _For 4x Multi-Frame Generation Unlock_: NVIDIA GeForce RTX 40-Series (Ada Lovelace) GPU.
  - _For experimental DLSSG SM86 Frame Generation_: NVIDIA GeForce RTX 30-Series (Ampere), a 64-bit D3D12 executable, and the game's original DLSS-G integration. See [RTX 30 setup](docs/RTX30-SM86.md).
- **Storage**: ~15 MB free space.
  - The optional SM86 proxy set requires about 115 MiB in the component cache and another 115 MiB per installed game, plus the upstream runtime's extracted bundle cache.

---

## 📦 Installation & Download

The release links below point to the original upstream project and **do not include this fork's RTX 30 implementation**. To try this fork, build its source on Windows with `cargo build --release --locked`; the executable is written to `target/release/dlss-studio.exe`.

### Standalone Setup / Installer (Recommended)

- Run **`dlss-studio-v<version>-setup.exe`** for standard Windows installation with Start Menu and Desktop shortcuts.
- **Seamless In-Place Updates**: Automatically detects previous installations, displaying an **"Update"** flow that safely terminates running application instances before copying files, while preserving all user libraries, settings, and custom folders.

### Portable Executable

1. Download **`dlss-studio-v<version>-portable.exe`** from the [Releases](https://github.com/bookamp/dlss-studio/releases) page.
2. Run from anywhere—no installation required.

---

## 🛠️ How to Use

1. **Launch DLSS 5 STUDIO**: Your installed games across Steam, Xbox Game Pass, Epic Games, and GOG will populate automatically.
2. **Select a Game**: Click on any game card to open its detail sheet.
3. **Choose Your Configuration**:
   - Select your **Rendering backend** (`ReShade` or `OptiScaler DLSS-NR`).
   - If using ReShade, select your **Installation route** (`Native DLSS (RenoDX)` or `DLSS 5 Feeder`).
   - If supported by your hardware and game engine, toggle **`Pre-SR Multipass`** (`1x`, `2x`, or `3x` passes) or **`Frame Generation`**. The FG control identifies the selected backend and explains unsupported configurations.
   - For RTX 30, opt into **`DLSSG SM86 (experimental, RTX 30)`** and choose a **2x, 3x, or 4x ceiling**. The installer downloads and verifies the runtime when selected.
4. **Click "Install DLSS 5"** (ensuring the game is closed).
5. **Launch Your Game**: Launch via the "Launch Game" button or your regular launcher. For SM86, enable DLSS Frame Generation in the game's own settings. To revert, close the game and click **"Restore originals"**.

---

## ⚙️ Settings & System Tray

- **Run in Background**: Minimizes to the Windows System Notification Area (System Tray) when clicking the window close button (`X`).
- **Launch at Windows Startup**: Automatically registers in `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` to start in the background when Windows boots.
- **Industrial Rust Theme**: Toggle between sleek modern dark/light glassmorphism and textured Rust industrial metal finishes.

---

## 🧪 RTX 30 Frame Generation

This fork implements [dlssg_for_sm86](https://github.com/sdli1995/dlssg_for_sm86) as an optional external backend. **It remains experimental: automated deployment tests pass, but physical RTX 30 hardware and real-game behavior have not been validated.**

### Compatibility and backend selection

| GPU and game | Rendering route | Managed Frame Generation backend |
| :--- | :--- | :--- |
| RTX 30, x64, D3D12, original native DLSS-G | Native DLSS / RenoDX, Feeder, or OptiScaler | DLSSG SM86, with 2x/3x/4x ceilings |
| RTX 40, supported game | Native DLSS / RenoDX or Feeder | RenoDX Ada MFG |
| RTX 40, supported game | OptiScaler | Standalone RTXMFG |
| RTX 30 with DX11, Vulkan, 32-bit, or no native DLSS-G | Any | SM86 unavailable |
| RTX 20, RTX 50, or unknown GPU | Any | Managed FG unlock unavailable |

SM86 requires the game's original `nvngx_dlssg.dll` or `sl.dlss_g.dll`. Super Resolution's `sl.dlss.dll` and files added by Studio do not establish native FG support. Choosing Feeder does not remove this requirement. Unknown or ambiguous GPU/API detection leaves the feature unavailable.

The 2x/3x/4x UI ceilings map to `MaxGeneratedFrames=1/2/3`. Studio configures optimized mode, the automatic compatibility preset, bundled runtime, and ordinary logging while preserving unrelated INI settings. A ceiling does not force the game to request that multiplier.

### Installation, conflicts, and restore

- SM86 0.3.3 is pinned to upstream commit `5e79459c2d521f8c3276ce9ee02342c0e2686982`. Each DLL and the upstream notices are SHA-256 verified in a separate versioned cache.
- Installation requires the complete, distinct upstream proxy set: `version.dll`, `winmm.dll`, `dbghelp.dll`, and `dinput8.dll`. Foreign files block installation. A previous Studio-managed backend may be replaced only when its bytes match the expected payload.
- Only one managed FG backend is installed at a time. SM86 does not use `dxgi.dll` or `d3d12.dll` as its proxy.
- Backend changes, reinstallations, and **Restore originals** retain original backups and track introduced files. If a completed installation's SM86 proxy has changed externally, resolve the conflict before switching or restoring.
- Runtime-created logs and the upstream user-level bundle cache are retained; Studio does not own those directories.

### Validation and limitations

Windows CI checks all targets, runs unit and regression tests, and separately downloads the verified payload to exercise installation, reinstallation, rendering/backend switches, conflict handling, and restoration in a synthetic game directory. The test copies DLLs as data and does not execute them.

Real-game FG behavior, image quality, performance, and proxy loading still need physical RTX 30 validation, with RTX 40 regression testing also pending. Single-proxy profiles, DX11, Vulkan, RTX 20, 6x, graphics-proxy fallbacks, and advanced tuning are not implemented.

See [RTX 30 setup and validation](docs/RTX30-SM86.md) for detailed instructions, recovery behavior, and the manual test matrix.

### Third-party runtime

Studio downloads SM86 independently and retains upstream third-party notices verbatim. No SM86 DLLs, NVIDIA runtime, or upstream GPL source are committed to this repository or bundled into Studio's executable. Packaging or redistributing third-party runtime resources requires review of the applicable upstream and NVIDIA terms.

---

## 📚 Acknowledgements & Third-Party Components

- **DLSSG for SM86**: Experimental external DLSS-G compatibility runtime for RTX 30-Series in this fork ([sdli1995/dlssg_for_sm86](https://github.com/sdli1995/dlssg_for_sm86)). Downloaded from a pinned upstream commit with its third-party notices; not bundled into Studio's executable.
- **DLSS 5 Swapper**: Original UI layout, visual design, and desktop concept ([rakanki911/DLSS5-Swapper](https://github.com/rakanki911/DLSS5-Swapper)).
- **dgVoodoo 2**: Legacy DirectX 1–9 to Direct3D 11/12 graphics wrapper by **Dege** ([dege-diosg/dgVoodoo2](https://github.com/dege-diosg/dgVoodoo2)).
- **DLSS 5 Feeder**: Universal ReShade frame interception pipeline for non-DLSS and non-DX12 titles by **jlrouzies-fr** ([jlrouzies-fr/DLSS5-Feeder](https://github.com/jlrouzies-fr/DLSS5-Feeder)).
- **MFGAdaUnlock-RenoDx**: Streamline Frame Generation 4x unlocker add-on for GeForce RTX 40-Series GPUs by **mavismmg** ([mavismmg/MFGAdaUnlock-RenoDx](https://github.com/mavismmg/MFGAdaUnlock-RenoDx)).
- **vort_Shaders & vort_Motion**: Temporal optical flow and motion vector calculation shaders by **vortigern11** ([vortigern11/vort_Shaders](https://github.com/vortigern11/vort_Shaders)).
- **OptiScaler DLSS-NR & Pre-SR**: Specialized neural reconstruction & multi-pass wrapper developed by **wilsjo2** ([OptiScaler-DLSSNR-PreSR-Multipass](https://github.com/wilsjo2/OptiScaler-DLSSNR-PreSR-Multipass)), based on upstream [OptiScaler](https://github.com/optiscaler/OptiScaler) by **cdozdil** (Nitec).
- **RenoDX & Frame Generation Mods**: HDR pipeline, Streamline contract hooking, and frame interception runtimes developed by **Otis_Inf**, **ShortFuse**, and the **RenoDX** project team.
- **ReShade**: Advanced generic post-processing injector, swapchain hook, and native C++ Add-on framework by **crosire** ([crosire/reshade](https://github.com/crosire/reshade) and [crosire/reshade-shaders](https://github.com/crosire/reshade-shaders)).
- **NVIDIA Streamline**: Cross-vendor open-source interposer framework for DLSS and Frame Generation ([NVIDIA/Streamline](https://github.com/NVIDIA/Streamline)).
- **Rust Ecosystem**: Built using [Dioxus](https://dioxuslabs.com/), [mimalloc](https://github.com/microsoft/mimalloc), [pelite](https://github.com/CasualX/pelite), [winres](https://github.com/mxre/winres), and native Win32 APIs.

---

## 📜 License

This project is licensed under the MIT License. See [LICENSE](https://github.com/bookamp/dlss-studio/blob/main/LICENSE) for details.
