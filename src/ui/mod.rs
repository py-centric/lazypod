pub mod details;
pub mod panels;
pub mod popups;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, EngineView};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.size());

    let active_str = match app.engine_view {
        EngineView::Both => "Both",
        EngineView::Docker => "Docker",
        EngineView::Podman => "Podman",
    };

    let missing_str = {
        let active = app.get_active_engines();
        let mut missing = Vec::new();
        match app.engine_view {
            EngineView::Both => {
                if !active.contains(&"docker".to_string()) {
                    missing.push("Docker");
                }
                if !active.contains(&"podman".to_string()) {
                    missing.push("Podman");
                }
            }
            EngineView::Docker => {
                if !active.contains(&"docker".to_string()) {
                    missing.push("Docker");
                }
            }
            EngineView::Podman => {
                if !active.contains(&"podman".to_string()) {
                    missing.push("Podman");
                }
            }
        }
        if missing.is_empty() {
            String::new()
        } else {
            format!(" (Missing: {})", missing.join(", "))
        }
    };

    let title = format!(
        " Lazypod | Active Engines: {active_str}{missing_str} | Built by PyCentric | '?' for help "
    );
    let title_block = Block::default().borders(Borders::ALL).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let title_paragraph = Paragraph::new(title).block(title_block);
    f.render_widget(title_paragraph, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    panels::draw_panels(f, app, main_chunks[0]);
    details::draw_details(f, app, main_chunks[1]);

    popups::draw_popups(f, app, f.size());

    // Status bar
    let (status, color) = if let Some(msg) = &app.status_message {
        (format!("Error: {msg}"), Color::Red)
    } else if app.is_pulling {
        (
            "Status: Pulling... | PyCentric".to_string(),
            Color::DarkGray,
        )
    } else if let Some(form) = &app.search_image_form {
        if form.is_searching {
            (
                "Status: Searching... | PyCentric".to_string(),
                Color::DarkGray,
            )
        } else {
            ("Status: OK | PyCentric".to_string(), Color::DarkGray)
        }
    } else {
        ("Status: OK | PyCentric".to_string(), Color::DarkGray)
    };

    let status_line = Paragraph::new(Line::from(vec![Span::styled(
        status,
        Style::default().fg(color),
    )]));
    f.render_widget(status_line, chunks[2]);
}
