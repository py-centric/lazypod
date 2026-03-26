use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ].as_ref())
        .split(chunks[0]);

    // Running Containers Block
    let r_style = if matches!(app.active_tab, Tab::Running) && !app.logs_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let running_items: Vec<ListItem> = app
        .running
        .iter()
        .map(|c| {
            let name = c.get_names().into_iter().next().unwrap_or_else(|| c.id.clone());
            ListItem::new(format!("▶ {} ({})", name, c.image))
        })
        .collect();

    let mut r_state = ListState::default();
    if matches!(app.active_tab, Tab::Running) {
        r_state.select(Some(app.selected_index));
    }

    let r_list = List::new(running_items)
        .block(Block::default().borders(Borders::ALL).title("Running").border_style(r_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(r_list, left_chunks[0], &mut r_state);

    // Stopped Containers Block
    let s_style = if matches!(app.active_tab, Tab::Stopped) && !app.logs_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let stopped_items: Vec<ListItem> = app
        .stopped
        .iter()
        .map(|c| {
            let name = c.get_names().into_iter().next().unwrap_or_else(|| c.id.clone());
            ListItem::new(format!("■ {} ({})", name, c.image))
        })
        .collect();

    let mut s_state = ListState::default();
    if matches!(app.active_tab, Tab::Stopped) {
        s_state.select(Some(app.selected_index));
    }

    let s_list = List::new(stopped_items)
        .block(Block::default().borders(Borders::ALL).title("Stopped").border_style(s_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(s_list, left_chunks[1], &mut s_state);

    // Images Block
    let i_style = if matches!(app.active_tab, Tab::Images) && !app.logs_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let images: Vec<ListItem> = app
        .images
        .iter()
        .map(|i| {
            let tag = i.get_names().into_iter().next().unwrap_or_else(|| "<none>".into());
            ListItem::new(tag)
        })
        .collect();

    let mut i_state = ListState::default();
    if matches!(app.active_tab, Tab::Images) {
        i_state.select(Some(app.selected_index));
    }

    let i_list = List::new(images)
        .block(Block::default().borders(Borders::ALL).title("Images").border_style(i_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(i_list, left_chunks[2], &mut i_state);

    // Volumes Block
    let v_style = if matches!(app.active_tab, Tab::Volumes) && !app.logs_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let volumes: Vec<ListItem> = app
        .volumes
        .iter()
        .map(|v| ListItem::new(v.name.clone()))
        .collect();

    let mut v_state = ListState::default();
    if matches!(app.active_tab, Tab::Volumes) {
        v_state.select(Some(app.selected_index));
    }

    let v_list = List::new(volumes)
        .block(Block::default().borders(Borders::ALL).title("Volumes").border_style(v_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(v_list, left_chunks[3], &mut v_state);

    // Networks Block
    let n_style = if matches!(app.active_tab, Tab::Networks) && !app.logs_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let networks: Vec<ListItem> = app
        .networks
        .iter()
        .map(|n| ListItem::new(n.name.clone()))
        .collect();

    let mut n_state = ListState::default();
    if matches!(app.active_tab, Tab::Networks) {
        n_state.select(Some(app.selected_index));
    }

    let n_list = List::new(networks)
        .block(Block::default().borders(Borders::ALL).title("Networks").border_style(n_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

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
                format!("Name: {}\nDriver: {}\nMountpoint: {}", v.name, v.driver, v.mountpoint)
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
        details_block = details_block.style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    }
    
    let details = Paragraph::new(details_text)
        .block(details_block)
        .scroll((app.logs_scroll, 0));
    f.render_widget(details, chunks[1]);

    // Help Block
    let help_text = Paragraph::new(" q: Quit | Tab: Switch Pane | ↓↑/jk: Navigate | Enter/Click: View Logs | Esc: Unfocus | s: Stop | u: Start | d: rm | x/e/i: Exec")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help_text, main_chunks[1]);

    // Confirmation Popup
    if app.show_confirmation {
        let text = Paragraph::new("Are you sure you want to stop this container?\n\nPress 'y' or 'Enter' to confirm, 'n' or 'Esc' to cancel.")
            .block(Block::default().title(" Confirm Stop ").borders(Borders::ALL).style(Style::default().fg(Color::Red)))
            .wrap(Wrap { trim: true });

        // Centered popup area (adjust sizing as necessary)
        let area = centered_rect(50, 20, f.size());
        f.render_widget(Clear, area); // clear background
        f.render_widget(text, area);
    }

    // Create Container Popup
    if let Some(form) = &app.create_container_form {
        let area = centered_rect(60, 40, f.size());
        let block = Block::default()
            .title(" Run Container (Enter: submit | Esc: cancel | Tab: next) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        
        f.render_widget(Clear, area);

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ].as_ref())
            .split(inner_area);

        let mut name_style = Style::default();
        let mut cmd_style = Style::default();
        let mut ports_style = Style::default();
        
        match form.active_field {
            0 => name_style = name_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            1 => cmd_style = cmd_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            2 => ports_style = ports_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            _ => {}
        }

        let name_p = Paragraph::new(form.name.as_str()).block(Block::default().title("Name").borders(Borders::ALL)).style(name_style);
        let cmd_p = Paragraph::new(form.command.as_str()).block(Block::default().title("Command").borders(Borders::ALL)).style(cmd_style);
        let ports_p = Paragraph::new(form.ports.as_str()).block(Block::default().title("Ports (e.g. 8080:80)").borders(Borders::ALL)).style(ports_style);

        f.render_widget(name_p, layout[0]);
        f.render_widget(cmd_p, layout[1]);
        f.render_widget(ports_p, layout[2]);
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
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ].as_ref())
            .split(inner_area);

        let query_p = Paragraph::new(format!("> {}_", form.query))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(query_p, layout[0]);

        if !form.results.is_empty() {
            let items: Vec<ListItem> = form.results.iter().map(|r| {
                let desc = match r.description.chars().count() {
                    n if n > 60 => format!("{}...", r.description.chars().take(57).collect::<String>()),
                    _ => r.description.clone(),
                };
                let official_tag = if !r.official.is_empty() { " [OK]" } else { "" };
                ListItem::new(format!("{}{} (★ {})\n  {}", r.name, official_tag, r.stars, desc))
            }).collect();

            let mut state = ListState::default();
            state.select(Some(form.selected));

            let list = List::new(items)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
            
            f.render_stateful_widget(list, layout[1], &mut state);
        } else if !form.query.is_empty() && !form.is_searching {
            let empty_p = Paragraph::new("Press Enter to search")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_p, layout[1]);
        }
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}
