# 🛡️ RsBorg (ReBorg)

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust%202024-orange?style=for-the-badge&logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/TUI-Ratatui%200.30-blue?style=for-the-badge" alt="Ratatui">
  <img src="https://img.shields.io/badge/Engine-BorgBackup-blueviolet?style=for-the-badge" alt="BorgBackup">
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/i18n-English%20%7C%20Portuguese-yellow?style=for-the-badge" alt="i18n">
</p>

<p align="center">
  <strong>RsBorg</strong> is a modern, fast, and ergonomic Terminal User Interface (TUI) for <a href="https://www.borgbackup.org/">BorgBackup</a>, written in pure Rust.<br>
  Engineered to simplify backup creation, restoration, visual diffing, integrity auditing, and Linux system automation without command-line friction.
</p>

---

## ✨ Features at a Glance

- ⚡ **Asynchronous & Non-Blocking Engine**:
  - Heavy operations (backup creation, archive comparison, repository check, pruning, and mounting) execute asynchronously in dedicated background threads.
  - Live progress telemetry with streaming throughput, current file tracking, and real-time status updates.
- 🌐 **Built-in Bilingual Support (i18n)**:
  - Instant on-the-fly toggling between **English** and **Portuguese** by pressing `[L]`.
- 📁 **Interactive Hierarchical File Selector**:
  - Intuitive directory tree navigation with multi-state selection:
    - `[X]` Explicitly Included
    - `[+]` Inherited Included
    - `[-]` Explicitly Excluded
    - `[!]` Inherited Excluded
    - `[ ]` Neutral
  - Toggle dotfiles (hidden files) anytime with `Ctrl+H`.
- 🔍 **Deep Archive Inspection & Instant Restoration**:
  - Explore archive contents inside any historical snapshot (`[i]`).
  - Search and filter files in real time (`[/]`).
  - Mount snapshots on-demand via FUSE (`[m]` / `[u]`) or extract files directly to a designated directory (`[r]`).
- ⚖️ **Visual Version Comparison & Diffing (`borg diff`)**:
  - Pick any two archives to inspect added, modified, or removed files (`[f]`).
  - Stateful table with automatic scrolling, size variation deltas, and optional deep content-level comparisons.
- 🧹 **Retention Policies & Safe Pruning (`borg prune`)**:
  - Define custom retention rules (*keep daily*, *weekly*, *monthly*, *yearly*) (`[p]`).
  - Built-in **Dry-run simulation** to preview which archives will be kept or deleted before applying changes.
- 🩺 **Integrity Auditing & Repair (`borg check`)**:
  - Comprehensive repository and archive verification wizard (`[v]`).
  - Automated repair option (`--repair`) and scrollable log viewer with error/warning counters.
- ⏰ **Backup Sets (Profiles) & System Automation (`systemd` & `cron`)**:
  - Save reusable profiles with custom inclusions, exclusions, and compression algorithms (`lz4`, `zstd,3`, `zstd,9`, `zlib,6`, `none`) (`[b]`).
  - **Sequential Smart Naming**: Automated naming patterns with counters and timestamps: `{SET_NAME}_#{COUNTER}_{DATE}` (e.g., `DEV_BACKUP_#1_2026-09-19_18-20`).
  - **Linux Automation Generator**:
    - Generates systemd user `.service` and `.timer` files. Install and reload the user daemon with a single keystroke (`[i]`).
    - Generates ready-to-use lines for `crontab -e`.
    - Implements robust escaping against shell injection, variable expansion, and systemd/cron format specifier collisions.

---

## 🚀 Installation & Prerequisites

### Prerequisites

1. **Rust Toolchain**: Rust 1.85+ (Rust 2024 Edition).
2. **BorgBackup**: `borg` version 1.2+ installed and accessible in your system `$PATH`.

```bash
# Ubuntu / Debian
sudo apt install borgbackup

# Fedora / RHEL
sudo dnf install borgbackup

# Arch Linux
sudo pacman -S borg
```

### Building from Source

Clone the repository and build using Cargo in release mode:

```bash
git clone https://github.com/GabrielSantarem/rsborg.git
cd rsborg
cargo build --release
```

The compiled binary will be located at `target/release/reborg`. You can install it into your system's binary path:

```bash
sudo cp target/release/reborg /usr/local/bin/rsborg
```

---

## ⌨️ Keyboard Shortcuts Cheat Sheet

### Global & General Navigation
| Key | Action |
| :--- | :--- |
| `↑` / `↓` or `k` / `j` | Move selection up / down |
| `Enter` | Enter directory / Confirm modal |
| `Esc` / `q` | Go back / Close modal / Exit |
| `L` | Toggle Language (English ⇄ Portuguese) |
| `F1` / `?` | Toggle Help screen |
| `Tab` | Cycle focus between form inputs or tabs |

### Main Dashboard (Snapshots List)
| Key | Action |
| :--- | :--- |
| `n` | Create a New Backup (Opens interactive file picker) |
| `i` | Inspect files inside the selected archive |
| `r` | Restore selected archive |
| `d` | Delete selected archive |
| `m` / `u` | Mount archive via FUSE / Unmount |
| `f` | Diff / Compare two archives |
| `p` | Configure Prune retention policy & run simulation |
| `v` | Check repository / archive integrity (`borg check`) |
| `b` | Manage Backup Sets (Profiles) & Automation |
| `R` | Switch active repository |
| `+` | Add / Register a new repository |

### File Browser & Profile Setup
| Key | Action |
| :--- | :--- |
| `Space` | Cycle selection: Neutral `[ ]` → Include `[X]` → Exclude `[-]` |
| `h` / `l` or `←` / `→` | Parent directory / Open directory |
| `Ctrl+H` | Toggle hidden files (dotfiles) |
| `Tab` | Switch focus between file tree and configuration options |

---

## 🛠️ Background Automation Guide

RsBorg simplifies background backups through native automation generation:

1. In the main window, press **`[b]`** to open **Backup Profiles**.
2. Press **`[a]`** to create a new profile (e.g., `DAILY_DOCS`):
   - Select the target folders and files.
   - Choose compression (e.g., `zstd,3` for a balanced ratio).
   - Set the schedule frequency (`Daily`, `Weekly`, `Hourly`).
3. Press **`[s]`** on your saved profile to access the **Automation Generator**:
   - **Systemd User Timer**: Press **`[i]`** to automatically write `rsborg-<name>.service` and `rsborg-<name>.timer` to `~/.config/systemd/user/` and reload systemd.
     To enable the timer immediately:
     ```bash
     systemctl --user enable --now rsborg-daily_docs.timer
     ```
   - **Crontab**: Switch to the Crontab tab to copy a clean, POSIX-safe cron command with proper escaping for your `crontab -e`.

---

## 🏗️ Architecture Overview

The codebase is organized into clean, modular components:

```
src/
├── app/          # Core state management, thread dispatching, and view routing
│   ├── mod.rs    # App state machine and orchestration logic
│   └── state.rs  # View enums, wizard states, and background thread payloads
├── borg.rs       # BorgBackup CLI wrapper, streaming stdout parser, and JSON deserializer
├── browser.rs    # Hierarchical file system browser with inheritance mechanics
├── config.rs     # Configuration persistence, profiles, and automation script generators
├── events.rs     # Crossterm event processing and keyboard navigation routing
├── i18n.rs       # Bilingual translation system (EN/PT)
└── ui/           # Modular Ratatui rendering components
    ├── backup.rs   # Backup creation wizard & live progress gauges
    ├── diff.rs     # Archive version comparison & diff table viewer
    ├── profiles.rs # Profile manager & automation generator modals
    └── ...
```

---

## 🧪 Testing & Code Quality

RsBorg maintains an extensive test suite covering the Borg engine, state machines, file tree inheritance, and configuration serialization:

```bash
# Run all unit and integration tests
cargo test

# Run linter checks
cargo clippy -- -D warnings
```

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
