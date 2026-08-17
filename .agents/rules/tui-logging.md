---
description: "Guidelines for diagnostic logging in TUI applications"
globs: ["src/**/*.rs"]
always_on: true
---

# TUI Diagnostic Logging Standard

1. **Prevent Screen Corruption**:
   - Never write diagnostic or debug logging to `stdout` or unredirected `stderr` while Ratatui/Crossterm alternate screen mode is active.
2. **Persistent Log File Support**:
   - Always support an environment variable (e.g. `LAZYPOD_LOG_FILE`) allowing users to stream logs to a persistent file.
   - Disable ANSI color escape codes (`with_ansi(false)`) when writing to files to ensure human-readable logs post-exit.
