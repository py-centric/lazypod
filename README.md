# Lazypod

Lazypod is a modern, responsive Terminal User Interface (TUI) for Podman, inspired by the interface of `lazydocker`. Written in Rust using `ratatui` and `crossterm`.

## Features
- Complete interactive TUI for Podman via `std::process::Command` integrations
- View, start, stop, and remove running and stopped containers
- View images, pull from registry via interactive search, and spawn containers from images
- View volumes and networks
- Direct interactive `/bin/sh` shell dropping into running containers
- Embedded real-time log viewer for pods

## Usage
Simply run:
```bash
cargo run
```
