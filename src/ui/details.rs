use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Tab};

pub fn draw_details(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let info_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let info_text = match app.active_tab {
        Tab::Running => app.running.get(app.selected_index).map(|c| {
            let ports = c.get_port_strings();
            let ports_str = if ports.is_empty() {
                String::new()
            } else {
                format!("\nPorts: {}", ports.join(", "))
            };
            format!(
                "ID: {}\nImage: {}\nCommand: {}\nStatus: {}\nEngine:{}{}",
                c.id,
                c.image,
                c.get_command(),
                c.get_status_str(),
                c.engine,
                ports_str
            )
        }),
        Tab::Stopped => app.stopped.get(app.selected_index).map(|c| {
            let ports = c.get_port_strings();
            let ports_str = if ports.is_empty() {
                String::new()
            } else {
                format!("\nPorts: {}", ports.join(", "))
            };
            format!(
                "ID: {}\nImage: {}\nCommand: {}\nStatus: {}\nEngine:{}{}",
                c.id,
                c.image,
                c.get_command(),
                c.get_status_str(),
                c.engine,
                ports_str
            )
        }),
        Tab::Images => app.images.get(app.selected_index).map(|i| {
            format!(
                "ID: {}\nTags: {}\nSize: {}\nCreated: {}\nEngine: {}",
                i.id,
                i.get_names().join(", "),
                i.get_size_str(),
                i.get_created_str(),
                i.engine
            )
        }),
        Tab::Volumes => app.volumes.get(app.selected_index).map(|v| {
            format!(
                "Name: {}\nDriver: {}\nMountpoint: {}\nEngine: {}",
                v.name, v.driver, v.mountpoint, v.engine
            )
        }),
        Tab::Networks => app.networks.get(app.selected_index).map(|n| {
            format!(
                "Name: {}\nID: {}\nDriver: {}\nEngine: {}",
                n.name, n.id, n.driver, n.engine
            )
        }),
        Tab::Pods => app.pods.get(app.selected_index).map(|p| {
            // Use containers directly from pod ps output
            let containers_str = if p.containers.is_empty() {
                "None".to_string()
            } else {
                let mut lines = Vec::new();
                for c in &p.containers {
                    let name = c.get_name();
                    let status = c.get_status_str();
                    lines.push(format!("  {} [{}] {}", name, &c.id[..std::cmp::min(12, c.id.len())], status));
                }
                lines.join("\n")
            };
            // Aggregate ports from containers in this pod (matched by pod_id in container list)
            let all_ports: Vec<String> = app.running.iter()
                .chain(app.stopped.iter())
                .filter(|c| c.pod_id.as_deref() == Some(&p.id))
                .flat_map(|c| c.get_port_strings())
                .collect();
            let ports_str = if all_ports.is_empty() {
                String::new()
            } else {
                format!("\nPorts: {}", all_ports.join(", "))
            };
            format!(
                "Name: {}\nID: {}\nStatus: {}\nCreated: {}\nEngine: {}\nContainers:\n{}{}",
                p.name, p.id, p.status, p.get_created_str(), p.engine, containers_str, ports_str
            )
        }),
    }
    .unwrap_or_else(|| "Nothing selected.".to_string());

    let p = Paragraph::new(info_text).block(info_block).wrap(Wrap { trim: true });
    f.render_widget(p, chunks[0]);

    let logs_border_style = if app.logs_focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let title = if app.logs_focused {
        " Logs (Press 'y' to copy line, 'Esc' to exit) "
    } else if matches!(app.active_tab, Tab::Pods) {
        " Pod Logs "
    } else {
        " Logs "
    };

    let logs_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(logs_border_style);

    if matches!(app.active_tab, Tab::Running | Tab::Stopped | Tab::Pods) {
        let items: Vec<ListItem> = app
            .container_logs
            .iter()
            .map(|l| ListItem::new(Line::from(l.clone())))
            .collect();
            
        let list = List::new(items)
            .block(logs_block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
            
        f.render_stateful_widget(list, chunks[1], &mut app.logs_state);
    } else {
        let hint = match app.active_tab {
            Tab::Pods => "Logs not available for pods. Select a container.",
            _ => "Logs only available for containers.",
        };
        let p = Paragraph::new(hint)
            .block(logs_block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[1]);
    }
}
