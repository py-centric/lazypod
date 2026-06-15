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
            format!(
                "ID: {}\nImage: {}\nCommand: {}\nStatus: {}\nEngine: {}",
                c.id,
                c.image,
                c.get_command(),
                c.get_status_str(),
                c.engine
            )
        }),
        Tab::Stopped => app.stopped.get(app.selected_index).map(|c| {
            format!(
                "ID: {}\nImage: {}\nCommand: {}\nStatus: {}\nEngine: {}",
                c.id,
                c.image,
                c.get_command(),
                c.get_status_str(),
                c.engine
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
    } else {
        " Logs "
    };

    let logs_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(logs_border_style);

    if matches!(app.active_tab, Tab::Running | Tab::Stopped) {
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
        let p = Paragraph::new("Logs only available for containers.")
            .block(logs_block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[1]);
    }
}
