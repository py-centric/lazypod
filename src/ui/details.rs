use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Tab};

#[allow(clippy::too_many_lines)]
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
            let cmd = c.get_command();
            let status = c.get_status_str();
            let engine = &c.engine;
            let id = &c.id;
            let image = &c.image;
            format!("ID: {id}\nImage: {image}\nCommand: {cmd}\nStatus: {status}\nEngine: {engine}{ports_str}")
        }),
        Tab::Stopped => app.stopped.get(app.selected_index).map(|c| {
            let ports = c.get_port_strings();
            let ports_str = if ports.is_empty() {
                String::new()
            } else {
                format!("\nPorts: {}", ports.join(", "))
            };
            let cmd = c.get_command();
            let status = c.get_status_str();
            let engine = &c.engine;
            let id = &c.id;
            let image = &c.image;
            format!("ID: {id}\nImage: {image}\nCommand: {cmd}\nStatus: {status}\nEngine: {engine}{ports_str}")
        }),
        Tab::Images => app.get_filtered_images().get(app.selected_index).map(|i| {
            let tags = i.get_names().join(", ");
            let size = i.get_size_str();
            let created = i.get_created_str();
            let engine = &i.engine;
            let id = &i.id;
            let dangling_str = if i.is_dangling() { "\nDangling: Yes" } else { "" };
            format!("ID: {id}\nTags: {tags}\nSize: {size}\nCreated: {created}\nEngine: {engine}{dangling_str}")
        }),
        Tab::Volumes => app.volumes.get(app.selected_index).map(|v| {
            let name = &v.name;
            let driver = &v.driver;
            let mountpoint = &v.mountpoint;
            let engine = &v.engine;
            format!("Name: {name}\nDriver: {driver}\nMountpoint: {mountpoint}\nEngine: {engine}")
        }),
        Tab::Networks => app.networks.get(app.selected_index).map(|n| {
            let name = &n.name;
            let id = &n.id;
            let driver = &n.driver;
            let engine = &n.engine;
            format!("Name: {name}\nID: {id}\nDriver: {driver}\nEngine: {engine}")
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
                    lines.push(format!(
                        "  {} [{}] {}",
                        name,
                        &c.id[..std::cmp::min(12, c.id.len())],
                        status
                    ));
                }
                lines.join("\n")
            };
            // Aggregate ports from containers in this pod (matched by pod_id in container list)
            let all_ports: Vec<String> = app
                .running
                .iter()
                .chain(app.stopped.iter())
                .filter(|c| c.pod_id.as_deref() == Some(&p.id))
                .flat_map(super::super::podman::models::Container::get_port_strings)
                .collect();
            let ports_str = if all_ports.is_empty() {
                String::new()
            } else {
                format!("\nPorts: {}", all_ports.join(", "))
            };
            let name = &p.name;
            let id = &p.id;
            let status = &p.status;
            let created = p.get_created_str();
            let engine = &p.engine;
            format!(
                "Name: {name}\nID: {id}\nStatus: {status}\nCreated: {created}\nEngine: {engine}\nContainers:\n{containers_str}{ports_str}"
            )
        }),
    }
    .unwrap_or_else(|| "Nothing selected.".to_string());

    let p = Paragraph::new(info_text)
        .block(info_block)
        .wrap(Wrap { trim: true });
    f.render_widget(p, chunks[0]);

    let logs_border_style = if app.logs_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let title = if app.logs_focused {
        " Logs (Press 'y' to copy line, 'Esc' to exit) "
    } else if matches!(app.active_tab, Tab::Pods) {
        " Pod Logs "
    } else if matches!(app.active_tab, Tab::Images) {
        " Image History / Layers "
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

        let list = List::new(items).block(logs_block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        f.render_stateful_widget(list, chunks[1], &mut app.logs_state);
    } else if matches!(app.active_tab, Tab::Images) {
        if app.image_history.is_empty() {
            let p = Paragraph::new("No image history available. Select an image.")
                .block(logs_block)
                .wrap(Wrap { trim: true });
            f.render_widget(p, chunks[1]);
        } else {
            let items: Vec<ListItem> = app
                .image_history
                .iter()
                .map(|l| ListItem::new(Line::from(l.clone())))
                .collect();
            let list = List::new(items).block(logs_block);
            f.render_widget(list, chunks[1]);
        }
    } else {
        let p = Paragraph::new("Logs only available for containers.")
            .block(logs_block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[1]);
    }
}
