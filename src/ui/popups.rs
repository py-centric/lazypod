use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use std::fmt::Write;

#[allow(clippy::too_many_lines)]
pub fn draw_popups(f: &mut Frame, app: &mut App, area: Rect) {
    if let Some(inspect_output) = &app.inspect_popup {
        let block = Block::default()
            .title(" Inspect (Esc/g to close, j/k to scroll) ")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );

        let lines: Vec<Line> = inspect_output
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();

        let p = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.inspect_scroll, 0));
        let popup_area = centered_rect(80, 80, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if app.show_help_tooltip {
        let block = Block::default()
            .title(" Help | Lazypod (Built by PyCentric) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let help_text = vec![
            Line::from("Global Bindings:"),
            Line::from("  q          Quit"),
            Line::from("  ?          Toggle this help"),
            Line::from("  Tab        Next panel"),
            Line::from("  BackTab    Previous panel"),
            Line::from("  E          Toggle engine (Both/Docker/Podman)"),
            Line::from("  r          Refresh data"),
            Line::from("  g          Inspect selected resource"),
            Line::from(""),
            Line::from("List Bindings:"),
            Line::from("  Up/Down    Navigate list"),
            Line::from("  d/Del      Remove selected item"),
            Line::from("  s          Stop running container / Start stopped container"),
            Line::from("  S/u        Start selected item"),
            Line::from("  x          Exec into container (custom command)"),
            Line::from("  i/e        Exec into container (/bin/sh)"),
            Line::from("  Enter      View logs (containers) / Create container (images)"),
            Line::from(""),
            Line::from("Images Tab:"),
            Line::from("  /          Search registry for images"),
            Line::from("  p          Pull an image directly"),
            Line::from("  t          Tag selected image"),
            Line::from("  f          Toggle dangling images filter"),
            Line::from("  P          Prune dangling images"),
            Line::from("  X          Prune all unused images"),
            Line::from("  c          Configure unqualified registries"),
            Line::from(""),
            Line::from("Pods Tab:"),
            Line::from("  P          Create a new pod"),
            Line::from("  s          Stop/Start pod"),
            Line::from("  d/Del      Remove pod"),
            Line::from(""),
            Line::from("Logs Panel:"),
            Line::from("  Up/Down    Scroll logs"),
            Line::from("  y/c        Copy selected log line to clipboard"),
            Line::from("  Esc        Exit logs focus"),
        ];
        let p = Paragraph::new(help_text)
            .block(block)
            .alignment(Alignment::Left);
        let popup_area = centered_rect(55, 70, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if app.is_pulling {
        let block = Block::default()
            .title(" Pulling Image ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let p = Paragraph::new("Please wait...")
            .block(block)
            .alignment(Alignment::Center);
        let popup_area = centered_rect(30, 20, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if let Some(form) = &app.exec_form {
        let block = Block::default()
            .title(" Exec Command ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let text = format!("{}\n\n(Enter to execute, Esc to cancel)", form.command);
        let p = Paragraph::new(text).block(block);
        let popup_area = centered_rect(50, 30, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if let Some(form) = &app.configure_registries_form {
        let block = Block::default()
            .title(" Configure Unqualified Registries ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let text = format!(
            "Enter registries separated by commas:\n{}\n\n(Enter to save, Esc to cancel)",
            form.registries
        );
        let p = Paragraph::new(text).block(block);
        let popup_area = centered_rect(50, 30, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if let Some(form) = &app.direct_pull_form {
        let block = Block::default()
            .title(" Direct Pull ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let text = format!(
            "Image name:\n{}\n\n(Enter to pull, Esc to cancel)",
            form.image
        );
        let p = Paragraph::new(text).block(block);
        let popup_area = centered_rect(50, 30, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if let Some(form) = &app.search_image_form {
        let block = Block::default()
            .title(" Search Images ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(centered_rect(60, 60, area));

        f.render_widget(Clear, centered_rect(60, 60, area));
        f.render_widget(block, centered_rect(60, 60, area));

        let search_text = format!("Query: {}", form.query);
        f.render_widget(Paragraph::new(search_text), chunks[0]);

        if form.is_searching {
            f.render_widget(Paragraph::new("Searching..."), chunks[1]);
        } else {
            let items: Vec<ListItem> = form
                .results
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let style = if i == form.selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(
                        format!(
                            "{:<30} | Stars: {:<4} | Official: {}",
                            &r.name[..std::cmp::min(30, r.name.len())],
                            r.stars,
                            r.official
                        ),
                        style,
                    )))
                })
                .collect();
            let list = List::new(items).block(Block::default().borders(Borders::TOP));
            f.render_widget(list, chunks[1]);
        }
        return;
    }

    if let Some(form) = &app.create_pod_form {
        let block = Block::default()
            .title(" Create Pod ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let pid_marker = if form.share_pid { "[x]" } else { "[ ]" };
        let net_marker = if form.share_net { "[x]" } else { "[ ]" };

        let text = vec![
            Line::from(Span::styled(
                format!("Name: {}", form.name),
                if form.active_field == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                format!("Network: {}", form.network),
                if form.active_field == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                format!("Share PID: {pid_marker} (Space to toggle)"),
                if form.active_field == 2 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                format!("Share Net: {net_marker} (Space to toggle)"),
                if form.active_field == 3 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(""),
            Line::from("(Tab to switch fields, Enter to create, Esc to cancel)"),
        ];

        let p = Paragraph::new(text).block(block);
        let popup_area = centered_rect(50, 40, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if let Some(form) = &app.create_container_form {
        let block = Block::default()
            .title(" Create Container ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let text = vec![
            Line::from(Span::styled(
                format!("Name: {}", form.name),
                if form.active_field == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                format!("Command: {}", form.command),
                if form.active_field == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                format!("Ports (e.g. 8080:80): {}", form.ports),
                if form.active_field == 2 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(Span::styled(
                format!("Env (e.g. FOO=BAR BAZ=1): {}", form.env),
                if form.active_field == 3 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            )),
            Line::from(""),
            Line::from("(Tab to switch fields, Enter to create, Esc to cancel)"),
        ];

        let p = Paragraph::new(text).block(block);
        let popup_area = centered_rect(50, 40, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if let Some(form) = &app.tag_image_form {
        let block = Block::default()
            .title(" Tag Image ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let selected_img = app
            .get_filtered_images()
            .get(app.selected_index)
            .map_or_else(
                || "Unknown".to_string(),
                |i| {
                    i.get_names()
                        .first()
                        .map_or_else(|| i.id.clone(), Clone::clone)
                },
            );
        let text = vec![
            Line::from(format!("Image: {selected_img}")),
            Line::from(""),
            Line::from(vec![
                Span::raw("Target Tag: "),
                Span::styled(&form.target_tag, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from("(Enter to tag, Esc to cancel)"),
        ];
        let p = Paragraph::new(text).block(block);
        let popup_area = centered_rect(55, 25, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
        return;
    }

    if app.show_confirmation {
        let block = Block::default()
            .title(" Confirmation ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let mut msg = "Are you sure you want to perform this action?".to_string();
        if let Some((tab, _engine, id, action)) = &app.pending_action {
            if action == "prune_dangling" {
                msg = "Are you sure you want to prune dangling images from all active engines?\n\nPress 'y' to continue, 'n' to cancel.".to_string();
            } else if action == "prune_all" {
                msg = "Are you sure you want to prune ALL unused images from all active engines?\n\nPress 'y' to continue, 'n' to cancel.".to_string();
            } else {
                let related = app.get_related_resources(tab, id);
                if related.is_empty() {
                    msg.push_str("\n\nPress 'y' to continue, 'n' to cancel.");
                } else {
                    msg.push_str("\n\nThis will also affect:");
                    for (rt, _, rid) in related {
                        let _ = write!(msg, "\n - {rt:?} {rid}");
                    }
                    msg.push_str(
                        "\n\nPress 'y' to continue, 'a' to apply to all related, 'n' to cancel.",
                    );
                }
            }
        }
        let p = Paragraph::new(msg)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        let popup_area = centered_rect(60, 30, area);
        f.render_widget(Clear, popup_area);
        f.render_widget(p, popup_area);
    }
}

#[must_use]
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
