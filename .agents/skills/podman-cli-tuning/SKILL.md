---
name: podman-cli-tuning
description: >-
  Best practices for fast and resilient CLI interactions with Podman and Docker
  engines in Lazypod.
---

# Podman CLI Tuning & Performance Guide

## 1. JSON Output Formatting: NDJSON vs. JSON Array
- **Array Format (`--format json`)**: Triggers deep metadata resolution on all items, which causes multi-minute execution times when querying 500+ image layers (`podman images -a`).
- **Template Format (`--format "{{json .}}"`)**: Emits newline-delimited JSON (NDJSON) in sub-second time (< 0.4s for 600+ items).
- **Rule**: Use `--format "{{json .}}"` for image and container inventories.

## 2. Multi-Engine Concurrency & Timeouts
- Query engines concurrently via `tokio::spawn` and `tokio::join!`.
- Enforce explicit command timeouts (`run_cmd_with_timeout`) of 10-15s to avoid blocking when an engine daemon is unresponsive or hung.
