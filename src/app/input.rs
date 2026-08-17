use crossterm::event::{KeyCode, KeyEvent};

use crate::app::dispatcher;
use crate::app::forms::{
    ConfigureRegistriesForm, CreateContainerForm, CreatePodForm, DirectPullForm, ExecForm,
    SearchImageForm,
};
use crate::app::{App, Tab};

/// Central keyboard event handler.
pub fn handle_key(app: &mut App, key: KeyEvent) {
    // 1. Inspect popup captures keys
    if app.inspect_popup.is_some() {
        handle_inspect_keys(app, key.code);
        return;
    }

    // 2. Create pod form keys
    if app.create_pod_form.is_some() {
        handle_create_pod_keys(app, key.code);
        return;
    }

    // 3. Help popup keys
    if app.show_help_tooltip {
        if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter) {
            app.show_help_tooltip = false;
        }
        return;
    }

    // 4. Direct pull form keys
    if app.direct_pull_form.is_some() {
        handle_direct_pull_keys(app, key.code);
        return;
    }

    // 5. Configure registries form keys
    if app.configure_registries_form.is_some() {
        handle_configure_registries_keys(app, key.code);
        return;
    }

    // 6. Exec form keys
    if app.exec_form.is_some() {
        handle_exec_form_keys(app, key.code);
        return;
    }

    // 7. Search image form keys
    if app.search_image_form.is_some() {
        handle_search_image_keys(app, key.code);
        return;
    }

    // 8. Create container form keys
    if app.create_container_form.is_some() {
        handle_create_container_keys(app, key.code);
        return;
    }

    // 9. Confirmation popup keys
    if app.show_confirmation {
        handle_confirmation_keys(app, key.code);
        return;
    }

    // 10. Focused logs navigation keys
    if app.logs_focused {
        handle_logs_focused_keys(app, key.code);
        return;
    }

    // 11. Main application navigation & action keys
    handle_main_keys(app, key.code);
}

fn handle_inspect_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('g' | 'q') => {
            app.inspect_popup = None;
            app.inspect_scroll = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.inspect_scroll = app.inspect_scroll.saturating_sub(1);
        }
        #[allow(clippy::cast_possible_truncation)]
        KeyCode::Down | KeyCode::Char('j') => {
            let total_lines = app
                .inspect_popup
                .as_ref()
                .map_or(0, |s| s.lines().count() as u16);
            let max_scroll = total_lines.saturating_sub(1);
            if app.inspect_scroll < max_scroll {
                app.inspect_scroll += 1;
            }
        }
        _ => {}
    }
}

fn handle_create_pod_keys(app: &mut App, code: KeyCode) {
    if let Some(form) = &mut app.create_pod_form {
        match code {
            KeyCode::Esc => app.create_pod_form = None,
            KeyCode::Enter => {
                dispatcher::submit_create_pod(app);
            }
            KeyCode::Tab | KeyCode::Down => form.next_field(),
            KeyCode::BackTab | KeyCode::Up => form.prev_field(),
            KeyCode::Backspace => match form.active_field {
                0 => {
                    form.name.pop();
                }
                1 => {
                    form.network.pop();
                }
                _ => {}
            },
            KeyCode::Char(' ') => match form.active_field {
                2 => form.share_pid = !form.share_pid,
                3 => form.share_net = !form.share_net,
                _ => {}
            },
            KeyCode::Char(c) => match form.active_field {
                0 => form.name.push(c),
                1 => form.network.push(c),
                _ => {}
            },
            _ => {}
        }
    }
}

fn handle_direct_pull_keys(app: &mut App, code: KeyCode) {
    let mut pull_image_direct = None;
    if let Some(form) = &mut app.direct_pull_form {
        if app.is_pulling {
            return;
        }
        match code {
            KeyCode::Esc => app.direct_pull_form = None,
            KeyCode::Enter => {
                let text = form.image.trim().to_string();
                if !text.is_empty() {
                    pull_image_direct = Some(text);
                }
            }
            KeyCode::Backspace => {
                form.image.pop();
            }
            KeyCode::Char(c) => form.image.push(c),
            _ => {}
        }
    }

    if let Some(img) = pull_image_direct {
        dispatcher::pull_image_direct(app, img);
    }
}

fn handle_configure_registries_keys(app: &mut App, code: KeyCode) {
    let mut submit_registries = None;
    if let Some(form) = &mut app.configure_registries_form {
        match code {
            KeyCode::Esc => app.configure_registries_form = None,
            KeyCode::Enter => submit_registries = Some(form.registries.clone()),
            KeyCode::Backspace => {
                form.registries.pop();
            }
            KeyCode::Char(c) => form.registries.push(c),
            _ => {}
        }
    }

    if let Some(regs) = submit_registries {
        dispatcher::configure_registries(app, regs);
    }
}

fn handle_exec_form_keys(app: &mut App, code: KeyCode) {
    let mut submit_exec = false;
    if let Some(form) = &mut app.exec_form {
        match code {
            KeyCode::Esc => {
                app.exec_form = None;
                app.pending_exec = None;
            }
            KeyCode::Enter => submit_exec = true,
            KeyCode::Backspace => {
                form.command.pop();
            }
            KeyCode::Char(c) => form.command.push(c),
            _ => {}
        }
    }

    if submit_exec {
        if let Some(c) = app.running.get(app.selected_index) {
            app.pending_exec = Some((c.engine.clone(), c.id.clone()));
        }
    }
}

fn handle_search_image_keys(app: &mut App, code: KeyCode) {
    let mut action_search = None;
    let mut action_pull = None;

    if let Some(form) = &mut app.search_image_form {
        if app.is_pulling {
            return;
        }
        match code {
            KeyCode::Esc => {
                app.search_image_form = None;
            }
            KeyCode::Enter => {
                if form.results.is_empty() {
                    action_search = Some(form.query.clone());
                } else if let Some(res) = form.results.get(form.selected) {
                    action_pull = Some(res.name.clone());
                }
            }
            KeyCode::Down | KeyCode::Tab => {
                if !form.results.is_empty() && form.selected < form.results.len().saturating_sub(1)
                {
                    form.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::BackTab => {
                form.selected = form.selected.saturating_sub(1);
            }
            KeyCode::Backspace => {
                if !form.results.is_empty() {
                    form.results.clear();
                }
                form.query.pop();
            }
            KeyCode::Char(c) => {
                if !form.results.is_empty() {
                    form.results.clear();
                }
                form.query.push(c);
            }
            _ => {}
        }
    }

    if let Some(query) = action_search {
        dispatcher::search_images(app, query);
    } else if let Some(name) = action_pull {
        dispatcher::pull_image_direct(app, name);
    }
}

fn handle_create_container_keys(app: &mut App, code: KeyCode) {
    if let Some(form) = &mut app.create_container_form {
        match code {
            KeyCode::Esc => {
                app.create_container_form = None;
            }
            KeyCode::Enter => {
                dispatcher::submit_create_container(app);
            }
            KeyCode::Tab | KeyCode::Down => form.next_field(),
            KeyCode::BackTab | KeyCode::Up => form.prev_field(),
            KeyCode::Backspace => {
                let field = match form.active_field {
                    0 => &mut form.name,
                    1 => &mut form.command,
                    2 => &mut form.ports,
                    _ => &mut form.env,
                };
                field.pop();
            }
            KeyCode::Char(c) => {
                let field = match form.active_field {
                    0 => &mut form.name,
                    1 => &mut form.command,
                    2 => &mut form.ports,
                    _ => &mut form.env,
                };
                field.push(c);
            }
            _ => {}
        }
    }
}

fn handle_confirmation_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.show_confirmation = false;
            if let Some((resource_type, engine, id, action)) = app.pending_action.take() {
                dispatcher::execute_resource_action(app, resource_type, engine, id, action);
            }
        }
        KeyCode::Char('a') => {
            app.show_confirmation = false;
            if let Some((resource_type, engine, id, action)) = app.pending_action.take() {
                let related = app.get_related_resources(&resource_type, &id);
                for (r_type, r_engine, r_id) in related {
                    dispatcher::execute_resource_action(
                        app,
                        r_type,
                        r_engine,
                        r_id,
                        action.clone(),
                    );
                }
                dispatcher::execute_resource_action(app, resource_type, engine, id, action);
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.show_confirmation = false;
            app.pending_action = None;
        }
        _ => {}
    }
}

fn handle_logs_focused_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Left | KeyCode::Char('h' | 'H' | 'q') => {
            app.logs_focused = false;
            app.logs_state.select(None);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(selected) = app.logs_state.selected() {
                if selected > 0 {
                    app.logs_state.select(Some(selected - 1));
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(selected) = app.logs_state.selected() {
                if selected < app.container_logs.len().saturating_sub(1) {
                    app.logs_state.select(Some(selected + 1));
                }
            }
        }
        KeyCode::Char('y' | 'c') => {
            if let Some(selected) = app.logs_state.selected() {
                if let Some(line) = app.container_logs.get(selected) {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(line.clone());
                    }
                }
            }
        }
        KeyCode::Char('x' | 'e' | 'i') => {
            if matches!(app.active_tab, Tab::Running) {
                if let Some(c) = app.running.get(app.selected_index) {
                    app.pending_exec = Some((c.engine.clone(), c.id.clone()));
                    app.exec_form = Some(ExecForm {
                        command: "/bin/sh".to_string(),
                    });
                    app.logs_focused = false;
                    app.logs_state.select(None);
                }
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_lines)]
fn handle_main_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => {
            if app.logs_focused {
                app.logs_focused = false;
                app.logs_state.select(None);
                app.active_tab = Tab::Running;
                app.selected_index = app.running_index;
            } else if app.active_tab == Tab::Pods {
                app.logs_focused = true;
            } else {
                app.switch_to_tab(app.active_tab.next());
            }
            dispatcher::trigger_fetch_logs(app);
        }
        KeyCode::Right | KeyCode::Char('l' | 'L') => {
            if matches!(app.active_tab, Tab::Running | Tab::Stopped | Tab::Pods) {
                app.logs_focused = true;
                return;
            }
            app.switch_to_tab(app.active_tab.next());
            dispatcher::trigger_fetch_logs(app);
        }
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h' | 'H') => {
            if app.logs_focused {
                app.logs_focused = false;
                app.logs_state.select(None);
                return;
            }
            app.switch_to_tab(app.active_tab.prev());
            dispatcher::trigger_fetch_logs(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = app.get_list_len_for_tab(&app.active_tab).saturating_sub(1);
            if app.selected_index < max {
                app.selected_index += 1;
                dispatcher::trigger_fetch_logs(app);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
                dispatcher::trigger_fetch_logs(app);
            }
        }
        KeyCode::Char('r') => {
            app.status_message = None;
            dispatcher::trigger_refresh_data(app);
        }
        KeyCode::Char('E') => {
            app.engine_view.next();
            app.selected_index = 0;
            dispatcher::trigger_refresh_data(app);
        }
        KeyCode::Char('s') => match app.active_tab {
            Tab::Running | Tab::Pods => dispatcher::handle_action(app, "stop"),
            Tab::Stopped => dispatcher::handle_action(app, "start"),
            _ => {}
        },
        KeyCode::Char('/') => {
            if matches!(app.active_tab, Tab::Images) {
                app.search_image_form = Some(SearchImageForm::default());
            }
        }
        KeyCode::Char('p') => {
            if matches!(app.active_tab, Tab::Images) {
                app.direct_pull_form = Some(DirectPullForm::default());
            }
        }
        KeyCode::Char('P') => {
            if matches!(app.active_tab, Tab::Pods) {
                app.create_pod_form = Some(CreatePodForm::default());
            }
        }
        KeyCode::Char('c') => {
            if matches!(app.active_tab, Tab::Images) {
                app.configure_registries_form = Some(ConfigureRegistriesForm::default());
            }
        }
        KeyCode::Char('i' | 'e') => {
            if matches!(app.active_tab, Tab::Running) {
                if let Some(c) = app.running.get(app.selected_index) {
                    app.pending_exec = Some((c.engine.clone(), c.id.clone()));
                    app.exec_form = Some(ExecForm {
                        command: "/bin/sh".to_string(),
                    });
                    if app.logs_focused {
                        app.logs_focused = false;
                        app.logs_state.select(None);
                    }
                }
            }
        }
        KeyCode::Char('x') => {
            if matches!(app.active_tab, Tab::Running)
                && app.running.get(app.selected_index).is_some()
            {
                app.exec_form = Some(ExecForm {
                    command: String::new(),
                });
                app.logs_focused = false;
                app.logs_state.select(None);
            }
        }
        KeyCode::Char('S' | 'u') => {
            if matches!(app.active_tab, Tab::Stopped) {
                dispatcher::handle_action(app, "start");
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => dispatcher::handle_action(app, "rm"),
        KeyCode::Enter => handle_primary_action(app),
        KeyCode::Char('?') => app.show_help_tooltip = true,
        KeyCode::Char('g') => dispatcher::trigger_inspect(app),
        _ => {}
    }
}

fn handle_primary_action(app: &mut App) {
    match app.active_tab {
        Tab::Running | Tab::Stopped | Tab::Pods => {
            app.logs_focused = true;
            if !app.container_logs.is_empty() && app.logs_state.selected().is_none() {
                app.logs_state
                    .select(Some(app.container_logs.len().saturating_sub(1)));
            }
        }
        Tab::Images if app.images.get(app.selected_index).is_some() => {
            app.create_container_form = Some(CreateContainerForm::default());
        }
        _ => {}
    }
}
