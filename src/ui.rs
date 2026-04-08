use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let app_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(f.size());

    let view_str = match app.engine_view {
        crate::app::EngineView::Both => "Both (Docker & Podman)",
        crate::app::EngineView::Docker => "Docker Only",
        crate::app::EngineView::Podman => "Podman Only",
    };
    let top_bar_text = Paragraph::new(format!(
        " Lazypod Filters ❯ Engine: [{}]  (Press 'E' to toggle)",
        view_str
    ))
    .style(
        Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(top_bar_text, app_layout[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(app_layout[1]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    // Running Containers Block
    let r_style = if matches!(app.active_tab, Tab::Running) && !app.logs_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let running_items: Vec<ListItem> = app
        .running
        .iter()
        .map(|c| {
            let name = c
                .get_names()
                .into_iter()
                .next()
                .unwrap_or_else(|| c.id.clone());
            let engine_icon = if c.engine == "docker" { "[D]" } else { "[P]" };
            ListItem::new(format!("▶ {} {} ({})", engine_icon, name, c.image))
        })
        .collect();

    let mut r_state = ListState::default();
    r_state.select(Some(if matches!(app.active_tab, Tab::Running) {
        app.selected_index
    } else {
        app.running_index
    }));

    let r_highlight_style = if matches!(app.active_tab, Tab::Running) && !app.logs_focused {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let r_list = List::new(running_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Running")
                .border_style(r_style),
        )
        .highlight_style(r_highlight_style);

    f.render_stateful_widget(r_list, left_chunks[0], &mut r_state);

    // Stopped Containers Block
    let s_style = if matches!(app.active_tab, Tab::Stopped) && !app.logs_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let stopped_items: Vec<ListItem> = app
        .stopped
        .iter()
        .map(|c| {
            let name = c
                .get_names()
                .into_iter()
                .next()
                .unwrap_or_else(|| c.id.clone());
            let engine_icon = if c.engine == "docker" { "[D]" } else { "[P]" };
            ListItem::new(format!("■ {} {} ({})", engine_icon, name, c.image))
        })
        .collect();

    let mut s_state = ListState::default();
    s_state.select(Some(if matches!(app.active_tab, Tab::Stopped) {
        app.selected_index
    } else {
        app.stopped_index
    }));

    let s_highlight_style = if matches!(app.active_tab, Tab::Stopped) && !app.logs_focused {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let s_list = List::new(stopped_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Stopped")
                .border_style(s_style),
        )
        .highlight_style(s_highlight_style);

    f.render_stateful_widget(s_list, left_chunks[1], &mut s_state);

    // Images Block
    let i_style = if matches!(app.active_tab, Tab::Images) && !app.logs_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let images: Vec<ListItem> = app
        .images
        .iter()
        .map(|i| {
            let tag = i
                .get_names()
                .into_iter()
                .next()
                .unwrap_or_else(|| "<none>".into());
            let engine_icon = if i.engine == "docker" { "[D]" } else { "[P]" };
            ListItem::new(format!("{} {}", engine_icon, tag))
        })
        .collect();

    let mut i_state = ListState::default();
    i_state.select(Some(if matches!(app.active_tab, Tab::Images) {
        app.selected_index
    } else {
        app.images_index
    }));

    let i_highlight_style = if matches!(app.active_tab, Tab::Images) && !app.logs_focused {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let i_list = List::new(images)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Images")
                .border_style(i_style),
        )
        .highlight_style(i_highlight_style);

    f.render_stateful_widget(i_list, left_chunks[2], &mut i_state);

    // Volumes Block
    let v_style = if matches!(app.active_tab, Tab::Volumes) && !app.logs_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let volumes: Vec<ListItem> = app
        .volumes
        .iter()
        .map(|v| {
            let engine_icon = if v.engine == "docker" { "[D]" } else { "[P]" };
            ListItem::new(format!("{} {}", engine_icon, v.name.clone()))
        })
        .collect();

    let mut v_state = ListState::default();
    v_state.select(Some(if matches!(app.active_tab, Tab::Volumes) {
        app.selected_index
    } else {
        app.volumes_index
    }));

    let v_highlight_style = if matches!(app.active_tab, Tab::Volumes) && !app.logs_focused {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let v_list = List::new(volumes)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Volumes")
                .border_style(v_style),
        )
        .highlight_style(v_highlight_style);

    f.render_stateful_widget(v_list, left_chunks[3], &mut v_state);

    // Networks Block
    let n_style = if matches!(app.active_tab, Tab::Networks) && !app.logs_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let networks: Vec<ListItem> = app
        .networks
        .iter()
        .map(|n| {
            let engine_icon = if n.engine == "docker" { "[D]" } else { "[P]" };
            ListItem::new(format!("{} {}", engine_icon, n.name.clone()))
        })
        .collect();

    let mut n_state = ListState::default();
    n_state.select(Some(if matches!(app.active_tab, Tab::Networks) {
        app.selected_index
    } else {
        app.networks_index
    }));

    let n_highlight_style = if matches!(app.active_tab, Tab::Networks) && !app.logs_focused {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let n_list = List::new(networks)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Networks")
                .border_style(n_style),
        )
        .highlight_style(n_highlight_style);

    f.render_stateful_widget(n_list, left_chunks[4], &mut n_state);

    // Details Block (Right panel)
    let details_text = match app.active_tab {
        Tab::Running => {
            if let Some(c) = app.running.get(app.selected_index) {
                format!(
                    "ID: {}\nImage: {}\nState: {}\nStatus: {}\nCommand: {}\n\nLogs (last 50 lines):\n{}",
                    c.id, c.image, c.get_state_str(), c.get_status_str(),
                    c.get_command(),
                    app.container_logs
                )
            } else {
                "No running container selected".to_string()
            }
        }
        Tab::Stopped => {
            if let Some(c) = app.stopped.get(app.selected_index) {
                format!(
                    "ID: {}\nImage: {}\nState: {}\nStatus: {}\nCommand: {}\n\nLogs (last 50 lines):\n{}",
                    c.id, c.image, c.get_state_str(), c.get_status_str(),
                    c.get_command(),
                    app.container_logs
                )
            } else {
                "No stopped container selected".to_string()
            }
        }
        Tab::Images => {
            if let Some(i) = app.images.get(app.selected_index) {
                format!("ID: {}\nSize: {} bytes", i.id, i.size.unwrap_or(0))
            } else {
                "No image selected".to_string()
            }
        }
        Tab::Volumes => {
            if let Some(v) = app.volumes.get(app.selected_index) {
                format!(
                    "Name: {}\nDriver: {}\nMountpoint: {}",
                    v.name, v.driver, v.mountpoint
                )
            } else {
                "No volume selected".to_string()
            }
        }
        Tab::Networks => {
            if let Some(n) = app.networks.get(app.selected_index) {
                format!("ID: {}\nName: {}\nDriver: {}", n.id, n.name, n.driver)
            } else {
                "No network selected".to_string()
            }
        }
    };

    let mut details_block = Block::default().borders(Borders::ALL).title("Details");
    if app.logs_focused {
        details_block = details_block.style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    }

    let details = Paragraph::new(details_text)
        .block(details_block)
        .scroll((app.logs_scroll, 0));
    f.render_widget(details, chunks[1]);

    // Help Block
    let tab_specific_help = match app.active_tab {
        Tab::Running => " | s: Stop | i/e: Shell | x: Exec",
        Tab::Stopped => " | u: Start | d: rm",
        Tab::Images => " | /: Search | p: Pull | c: Regs | Enter: Run",
        Tab::Volumes | Tab::Networks => " | d: rm",
    };

    let help_text = Paragraph::new(format!(
        " ?: Help Tooltips | q: Quit | Tab: Cycle Panes | ↓↑: Nav (boundary switch){} | E: Toggle Engine",
        tab_specific_help
    ))
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help_text, main_chunks[1]);

    // Confirmation Popup
    if app.show_confirmation {
        let (title, action_text) = if let Some((resource_type, _, id, action)) = &app.pending_action {
            let t = format!(" Confirm {} ", action);
            let related = app.get_related_resources(resource_type, id);
            let a = if !related.is_empty() {
                format!(
                    "Are you sure you want to {} this resource?\nFound {} related resources (containers).\n\nPress 'y' or 'Enter' for this only\nPress 'a' to delete ALL (resource + related)",
                    action,
                    related.len()
                )
            } else {
                format!("Are you sure you want to {} this resource?", action)
            };
            (t, a)
        } else {
            (" Confirm Stop ".to_string(), "Are you sure you want to stop this container?".to_string())
        };

        let footer = if app.pending_action.is_some() && !app.get_related_resources(&app.pending_action.as_ref().unwrap().0, &app.pending_action.as_ref().unwrap().2).is_empty() {
            "\n\nPress 'y'/'Enter': this only | 'a': ALL | 'n'/'Esc': cancel"
        } else {
            "\n\nPress 'y' or 'Enter' to confirm, 'n' or 'Esc' to cancel."
        };

        let text = Paragraph::new(format!("{}{}", action_text, footer))
            .block(Block::default().title(title).borders(Borders::ALL).style(Style::default().fg(Color::Red)))
            .wrap(Wrap { trim: true });

        // Centered popup area
        let area = centered_rect(50, 30, f.size());
        f.render_widget(Clear, area);
        f.render_widget(text, area);
    }

    // Create Container Popup
    if let Some(form) = &app.create_container_form {
        let area = centered_rect(60, 50, f.size());
        let block = Block::default()
            .title(" Run Container (Enter: submit | Esc: cancel | Tab: next) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(Clear, area);

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(inner_area);

        let mut name_style = Style::default();
        let mut cmd_style = Style::default();
        let mut ports_style = Style::default();
        let mut env_style = Style::default();

        match form.active_field {
            0 => name_style = name_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            1 => cmd_style = cmd_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            2 => ports_style = ports_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            3 => env_style = env_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            _ => {}
        }

        let name_p = Paragraph::new(form.name.as_str())
            .block(Block::default().title("Name").borders(Borders::ALL))
            .style(name_style);
        let cmd_p = Paragraph::new(form.command.as_str())
            .block(Block::default().title("Command").borders(Borders::ALL))
            .style(cmd_style);
        let ports_p = Paragraph::new(form.ports.as_str())
            .block(
                Block::default()
                    .title("Ports (e.g. 8080:80)")
                    .borders(Borders::ALL),
            )
            .style(ports_style);
        let env_p = Paragraph::new(form.env.as_str())
            .block(
                Block::default()
                    .title("Env (e.g. VAR=val foo=bar)")
                    .borders(Borders::ALL),
            )
            .style(env_style);

        f.render_widget(name_p, layout[0]);
        f.render_widget(cmd_p, layout[1]);
        f.render_widget(ports_p, layout[2]);
        f.render_widget(env_p, layout[3]);
    }

    // Search Image Popup
    if let Some(form) = &app.search_image_form {
        let area = centered_rect(80, 80, f.size());
        let title = if app.is_pulling {
            " Pulling Image... Please wait "
        } else if form.is_searching {
            " Searching... "
        } else {
            " Search Images (Enter: search/pull | Esc: cancel | ↑↓: select) "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(Clear, area);
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(inner_area);

        let query_p = Paragraph::new(format!("> {}_", form.query))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(query_p, layout[0]);

        if !form.results.is_empty() {
            let items: Vec<ListItem> = form
                .results
                .iter()
                .map(|r| {
                    let desc = match r.description.chars().count() {
                        n if n > 60 => {
                            format!("{}...", r.description.chars().take(57).collect::<String>())
                        }
                        _ => r.description.clone(),
                    };
                    let official_tag = if !r.official.is_empty() { " [OK]" } else { "" };
                    ListItem::new(format!(
                        "{}{} (★ {})\n  {}",
                        r.name, official_tag, r.stars, desc
                    ))
                })
                .collect();

            let mut state = ListState::default();
            state.select(Some(form.selected));

            let list = List::new(items).highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

            f.render_stateful_widget(list, layout[1], &mut state);
        } else if !form.query.is_empty() && !form.is_searching {
            let empty_p =
                Paragraph::new("Press Enter to search").style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_p, layout[1]);
        }
    }

    if let Some(form) = &app.direct_pull_form {
        let area = centered_rect(50, 20, f.size());
        let text = if app.is_pulling { " Pulling... " } else { "> " };
        let p = Paragraph::new(format!("{}{}_", text, form.image)).block(
            Block::default()
                .title(" Direct Pull Image (Enter: pull | Esc: cancel) ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(Clear, area);
        f.render_widget(p, area);
    }

    if let Some(form) = &app.configure_registries_form {
        let area = centered_rect(60, 20, f.size());
        let p = Paragraph::new(format!("> {}_", form.registries)).block(
            Block::default()
                .title(" Configure Search Registries (comma separated) ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        );
        f.render_widget(Clear, area);
        f.render_widget(p, area);
    }

    if let Some(form) = &app.exec_form {
        // Only show if the user is expected to type (i.e. command is empty or they initiated 'x')
        let area = centered_rect(60, 20, f.size());
        let p = Paragraph::new(format!("> {}_", form.command)).block(
            Block::default()
                .title(" Exec Command (e.g. /bin/bash, ls -l) | Enter: run | Esc: cancel ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(Clear, area);
        f.render_widget(p, area);
    }

    // Help Tooltip Popup
    if app.show_help_tooltip {
        let area = centered_rect(50, 60, f.size());
        let help_text = "
LAZYPOD KEYBINDINGS:

Global:
  q, Ctrl+c     : Quit
  Tab           : Cycle through ALL Panes (Left Tabs -> Logs)
  ?, Esc        : Toggle this Help popup
  E             : Toggle Engine Filter (Docker/Podman/Both)

Navigation:
  Up/Down, k/j  : Move selection (Switches panes at boundaries)
  Enter         : Primary Action (Logs, Run Image)

Actions:
  s             : Toggle Stop (Running tab) / Start (Stopped tab)
  d, Delete     : Remove resource (container, image, volume)
  i, e          : Interactive Shell (defaults to /bin/sh)
  x             : Exec Custom Command (opens prompt)
  /             : Search Images online (Images tab)
  p             : Pull Image directly by name (Images tab)
  c             : Configure Search Registries (Images tab)
";
        let text = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help Tooltips ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(Clear, area);
        f.render_widget(text, area);
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_centered_rect() {
        let root = Rect::new(0, 0, 100, 100);
        let rect = centered_rect(50, 50, root);
        
        // 50% of 100 is 50. 
        // (100 - 50) / 2 = 25 start.
        assert_eq!(rect.width, 50);
        assert_eq!(rect.height, 50);
        assert_eq!(rect.x, 25);
        assert_eq!(rect.y, 25);
    }
}
