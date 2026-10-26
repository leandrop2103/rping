# Technical Documentation and Complete Guide for rping

> A modern, high-performance, colored-output Rust reimplementation of the classic `ping` utility (Mike Muuss / iputils).

---

## Table of Contents
1. [Overview & Architecture](#1-overview--architecture)
2. [Prerequisites](#2-prerequisites)
3. [Step-by-Step: Build & Compilation](#3-step-by-step-build--compilation)
4. [Step-by-Step: Installation](#4-step-by-step-installation)
5. [Step-by-Step: Usage & CLI Options](#5-step-by-step-usage--cli-options)
6. [Internal Code Architecture (Modules)](#6-internal-code-architecture-modules)
7. [Technical Mechanisms & Optimizations](#7-technical-mechanisms--optimizations)
8. [Testing, Benchmarks & Quality Assurance](#8-testing-benchmarks--quality-assurance)
9. [Publishing & Distribution (WinGet & GitHub Releases)](#9-publishing--distribution-winget--github-releases)

---

## 1. Overview & Architecture

Standard ping utilities on Windows and Linux often present key limitations:
- Standard Windows implementations require elevated administrator privileges when using traditional raw sockets (`SOCK_RAW`).
- Native Windows ping output is monochrome and rigid.
- Discrepancies exist between command-line flags on Windows (`-n`, `-l`) and Linux (`-c`, `-s`).

**rping** addresses all these pain points:
- **No UAC Elevation Required**: On Windows, it leverages the official Windows IP Helper API (`iphlpapi.dll`) through safe FFI wrappers (`IcmpCreateFile`, `IcmpSendEcho` for IPv4 and `Icmp6CreateFile`, `Icmp6SendEcho2` for IPv6). Standard non-admin users can execute it seamlessly.
- **Hybrid Syntax**: Accepts both Linux conventions (`-c`, `-s`) and Windows conventions (`-n`, `-l`).
- **Modern Colored Terminal Output**: Real-time RTT latency color-coding (green, yellow, and red ranges) with automatic Virtual Terminal Processing enablement in Windows console environments.
- **Graceful Signal Handling**: `Ctrl+C` interruptions are trapped cleanly to print full round-trip statistics before termination.

---

## 2. Prerequisites

To build and develop the project from source:
- **Rust Toolchain**: Rust 1.70 or newer (`rustup default stable`).
- **Target**: `x86_64-pc-windows-msvc` (or `x86_64-unknown-linux-gnu` on Linux).
- **Git**: For version control.
- **PowerShell** or **CMD**: For running provided automation build scripts.

---

## 3. Step-by-Step: Build & Compilation

The project includes pre-configured automation scripts to simplify compilation without having to memorize complex Cargo release parameters.

### Method 1: Via PowerShell Script (Recommended on Windows)
Open PowerShell in the project directory and run:
```powershell
.\build.ps1
```
This script executes:
1. `cargo build --release` with all performance optimization flags.
2. Copies the produced binary directly to the project root as `rping.exe`.
3. Creates a companion copy named `ping.exe`.

### Method 2: Via Batch Script (Windows CMD)
```cmd
build.cmd
```

### Method 3: Direct Cargo Build
```powershell
# Compile release build with all optimizations
cargo build --release

# Optional: copy directly to root
Copy-Item target\release\rping.exe .\rping.exe -Force
Copy-Item target\release\rping.exe .\ping.exe -Force
```

---

## 4. Step-by-Step: Installation

### Option A: Official Installation via WinGet (Windows Package Manager)
Once published and approved:
```powershell
winget install leandrop2103.rping
```
WinGet automatically pulls the standalone portable binary and links it to your Windows system PATH.

### Option B: Automated Local Installer
The repository includes an `instalar.cmd` script that requests UAC elevation and copies the binary into a system directory (`C:\Windows` or system root):
```cmd
instalar.cmd
```

### Option C: Manual Addition to User PATH
1. Create a dedicated directory, e.g., `C:\bin`.
2. Add `C:\bin` to your environment `PATH`.
3. Move `rping.exe` into `C:\bin`.

---

## 5. Step-by-Step: Usage & CLI Options

### Basic Syntax
```text
rping [OPTIONS] <TARGET>
```

### CLI Parameters & Flags
| Option | Windows Alias | Description | Default |
|---|---|---|---|
| `-c, --count <COUNT>` | `-n <COUNT>` | Number of ECHO_REQUEST packets to send | Infinite |
| `-i, --interval <SECS>` | — | Pacing interval between requests in seconds | `1.0` |
| `-s, --size <BYTES>` | `-l <BYTES>` | Payload data size in bytes | `56` bytes |
| `-T, --ttl <VAL>` | — | IP Time-To-Live hop count | `64` |
| `-W, --timeout <SECS>` | — | Response timeout per ping in seconds | `1.0` s |
| `-w, --deadline <SECS>` | — | Overall deadline timeout regardless of count | None |
| `-t, --timestamp` | — | Print microsecond-accurate UNIX epoch timestamp | Disabled |
| `-th, --th` | — | Print human-friendly timestamp (`dd/mm-hh:mm:ss`) | Disabled |
| `-4, --ipv4` | — | Force IPv4 address resolution | Auto |
| `-6, --ipv6` | — | Force IPv6 address resolution | Auto |
| `-a, --audible` | — | Emit terminal beep on each received reply | Disabled |
| `-q, --quiet` | — | Quiet mode (prints only header and final stats) | Disabled |
| `--no-color` | — | Disable terminal ANSI color formatting | Auto |

### Usage Examples
```powershell
# 1. Continuous default ping
.\rping.exe google.com

# 2. Count limit with human-readable timestamp
.\rping.exe -th -c 5 1.1.1.1

# 3. Traditional Windows flags (-n count, -l payload size)
.\ping.exe -n 4 -l 64 8.8.8.8

# 4. Enforce IPv6 with custom payload size
.\rping.exe -6 -s 128 cloudflare.com
```

---

## 6. Internal Code Architecture (Modules)

The source tree in [`src`](file:///d:/rust/ping/src) is cleanly separated into specialized modules:

```text
src/
├── app.rs          # Application lifecycle coordinator: ping loop, timers, signals
├── cli.rs          # Argument parsing and validation using `clap`
├── error.rs        # Centralized error types (`PingError` enum) and conversions
├── lib.rs          # Root library module exporting public structures
├── main.rs         # Entry point, logger setup, and top-level runner
├── stats.rs        # Metrics collector: min, max, avg RTT, packet loss, and mdev calculation
├── display/        # Formatting and presentation layer
│   ├── mod.rs      # Formats request banners, reply lines, and final summary
│   └── style.rs    # ANSI styling logic and NO_COLOR standard detection
└── engine/         # ICMP network abstraction layer
    ├── mod.rs      # Unified `PingEngine` trait definition
    ├── unix.rs     # Linux/Unix implementation using datagram/raw sockets
    └── windows.rs  # Windows implementation via IP Helper API (IcmpSendEcho / Icmp6SendEcho2)
```

---

## 7. Technical Mechanisms & Optimizations

1. **Safe Windows FFI (`engine/windows.rs`)**:
   - Bypasses `SOCK_RAW` limitations to avoid administrator elevation requirements.
   - Binds `IcmpCreateFile` and `IcmpSendEcho` with memory-aligned structures for `ICMP_ECHO_REPLY` and `IP_OPTION_INFORMATION`.
2. **Zero-Allocation Hot-Path**:
   - Send and receive packet buffers are allocated once before the execution loop and reused across subsequent iterations.
3. **High-Precision Mean Deviation (`mdev`) Calculation**:
   - Implemented in [`src/stats.rs`](file:///d:/rust/ping/src/stats.rs) using an incremental online variance algorithm leveraging `mul_add` (FMA).
4. **Binary Optimization Settings (`Cargo.toml`)**:
   - `opt-level = "z"` (optimized for binary size and CPU cache locality).
   - `lto = "fat"` (Full cross-crate link-time optimization).
   - `codegen-units = 1` (maximum interprocedural analysis).
   - `panic = "abort"` (eliminates stack unwinding bloat).
   - `strip = true` (removes debug symbols from release output).
   - `crt-static` (statically links the C runtime, making the binary self-contained).

---

## 8. Testing, Benchmarks & Quality Assurance

To ensure software correctness, reliability, and performance:

```powershell
# Run all unit and integration tests
cargo test

# Execute strict Rust linter (clippy)
cargo clippy --all-targets --all-features -- -D warnings

# Execute criterion benchmark suite
cargo bench
```

---

## 9. Publishing & Distribution (WinGet & GitHub Releases)

The distribution workflow consists of:
1. **GitHub Releases**:
   - Every released tag (e.g., `v0.1.0`) packages the standalone binary inside a `.zip` archive alongside its SHA-256 checksum.
2. **WinGet Manifest**:
   - Located under `manifests/l/leandrop2103/rping/0.1.0/`.
   - Defines package metadata (`leandrop2103.rping`), the release asset URL, and `portable` installer type.
   - Submitted as a pull request to the official [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) repository.
