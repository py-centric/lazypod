use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, Tab};

pub fn draw_panels(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(17),
            Constraint::Percentage(17),
            Constraint::Percentage(17),
            Constraint::Percentage(17),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ])
        .split(area);

    let (r_items, r_state) = prepare_running_list(app);
    f.render_stateful_widget(r_items, chunks[0], &mut r_state.clone());

    let (s_items, s_state) = prepare_stopped_list(app);
    f.render_stateful_widget(s_items, chunks[1], &mut s_state.clone());

    let (i_items, i_state) = prepare_image_list(app);
    f.render_stateful_widget(i_items, chunks[2], &mut i_state.clone());

    let (v_items, v_state) = prepare_volume_list(app);
    f.render_stateful_widget(v_items, chunks[3], &mut v_state.clone());

    let (n_items, n_state) = prepare_network_list(app);
    f.render_stateful_widget(n_items, chunks[4], &mut n_state.clone());

    let (p_items, p_state) = prepare_pods_list(app);
    f.render_stateful_widget(p_items, chunks[5], &mut p_state.clone());
}

fn prepare_running_list(app: &App) -> (List<'static>, ListState) {
    let items: Vec<ListItem> = app
        .running
        .iter()
        .map(|c| {
            let names = c.get_names();
            let name = names.first().map(|s| s.as_str()).unwrap_or("Unknown");
            let prefix = if c.engine == "docker" { "[D] " } else { "[P] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Blue)),
                Span::raw(format!("{} ({})", name, &c.id[..std::cmp::min(12, c.id.len())])),
            ]))
        })
        .collect();

    let title = format!(" Running ({}) ", app.running.len());
    let border_style = if matches!(app.active_tab, Tab::Running) && !app.logs_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if matches!(app.active_tab, Tab::Running) && !app.running.is_empty() {
        state.select(Some(app.selected_index));
    }
    (list, state)
}

fn prepare_stopped_list(app: &App) -> (List<'static>, ListState) {
    let items: Vec<ListItem> = app
        .stopped
        .iter()
        .map(|c| {
            let names = c.get_names();
            let name = names.first().map(|s| s.as_str()).unwrap_or("Unknown");
            let prefix = if c.engine == "docker" { "[D] " } else { "[P] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Blue)),
                Span::raw(format!("{} ({})", name, &c.id[..std::cmp::min(12, c.id.len())])),
            ]))
        })
        .collect();

    let title = format!(" Stopped ({}) ", app.stopped.len());
    let border_style = if matches!(app.active_tab, Tab::Stopped) && !app.logs_focused {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if matches!(app.active_tab, Tab::Stopped) && !app.stopped.is_empty() {
        state.select(Some(app.selected_index));
    }
    (list, state)
}

fn prepare_image_list(app: &App) -> (List<'static>, ListState) {
    let items: Vec<ListItem> = app
        .images
        .iter()
        .map(|i| {
            let names = i.get_names();
            let name = names.first().map(|s| s.as_str()).unwrap_or("<none>");
            let prefix = if i.engine == "docker" { "[D] " } else { "[P] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Blue)),
                Span::raw(format!("{} ({})", name, i.get_size_str())),
            ]))
        })
        .collect();

    let title = format!(" Images ({}) ", app.images.len());
    let border_style = if matches!(app.active_tab, Tab::Images) && !app.logs_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if matches!(app.active_tab, Tab::Images) && !app.images.is_empty() {
        state.select(Some(app.selected_index));
    }
    (list, state)
}

fn prepare_volume_list(app: &App) -> (List<'static>, ListState) {
    let items: Vec<ListItem> = app
        .volumes
        .iter()
        .map(|v| {
            let prefix = if v.engine == "docker" { "[D] " } else { "[P] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Blue)),
                Span::raw(v.name.clone()),
            ]))
        })
        .collect();

    let title = format!(" Volumes ({}) ", app.volumes.len());
    let border_style = if matches!(app.active_tab, Tab::Volumes) && !app.logs_focused {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if matches!(app.active_tab, Tab::Volumes) && !app.volumes.is_empty() {
        state.select(Some(app.selected_index));
    }
    (list, state)
}

fn prepare_network_list(app: &App) -> (List<'static>, ListState) {
    let items: Vec<ListItem> = app
        .networks
        .iter()
        .map(|n| {
            let prefix = if n.engine == "docker" { "[D] " } else { "[P] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Blue)),
                Span::raw(n.name.clone()),
            ]))
        })
        .collect();

    let title = format!(" Networks ({}) ", app.networks.len());
    let border_style = if matches!(app.active_tab, Tab::Networks) && !app.logs_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if matches!(app.active_tab, Tab::Networks) && !app.networks.is_empty() {
        state.select(Some(app.selected_index));
    }
    (list, state)
}

fn prepare_pods_list(app: &App) -> (List<'static>, ListState) {
    let items: Vec<ListItem> = app
        .pods
        .iter()
        .map(|p| {
            let prefix = if p.engine == "docker" { "[D] " } else { "[P] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Blue)),
                Span::raw(format!("{} ({})", p.name, &p.id[..std::cmp::min(12, p.id.len())])),
            ]))
        })
        .collect();

    let title = format!(" Pods ({}) ", app.pods.len());
    let border_style = if matches!(app.active_tab, Tab::Pods) && !app.logs_focused {
        Style::default().fg(Color::LightGreen)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if matches!(app.active_tab, Tab::Pods) && !app.pods.is_empty() {
        state.select(Some(app.selected_index));
    }
    (list, state)
}
