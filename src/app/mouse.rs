use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::dispatcher;
use crate::app::{App, Tab};

/// Central mouse event handler.
pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    if app.show_confirmation || app.create_container_form.is_some() || app.create_pod_form.is_some()
    {
        return;
    }

    match mouse.kind {
        MouseEventKind::ScrollDown => handle_scroll_down(app),
        MouseEventKind::ScrollUp => handle_scroll_up(app),
        MouseEventKind::Down(MouseButton::Left) => handle_click(app, mouse.column, mouse.row),
        _ => {}
    }
}

#[allow(clippy::cast_possible_truncation)]
fn handle_scroll_down(app: &mut App) {
    if app.inspect_popup.is_some() {
        let total_lines = app
            .inspect_popup
            .as_ref()
            .map_or(0, |s| s.lines().count() as u16);
        let max_scroll = total_lines.saturating_sub(1);
        if app.inspect_scroll < max_scroll {
            app.inspect_scroll += 1;
        }
        return;
    }

    if app.logs_focused {
        if let Some(selected) = app.logs_state.selected() {
            if selected < app.container_logs.len().saturating_sub(1) {
                app.logs_state.select(Some(selected + 1));
            }
        }
        return;
    }

    let max = app.get_list_len_for_tab(&app.active_tab).saturating_sub(1);
    if app.selected_index < max {
        app.selected_index += 1;
        dispatcher::trigger_fetch_logs(app);
    }
}

fn handle_scroll_up(app: &mut App) {
    if app.inspect_popup.is_some() {
        app.inspect_scroll = app.inspect_scroll.saturating_sub(1);
        return;
    }

    if app.logs_focused {
        if let Some(selected) = app.logs_state.selected() {
            if selected > 0 {
                app.logs_state.select(Some(selected - 1));
            }
        }
        return;
    }

    if app.selected_index > 0 {
        app.selected_index -= 1;
        dispatcher::trigger_fetch_logs(app);
    }
}

fn handle_click(app: &mut App, click_x: u16, click_y: u16) {
    if app.inspect_popup.is_some() {
        return;
    }

    if let Ok((cols, rows)) = crossterm::terminal::size() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(Rect {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            });

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        let panel_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(17),
                Constraint::Percentage(17),
                Constraint::Percentage(17),
                Constraint::Percentage(17),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
            ])
            .split(main_chunks[0]);

        // If clicked in the right panel (details/logs)
        if click_x >= main_chunks[1].x
            && click_x < main_chunks[1].x + main_chunks[1].width
            && click_y >= main_chunks[1].y
            && click_y < main_chunks[1].y + main_chunks[1].height
        {
            if matches!(app.active_tab, Tab::Running | Tab::Stopped | Tab::Pods) {
                app.logs_focused = true;
                if !app.container_logs.is_empty() && app.logs_state.selected().is_none() {
                    app.logs_state
                        .select(Some(app.container_logs.len().saturating_sub(1)));
                }
            }
        }
        // If clicked in the left panel
        else if click_x >= main_chunks[0].x
            && click_x < main_chunks[0].x + main_chunks[0].width
            && click_y >= main_chunks[0].y
            && click_y < main_chunks[0].y + main_chunks[0].height
        {
            app.logs_focused = false;
            app.logs_state.select(None);

            let panel_tab_map = [
                (0usize, Tab::Running),
                (1, Tab::Stopped),
                (2, Tab::Images),
                (3, Tab::Volumes),
                (4, Tab::Networks),
                (5, Tab::Pods),
            ];

            for (idx, tab) in &panel_tab_map {
                if click_y >= panel_chunks[*idx].y
                    && click_y < panel_chunks[*idx].y + panel_chunks[*idx].height
                {
                    app.save_current_index();
                    app.active_tab = tab.clone();
                    let list_offset = panel_chunks[*idx].y + 1; // account for border
                    let raw_idx = click_y.saturating_sub(list_offset) as usize;
                    let max = app.get_list_len_for_tab(&app.active_tab).saturating_sub(1);
                    app.selected_index = std::cmp::min(raw_idx, max);
                    app.load_current_index();
                    break;
                }
            }
            dispatcher::trigger_fetch_logs(app);
        }
    }
}
