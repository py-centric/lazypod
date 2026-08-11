# Lazypod

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)
[![Rust Quality](https://github.com/lazypod/lazypod/actions/workflows/quality.yml/badge.svg)](https://github.com/lazypod/lazypod/actions/workflows/quality.yml)

Lazypod is a modern, responsive Terminal User Interface (TUI) for container management, supporting both **Docker** and **Podman**. Inspired by the interface of [lazydocker](https://github.com/jesseduffield/lazydocker) and the [lazypodman](https://github.com/guillheu/lazypodman) project (which has unfortunately gone stale), it's written in Rust using `ratatui` and `crossterm`.

## Features
- Complete interactive TUI for Docker and Podman via `std::process::Command` integrations
- Multi-engine support: toggle between viewing Docker containers, Podman pods, or both
- View, start, stop, and remove running and stopped containers
- View images, pull from registry via interactive search, and spawn containers from images
- Direct Image Pulling and interactive configuration of Podman search registries
- Run Container popup with support for passing environment variables
- View volumes and networks
- Direct interactive shell dropping into running containers (`/bin/sh` or custom command)
- Embedded real-time log viewer for pods
- Contextual Help Bar depending on active tab and global Help Tooltips for keybindings

## Prerequisites
- **Podman**: Required for core functionality and Podman-specific features.
- **Docker** (Optional): Supported for viewing and managing Docker resources.
- **Rust/Cargo**: Required to build the project from source (`cargo version >= 1.75` recommended).

## Installation & Build

### Build from Source
To build the optimized release binary, ensure you have [Rust and Cargo](https://rustup.rs/) installed:

```bash
cargo build --release
```

The resulting binary will be located at `target/release/lazypod`.

### Execution
You can run the binary directly:
- **Linux/macOS**: `./target/release/lazypod`
- **Windows**: `.\target\release\lazypod.exe`

### Cross-Compilation (Optional)
Lazypod is built with `ratatui` and `crossterm`, making it cross-platform. To build for other architectures:
- **Windows**: `cargo build --release --target x86_64-pc-windows-msvc`
- **macOS (Intel)**: `cargo build --release --target x86_64-apple-darwin`
- **macOS (Apple Silicon)**: `cargo build --release --target aarch64-apple-darwin`

## Usage
Simply run:
```bash
cargo run
```
or use the optimized binary as shown above.

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) guide for information on local development, testing, and our required branching strategy for automated Semantic Versioning.

All contributors must sign the [Contributor License Agreement](.github/CLA.md) before their pull request can be merged.

## License

This project is **dual-licensed**:

### Open Source License

Licensed under the [GNU Affero General Public License v3.0 (AGPL-3.0)](LICENSE). You are free to use, modify, and distribute this software under the terms of the AGPL-3.0.

**Key AGPL-3.0 obligations:**
- If you modify and distribute the software, you must release your modifications under AGPL-3.0
- If you run a modified version as a network service, you must make the source code available to users of that service
- All derivative works must be licensed under AGPL-3.0

### Commercial License

If the AGPL-3.0 license terms do not work for your use case (e.g., you want to use Lazypod in proprietary software without open-sourcing your code), a commercial license is available.

**Commercial license benefits:**
- No obligation to open-source your modifications
- No network service disclosure requirements
- Priority support options available

To inquire about commercial licensing, contact: **licensing@lazypod.dev**

---

See the [LICENSE](LICENSE) file for the full AGPL-3.0 license text.
