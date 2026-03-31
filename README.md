# Lazypod

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
- **Rust/Cargo**: Required to build the project from source (`cargo version >= 1.70` recommended).

## Usage
Simply run:
```bash
cargo run
```

## License

This project is licensed under the [MIT License](LICENSE), an [OSI-approved](https://opensource.org/licenses/MIT) open-source license.
