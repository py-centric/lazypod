// Copyright (C) 2026 lazypod contributors
// SPDX-License-Identifier: AGPL-3.0-only

// This file is part of Lazypod.
//
// Lazypod is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Lazypod is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Lazypod. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use lazypod::app::App;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    // Detect container engines before entering the alternate screen: these
    // probes block on subprocesses, and doing so after switching screens
    // leaves a blank terminal window that garbles the first rendered frame.
    let mut app = App::new();

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let res = app.run(&mut terminal).await;

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;

    if let Err(ref err) = res {
        tracing::error!("Application error: {err:?}");
        eprintln!("Application error: {err:?}");
    }

    res
}

/// Initialize tracing without corrupting the TUI.
///
/// While the alternate screen is active, any write to stdout or a TTY-bound
/// stderr lands mid-frame and ratatui's diff renderer never repaints those
/// cells, leaving garbage across the UI. Logs therefore go to
/// `LAZYPOD_LOG_FILE` when set; otherwise to stderr only if it has been
/// redirected away from the terminal; otherwise to a temp file.
fn init_tracing() {
    use crossterm::tty::IsTty;

    let filter = |level: tracing::Level| {
        tracing_subscriber::EnvFilter::from_default_env().add_directive(level.into())
    };

    if let Ok(log_path) = std::env::var("LAZYPOD_LOG_FILE") {
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter(tracing::Level::DEBUG))
                .with_writer(file)
                .with_ansi(false)
                .try_init();
        }
        return;
    }

    if !std::io::stderr().is_tty() {
        // stderr was redirected (pipe/file/journal): safe to stream there.
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter(tracing::Level::INFO))
            .with_writer(std::io::stderr)
            .try_init();
        return;
    }

    let fallback = std::env::temp_dir().join("lazypod.log");
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&fallback)
    {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter(tracing::Level::DEBUG))
            .with_writer(file)
            .with_ansi(false)
            .try_init();
    }
}
