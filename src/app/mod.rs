pub mod forms;
pub mod state;

use std::sync::Arc;
use tokio::sync::mpsc;
use ratatui::{backend::Backend, Terminal, widgets::ListState};
use crossterm::event::KeyCode;

use crate::action::Action;
use crate::events::EventHandler;
use crate::podman::{Container, EngineClient, Image, LocalEngines, Network, Pod, Volume};
use crate::ui;
pub use forms::*;
pub use state::{EngineView, Tab};
use anyhow::Result;

pub struct App {
    pub should_quit: bool,
    pub active_tab: Tab,
    pub running: Vec<Container>,
    pub stopped: Vec<Container>,
    pub images: Vec<Image>,
    pub volumes: Vec<Volume>,
    pub networks: Vec<Network>,
    pub pods: Vec<Pod>,
    pub selected_index: usize,
    pub running_index: usize,
    pub stopped_index: usize,
    pub images_index: usize,
    pub volumes_index: usize,
    pub networks_index: usize,
    pub pods_index: usize,
    pub show_confirmation: bool,
    pub create_container_form: Option<CreateContainerForm>,
    pub search_image_form: Option<SearchImageForm>,
    pub is_pulling: bool,
    pub container_logs: Vec<String>,
    pub logs_focused: bool,
    pub logs_state: ListState,
    pub pending_exec: Option<(String, String)>, // (engine, id)
    pub engine_view: EngineView,
    pub engine_client: Arc<Box<dyn EngineClient>>,
    pub show_help_tooltip: bool,
    pub direct_pull_form: Option<DirectPullForm>,
    pub configure_registries_form: Option<ConfigureRegistriesForm>,
    pub exec_form: Option<ExecForm>,
    pub pending_action: Option<(Tab, String, String, String)>,
    pub available_engines: Vec<String>,
    pub action_tx: Option<mpsc::UnboundedSender<Action>>,
    pub status_message: Option<String>,
    pub inspect_popup: Option<String>,
    pub inspect_scroll: u16,
    pub create_pod_form: Option<CreatePodForm>,
}

impl App {
    pub fn new() -> Self {
        let mut available_engines = Vec::new();
        // Fast synchronous check
        if std::process::Command::new("docker").arg("--version").output().is_ok() {
            available_engines.push("docker".to_string());
        }
        if std::process::Command::new("podman").arg("--version").output().is_ok() {
            available_engines.push("podman".to_string());
        }

        Self {
            should_quit: false,
            active_tab: Tab::Running,
            running: Vec::new(),
            stopped: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
            pods: Vec::new(),
            selected_index: 0,
            running_index: 0,
            stopped_index: 0,
            images_index: 0,
            volumes_index: 0,
            networks_index: 0,
            pods_index: 0,
            show_confirmation: false,
            create_container_form: None,
            search_image_form: None,
            is_pulling: false,
            container_logs: Vec::new(),
            logs_focused: false,
            logs_state: ListState::default(),
            pending_exec: None,
            engine_view: EngineView::Both,
            engine_client: Arc::new(Box::new(LocalEngines)),
            show_help_tooltip: false,
            direct_pull_form: None,
            configure_registries_form: None,
            exec_form: None,
            pending_action: None,
            available_engines,
            action_tx: None,
            status_message: None,
            inspect_popup: None,
            inspect_scroll: 0,
            create_pod_form: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_client(client: Box<dyn EngineClient>) -> Self {
        let mut app = Self::new();
        app.engine_client = Arc::new(client);
        app
    }

    pub fn get_active_engines(&self) -> Vec<String> {
        let desired = match self.engine_view {
            EngineView::Both => vec!["docker".to_string(), "podman".to_string()],
            EngineView::Docker => vec!["docker".to_string()],
            EngineView::Podman => vec!["podman".to_string()],
        };
        desired.into_iter().filter(|e| self.available_engines.contains(e)).collect()
    }

    pub fn get_default_target_engine(&self) -> String {
        self.get_active_engines()
            .into_iter()
            .next()
            .unwrap_or_else(|| "docker".into())
    }

    // --- Extracted navigation helpers (used in future refactoring) ---

    #[allow(dead_code)]
    fn get_tab_list(&self) -> Vec<Tab> {
        vec![
            Tab::Running,
            Tab::Stopped,
            Tab::Images,
            Tab::Volumes,
            Tab::Networks,
            Tab::Pods,
        ]
    }

    #[allow(dead_code)]
    fn next_tab(&mut self) {
        let tabs = self.get_tab_list();
        if let Some(pos) = tabs.iter().position(|t| *t == self.active_tab) {
            let next_pos = (pos + 1) % tabs.len();
            self.switch_to_tab(tabs[next_pos].clone());
        }
    }

    #[allow(dead_code)]
    fn prev_tab(&mut self) {
        let tabs = self.get_tab_list();
        if let Some(pos) = tabs.iter().position(|t| *t == self.active_tab) {
            let prev_pos = if pos == 0 { tabs.len() - 1 } else { pos - 1 };
            self.switch_to_tab(tabs[prev_pos].clone());
        }
    }

    #[allow(dead_code)]
    fn switch_to_tab(&mut self, new_tab: Tab) {
        // Save current index
        self.save_current_index();
        // Switch tab
        self.active_tab = new_tab;
        // Load new index
        self.load_current_index();
        // Clear logs focus
        self.logs_focused = false;
        self.logs_state.select(None);
        self.trigger_fetch_logs();
    }

    fn save_current_index(&mut self) {
        match self.active_tab {
            Tab::Running => self.running_index = self.selected_index,
            Tab::Stopped => self.stopped_index = self.selected_index,
            Tab::Images => self.images_index = self.selected_index,
            Tab::Volumes => self.volumes_index = self.selected_index,
            Tab::Networks => self.networks_index = self.selected_index,
            Tab::Pods => self.pods_index = self.selected_index,
        }
    }

    fn load_current_index(&mut self) {
        self.selected_index = match self.active_tab {
            Tab::Running => self.running_index,
            Tab::Stopped => self.stopped_index,
            Tab::Images => self.images_index,
            Tab::Volumes => self.volumes_index,
            Tab::Networks => self.networks_index,
            Tab::Pods => self.pods_index,
        };
    }

    fn get_list_len_for_tab(&self, tab: &Tab) -> usize {
        match tab {
            Tab::Running => self.running.len(),
            Tab::Stopped => self.stopped.len(),
            Tab::Images => self.images.len(),
            Tab::Volumes => self.volumes.len(),
            Tab::Networks => self.networks.len(),
            Tab::Pods => self.pods.len(),
        }
    }

    fn get_selected_resource(&self) -> Option<(Tab, String, String)> {
        match self.active_tab {
            Tab::Running | Tab::Stopped => {
                let list = if self.active_tab == Tab::Running { &self.running } else { &self.stopped };
                list.get(self.selected_index)
                    .map(|c| (self.active_tab.clone(), c.engine.clone(), c.id.clone()))
            }
            Tab::Images => {
                self.images.get(self.selected_index)
                    .map(|i| (Tab::Images, i.engine.clone(), i.id.clone()))
            }
            Tab::Volumes => {
                self.volumes.get(self.selected_index)
                    .map(|v| (Tab::Volumes, v.engine.clone(), v.name.clone()))
            }
            Tab::Networks => {
                self.networks.get(self.selected_index)
                    .map(|n| (Tab::Networks, n.engine.clone(), n.id.clone()))
            }
            Tab::Pods => {
                self.pods.get(self.selected_index)
                    .map(|p| (Tab::Pods, p.engine.clone(), p.id.clone()))
            }
        }
    }

    fn spawn_resource_action(&self, tab: Tab, engine: String, id: String, action: String) {
        let client = self.engine_client.clone();
        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            let result = match tab {
                Tab::Running | Tab::Stopped => client.action_container(&engine, &id, &action).await,
                Tab::Images => client.action_image(&engine, &id, &action).await,
                Tab::Volumes => client.action_volume(&engine, &id, &action).await,
                Tab::Networks => client.action_network(&engine, &id, &action).await,
                Tab::Pods => client.action_pod(&engine, &id, &action).await,
            };
            if let Some(tx) = tx {
                match result {
                    Ok(()) => { let _ = tx.send(Action::ActionComplete); }
                    Err(e) => { let _ = tx.send(Action::Error { message: e.to_string() }); }
                }
            }
        });
    }

    fn propagate_error(&mut self, message: String) {
        self.status_message = Some(message);
    }

    // --- End extracted helpers ---

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut events = EventHandler::new(250);
        self.action_tx = Some(events.sender.clone());

        self.trigger_refresh_data();

        while !self.should_quit {
            terminal.draw(|f| ui::draw(f, self))?;

            if let Some(action) = events.next().await {
                self.update(action);
            }

            if let Some((engine, cmd)) = self.pending_exec.take() {
                drop(events);
                self.action_tx = None;

                crossterm::terminal::disable_raw_mode()?;
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::LeaveAlternateScreen,
                    crossterm::event::DisableMouseCapture
                )?;

                let mut args = vec!["exec", "-it", &cmd];
                let custom_cmd = self
                    .exec_form
                    .take()
                    .map(|f| f.command)
                    .unwrap_or_else(|| "/bin/sh".to_string());

                let custom_args: Vec<&str> = custom_cmd.split_whitespace().collect();
                if !custom_args.is_empty() {
                    args.extend(custom_args);
                } else {
                    args.push("/bin/sh");
                }

                let mut child = std::process::Command::new(engine).args(&args).spawn()?;
                let _ = child.wait()?;

                crossterm::terminal::enable_raw_mode()?;
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::EnterAlternateScreen,
                    crossterm::event::EnableMouseCapture
                )?;
                terminal.clear()?;

                events = EventHandler::new(250);
                self.action_tx = Some(events.sender.clone());
                self.trigger_refresh_data();
            }
        }

        Ok(())
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Tick => (),
            Action::DataRefreshed { running, stopped, images, volumes, networks, pods } => {
                self.running = running;
                self.stopped = stopped;
                self.images = images;
                self.volumes = volumes;
                self.networks = networks;
                self.pods = pods;
                self.running_index = std::cmp::min(self.running_index, self.running.len().saturating_sub(1));
                self.stopped_index = std::cmp::min(self.stopped_index, self.stopped.len().saturating_sub(1));
                self.images_index = std::cmp::min(self.images_index, self.images.len().saturating_sub(1));
                self.volumes_index = std::cmp::min(self.volumes_index, self.volumes.len().saturating_sub(1));
                self.networks_index = std::cmp::min(self.networks_index, self.networks.len().saturating_sub(1));
                self.pods_index = std::cmp::min(self.pods_index, self.pods.len().saturating_sub(1));

                self.load_current_index();
                self.trigger_fetch_logs();
            }
            Action::LogsRefreshed { logs } => {
                self.container_logs = logs;
                if self.logs_focused && self.container_logs.is_empty() {
                    self.logs_state.select(None);
                } else if self.logs_focused {
                    let max = self.container_logs.len().saturating_sub(1);
                    if let Some(sel) = self.logs_state.selected() {
                        self.logs_state.select(Some(std::cmp::min(sel, max)));
                    } else {
                        self.logs_state.select(Some(max));
                    }
                } else {
                    self.logs_state.select(None);
                }
            }
            Action::SearchResults { results } => {
                if let Some(form) = &mut self.search_image_form {
                    form.results = results;
                    form.selected = 0;
                    form.is_searching = false;
                }
            }
            Action::InspectResult { output } => {
                self.inspect_popup = Some(output);
                self.inspect_scroll = 0;
            }
            Action::PullComplete => {
                self.is_pulling = false;
                self.direct_pull_form = None;
                self.search_image_form = None;
                self.trigger_refresh_data();
            }
            Action::ActionComplete => {
                self.status_message = None;
                self.trigger_refresh_data();
            }
            Action::Error { message } => {
                self.propagate_error(message);
            }
            Action::Key(k) => self.handle_key(k),
            Action::Mouse(m) => self.handle_mouse(m),
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        // Inspect popup takes priority
        if self.inspect_popup.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('g') | KeyCode::Char('q') => {
                    self.inspect_popup = None;
                    self.inspect_scroll = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.inspect_scroll = self.inspect_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let total_lines = self.inspect_popup.as_ref()
                        .map(|s| s.lines().count() as u16)
                        .unwrap_or(0);
                    let max_scroll = total_lines.saturating_sub(1);
                    if self.inspect_scroll < max_scroll {
                        self.inspect_scroll += 1;
                    }
                }
                _ => {}
            }
            return;
        }

        // Create pod form
        if let Some(form) = &mut self.create_pod_form {
            match key.code {
                KeyCode::Esc => self.create_pod_form = None,
                KeyCode::Enter => {
                    self.submit_create_pod();
                    self.create_pod_form = None;
                }
                KeyCode::Tab | KeyCode::Down => {
                    form.active_field = (form.active_field + 1) % 4;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    form.active_field = form.active_field.checked_sub(1).unwrap_or(3);
                }
                KeyCode::Backspace => {
                    match form.active_field {
                        0 => { form.name.pop(); }
                        1 => { form.network.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Char(' ') => {
                    match form.active_field {
                        2 => { form.share_pid = !form.share_pid; }
                        3 => { form.share_net = !form.share_net; }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    match form.active_field {
                        0 => form.name.push(c),
                        1 => form.network.push(c),
                        _ => {}
                    }
                }
                _ => {}
            }
            return;
        }

        if self.show_help_tooltip {
            if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter) {
                self.show_help_tooltip = false;
            }
            return;
        }

        let mut pull_image_direct = None;
        if let Some(form) = &mut self.direct_pull_form {
            if self.is_pulling {
                return;
            }
            match key.code {
                KeyCode::Esc => self.direct_pull_form = None,
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
            if pull_image_direct.is_none() {
                return;
            }
        }
        if let Some(img) = pull_image_direct {
            self.is_pulling = true;
            let target = self.get_default_target_engine();
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                let _ = client.pull_image(&target, &img).await;
                if let Some(tx) = tx {
                    let _ = tx.send(Action::PullComplete);
                }
            });
            return;
        }

        let mut submit_registries = None;
        if let Some(form) = &mut self.configure_registries_form {
            match key.code {
                KeyCode::Esc => self.configure_registries_form = None,
                KeyCode::Enter => submit_registries = Some(form.registries.clone()),
                KeyCode::Backspace => {
                    form.registries.pop();
                }
                KeyCode::Char(c) => form.registries.push(c),
                _ => {}
            }
            if submit_registries.is_none() {
                return;
            }
        }
        if let Some(regs) = submit_registries {
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                let _ = client.configure_registries(&regs).await;
                if let Some(tx) = tx {
                    let _ = tx.send(Action::ActionComplete);
                }
            });
            self.configure_registries_form = None;
            return;
        }

        let mut submit_exec = None;
        if let Some(form) = &mut self.exec_form {
            match key.code {
                KeyCode::Esc => {
                    self.exec_form = None;
                    self.pending_exec = None;
                }
                KeyCode::Enter => {
                    submit_exec = Some(form.command.clone());
                }
                KeyCode::Backspace => {
                    form.command.pop();
                }
                KeyCode::Char(c) => form.command.push(c),
                _ => {}
            }
            if submit_exec.is_some() {
                if let Some(c) = self.running.get(self.selected_index) {
                    self.pending_exec = Some((c.engine.clone(), c.id.clone()));
                }
                return;
            } else {
                return;
            }
        }

        let mut action_search = None;
        let mut action_pull = None;

        if let Some(form) = &mut self.search_image_form {
            if self.is_pulling {
                return;
            }
            match key.code {
                KeyCode::Esc => {
                    self.search_image_form = None;
                }
                KeyCode::Enter => {
                    if form.results.is_empty() {
                        action_search = Some(form.query.clone());
                    } else if let Some(res) = form.results.get(form.selected) {
                        action_pull = Some(res.name.clone());
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    if !form.results.is_empty()
                        && form.selected < form.results.len().saturating_sub(1)
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

            if action_search.is_none() && action_pull.is_none() {
                return;
            }
        }

        if let Some(query) = action_search {
            if let Some(form) = &mut self.search_image_form {
                form.is_searching = true;
            }
            let engines = self.get_active_engines();
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                if let Ok(results) = client.search_images(&engines, &query).await {
                    if let Some(tx) = tx {
                        let _ = tx.send(Action::SearchResults { results });
                    }
                }
            });
            return;
        }

        if let Some(name) = action_pull {
            self.is_pulling = true;
            let target_engine = self.get_default_target_engine();
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                let _ = client.pull_image(&target_engine, &name).await;
                if let Some(tx) = tx {
                    let _ = tx.send(Action::PullComplete);
                }
            });
            return;
        }

        if let Some(form) = &mut self.create_container_form {
            match key.code {
                KeyCode::Esc => {
                    self.create_container_form = None;
                }
                KeyCode::Enter => {
                    self.submit_create_container();
                    self.create_container_form = None;
                }
                KeyCode::Tab | KeyCode::Down => {
                    form.active_field = (form.active_field + 1) % 4;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    form.active_field = form.active_field.checked_sub(1).unwrap_or(3);
                }
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
            return;
        }
        if self.show_confirmation {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    self.show_confirmation = false;
                    if let Some((resource_type, engine, id, action)) = self.pending_action.take() {
                        self.execute_resource_action(resource_type, engine, id, action);
                    }
                }
                KeyCode::Char('a') => {
                    self.show_confirmation = false;
                    if let Some((resource_type, engine, id, action)) = self.pending_action.take() {
                        let related = self.get_related_resources(&resource_type, &id);
                        for (r_type, r_engine, r_id) in related {
                            self.execute_resource_action(r_type, r_engine, r_id, action.clone());
                        }
                        self.execute_resource_action(resource_type, engine, id, action);
                    }
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.show_confirmation = false;
                    self.pending_action = None;
                }
                _ => {}
            }
            return;
        }

        if self.logs_focused {
            match key.code {
                KeyCode::Esc
                | KeyCode::Left
                | KeyCode::Char('h')
                | KeyCode::Char('H')
                | KeyCode::Char('q') => {
                    self.logs_focused = false;
                    self.logs_state.select(None);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(selected) = self.logs_state.selected() {
                        if selected > 0 {
                            self.logs_state.select(Some(selected - 1));
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(selected) = self.logs_state.selected() {
                        if selected < self.container_logs.len().saturating_sub(1) {
                            self.logs_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Char('y') | KeyCode::Char('c') => {
                    if let Some(selected) = self.logs_state.selected() {
                        if let Some(line) = self.container_logs.get(selected) {
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                let _ = clipboard.set_text(line.clone());
                            }
                        }
                    }
                }
                KeyCode::Char('x') | KeyCode::Char('e') | KeyCode::Char('i') => {
                    if matches!(self.active_tab, Tab::Running) {
                        if let Some(c) = self.running.get(self.selected_index) {
                            self.pending_exec = Some((c.engine.clone(), c.id.clone()));
                            self.exec_form = Some(ExecForm {
                                command: "/bin/sh".to_string(),
                            });
                            self.logs_focused = false;
                            self.logs_state.select(None);
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                if self.logs_focused {
                    self.logs_focused = false;
                    self.logs_state.select(None);
                    self.active_tab = Tab::Running;
                    self.selected_index = self.running_index;
                } else {
                    self.save_current_index();
                    match self.active_tab {
                        Tab::Running => {
                            self.active_tab = Tab::Stopped;
                        }
                        Tab::Stopped => {
                            self.active_tab = Tab::Images;
                        }
                        Tab::Images => {
                            self.active_tab = Tab::Volumes;
                        }
                        Tab::Volumes => {
                            self.active_tab = Tab::Networks;
                        }
                        Tab::Networks => {
                            self.active_tab = Tab::Pods;
                        }
                        Tab::Pods => {
                            self.logs_focused = true;
                        }
                    }
                    self.load_current_index();
                }
                self.trigger_fetch_logs();
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                if matches!(self.active_tab, Tab::Running | Tab::Stopped | Tab::Pods) {
                    self.logs_focused = true;
                    return;
                }
                self.save_current_index();
                match self.active_tab {
                    Tab::Running => {
                        self.active_tab = Tab::Stopped;
                    }
                    Tab::Stopped => {
                        self.active_tab = Tab::Images;
                    }
                    Tab::Images => {
                        self.active_tab = Tab::Volumes;
                    }
                    Tab::Volumes => {
                        self.active_tab = Tab::Networks;
                    }
                    Tab::Networks => {
                        self.active_tab = Tab::Pods;
                    }
                    Tab::Pods => {
                        self.active_tab = Tab::Running;
                    }
                };
                self.load_current_index();
                self.trigger_fetch_logs();
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                if self.logs_focused {
                    self.logs_focused = false;
                    self.logs_state.select(None);
                    return;
                }
                self.save_current_index();
                match self.active_tab {
                    Tab::Running => {
                        self.active_tab = Tab::Pods;
                    }
                    Tab::Stopped => {
                        self.active_tab = Tab::Running;
                    }
                    Tab::Images => {
                        self.active_tab = Tab::Stopped;
                    }
                    Tab::Volumes => {
                        self.active_tab = Tab::Images;
                    }
                    Tab::Networks => {
                        self.active_tab = Tab::Volumes;
                    }
                    Tab::Pods => {
                        self.active_tab = Tab::Networks;
                    }
                };
                self.load_current_index();
                self.trigger_fetch_logs();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.get_list_len_for_tab(&self.active_tab).saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
                    self.trigger_fetch_logs();
                } else if !self.logs_focused {
                    self.save_current_index();
                    match self.active_tab {
                        Tab::Running => {
                            self.active_tab = Tab::Stopped;
                        }
                        Tab::Stopped => {
                            self.active_tab = Tab::Images;
                        }
                        Tab::Images => {
                            self.active_tab = Tab::Volumes;
                        }
                        Tab::Volumes => {
                            self.active_tab = Tab::Networks;
                        }
                        Tab::Networks => {
                            self.active_tab = Tab::Pods;
                        }
                        Tab::Pods => {
                            self.logs_focused = true;
                            return;
                        }
                    }
                    self.load_current_index();
                    self.trigger_fetch_logs();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.trigger_fetch_logs();
                } else if !self.logs_focused {
                    self.save_current_index();
                    match self.active_tab {
                        Tab::Running => {
                            self.logs_focused = true;
                            return;
                        }
                        Tab::Stopped => {
                            self.active_tab = Tab::Running;
                        }
                        Tab::Images => {
                            self.active_tab = Tab::Stopped;
                        }
                        Tab::Volumes => {
                            self.active_tab = Tab::Images;
                        }
                        Tab::Networks => {
                            self.active_tab = Tab::Volumes;
                        }
                        Tab::Pods => {
                            self.active_tab = Tab::Networks;
                        }
                    }
                    self.load_current_index();
                    self.trigger_fetch_logs();
                }
            }
            KeyCode::Char('r') => {
                self.status_message = None;
                self.trigger_refresh_data();
            }
            KeyCode::Char('E') => {
                self.engine_view.next();
                self.selected_index = 0;
                self.trigger_refresh_data();
            }
            KeyCode::Char('s') => {
                match self.active_tab {
                    Tab::Running => self.handle_action("stop"),
                    Tab::Stopped => self.handle_action("start"),
                    Tab::Pods => self.handle_action("stop"),
                    _ => {}
                }
            }
            KeyCode::Char('/') => {
                if matches!(self.active_tab, Tab::Images) {
                    self.search_image_form = Some(SearchImageForm::default());
                }
            }
            KeyCode::Char('p') => {
                if matches!(self.active_tab, Tab::Images) {
                    self.direct_pull_form = Some(DirectPullForm::default());
                }
            }
            KeyCode::Char('P') => {
                if matches!(self.active_tab, Tab::Pods) {
                    self.create_pod_form = Some(CreatePodForm::default());
                }
            }
            KeyCode::Char('c') => {
                if matches!(self.active_tab, Tab::Images) {
                    self.configure_registries_form = Some(ConfigureRegistriesForm::default());
                }
            }
            KeyCode::Char('i') | KeyCode::Char('e') => {
                if matches!(self.active_tab, Tab::Running) {
                    if let Some(c) = self.running.get(self.selected_index) {
                        self.pending_exec = Some((c.engine.clone(), c.id.clone()));
                        self.exec_form = Some(ExecForm {
                            command: "/bin/sh".to_string(),
                        });
                        if self.logs_focused {
                            self.logs_focused = false;
                            self.logs_state.select(None);
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                if matches!(self.active_tab, Tab::Running)
                    && self.running.get(self.selected_index).is_some() {
                        self.exec_form = Some(ExecForm {
                            command: String::new(),
                        });
                        self.logs_focused = false;
                        self.logs_state.select(None);
                    }
            }
            KeyCode::Char('S') | KeyCode::Char('u') => {
                if matches!(self.active_tab, Tab::Stopped) {
                    self.handle_action("start");
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => self.handle_action("rm"),
            KeyCode::Enter => self.handle_primary_action(),
            KeyCode::Char('?') => self.show_help_tooltip = true,
            KeyCode::Char('g') => self.trigger_inspect(),
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        if self.show_confirmation || self.create_container_form.is_some() || self.create_pod_form.is_some() {
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollDown => {
                // Inspect popup captures scroll
                if self.inspect_popup.is_some() {
                    let total_lines = self.inspect_popup.as_ref()
                        .map(|s| s.lines().count() as u16)
                        .unwrap_or(0);
                    let max_scroll = total_lines.saturating_sub(1);
                    if self.inspect_scroll < max_scroll {
                        self.inspect_scroll += 1;
                    }
                    return;
                }
                if self.logs_focused {
                    if let Some(selected) = self.logs_state.selected() {
                        if selected < self.container_logs.len().saturating_sub(1) {
                            self.logs_state.select(Some(selected + 1));
                        }
                    }
                    return;
                }
                let max = self.get_list_len_for_tab(&self.active_tab).saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
                    self.trigger_fetch_logs();
                }
            }
            MouseEventKind::ScrollUp => {
                // Inspect popup captures scroll
                if self.inspect_popup.is_some() {
                    self.inspect_scroll = self.inspect_scroll.saturating_sub(1);
                    return;
                }
                if self.logs_focused {
                    if let Some(selected) = self.logs_state.selected() {
                        if selected > 0 {
                            self.logs_state.select(Some(selected - 1));
                        }
                    }
                    return;
                }
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.trigger_fetch_logs();
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Block clicks when inspect popup is open
                if self.inspect_popup.is_some() {
                    return;
                }
                if let Ok((cols, rows)) = crossterm::terminal::size() {
                    use ratatui::layout::{Constraint, Direction, Layout, Rect};
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(0),
                            Constraint::Length(1),
                        ])
                        .split(Rect { x: 0, y: 0, width: cols, height: rows });
                        
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

                    let click_x = mouse.column;
                    let click_y = mouse.row;

                    // If clicked in the right panel (details/logs)
                    if click_x >= main_chunks[1].x && click_x < main_chunks[1].x + main_chunks[1].width 
                        && click_y >= main_chunks[1].y && click_y < main_chunks[1].y + main_chunks[1].height {
                        if matches!(self.active_tab, Tab::Running | Tab::Stopped) {
                            self.logs_focused = true;
                            if !self.container_logs.is_empty() && self.logs_state.selected().is_none() {
                                self.logs_state.select(Some(self.container_logs.len().saturating_sub(1)));
                            }
                        }
                    } 
                    // If clicked in the left panel
                    else if click_x >= main_chunks[0].x && click_x < main_chunks[0].x + main_chunks[0].width 
                        && click_y >= main_chunks[0].y && click_y < main_chunks[0].y + main_chunks[0].height {
                        
                        self.logs_focused = false;
                        self.logs_state.select(None);

                        let panel_tab_map = [
                            (0usize, Tab::Running),
                            (1, Tab::Stopped),
                            (2, Tab::Images),
                            (3, Tab::Volumes),
                            (4, Tab::Networks),
                            (5, Tab::Pods),
                        ];

                        for (idx, tab) in &panel_tab_map {
                            if click_y >= panel_chunks[*idx].y && click_y < panel_chunks[*idx].y + panel_chunks[*idx].height {
                                self.save_current_index();
                                self.active_tab = tab.clone();
                                let list_offset = panel_chunks[*idx].y + 1; // account for border
                                let raw_idx = click_y.saturating_sub(list_offset) as usize;
                                let max = self.get_list_len_for_tab(&self.active_tab).saturating_sub(1);
                                self.selected_index = std::cmp::min(raw_idx, max);
                                self.load_current_index();
                                break;
                            }
                        }
                        self.trigger_fetch_logs();
                    }
                }
            }
            _ => {}
        }
    }

    fn trigger_refresh_data(&mut self) {
        let engines = self.get_active_engines();
        let client = self.engine_client.clone();
        let tx = self.action_tx.clone();
        
        tokio::spawn(async move {
            let mut running = Vec::new();
            let mut stopped = Vec::new();
            let containers_result = client.get_containers(&engines).await;
            match containers_result {
                Ok(c) => {
                    running = c.iter().filter(|x| x.is_running()).cloned().collect();
                    stopped = c.iter().filter(|x| !x.is_running()).cloned().collect();
                }
                Err(e) => {
                    if let Some(tx) = &tx {
                        let _ = tx.send(Action::Error { message: format!("Containers: {}", e) });
                    }
                }
            }
            let images = match client.get_images(&engines).await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(tx) = &tx {
                        let _ = tx.send(Action::Error { message: format!("Images: {}", e) });
                    }
                    Vec::new()
                }
            };
            let volumes = match client.get_volumes(&engines).await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(tx) = &tx {
                        let _ = tx.send(Action::Error { message: format!("Volumes: {}", e) });
                    }
                    Vec::new()
                }
            };
            let networks = match client.get_networks(&engines).await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(tx) = &tx {
                        let _ = tx.send(Action::Error { message: format!("Networks: {}", e) });
                    }
                    Vec::new()
                }
            };
            let pods = match client.get_pods(&engines).await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(tx) = &tx {
                        let _ = tx.send(Action::Error { message: format!("Pods: {}", e) });
                    }
                    Vec::new()
                }
            };

            if let Some(tx) = tx {
                let _ = tx.send(Action::DataRefreshed { running, stopped, images, volumes, networks, pods });
            }
        });
    }

    fn trigger_fetch_logs(&mut self) {
        let (engine, id, is_pod) = match self.active_tab {
            Tab::Running => {
                if let Some(c) = self.running.get(self.selected_index) {
                    (Some(c.engine.clone()), Some(c.id.clone()), false)
                } else {
                    (None, None, false)
                }
            }
            Tab::Stopped => {
                if let Some(c) = self.stopped.get(self.selected_index) {
                    (Some(c.engine.clone()), Some(c.id.clone()), false)
                } else {
                    (None, None, false)
                }
            }
            Tab::Pods => {
                if let Some(p) = self.pods.get(self.selected_index) {
                    (Some(p.engine.clone()), Some(p.id.clone()), true)
                } else {
                    (None, None, false)
                }
            }
            _ => (None, None, false),
        };

        if let (Some(engine), Some(id)) = (engine, id) {
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                let logs = if is_pod {
                    client.get_pod_logs(&engine, &id).await.unwrap_or_default()
                } else {
                    client.get_container_logs(&engine, &id).await.unwrap_or_default()
                };
                if let Some(tx) = tx {
                    let _ = tx.send(Action::LogsRefreshed { logs });
                }
            });
        } else {
            self.container_logs.clear();
            self.logs_state.select(None);
        }
    }

    fn trigger_inspect(&mut self) {
        let resource = self.get_selected_resource();
        if let Some((tab, engine, id)) = resource {
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                let result = match tab {
                    Tab::Pods => client.get_container_inspect(&engine, &id).await,
                    _ => client.get_container_inspect(&engine, &id).await,
                };
                if let Some(tx) = tx {
                    match result {
                        Ok(output) => { let _ = tx.send(Action::InspectResult { output }); }
                        Err(e) => { let _ = tx.send(Action::Error { message: e.to_string() }); }
                    }
                }
            });
        }
    }

    fn handle_action(&mut self, action: &str) {
        if action == "stop" || action == "rm" {
            if let Some((tab, engine, id)) = self.get_selected_resource() {
                self.pending_action = Some((tab, engine, id, action.to_string()));
                self.show_confirmation = true;
            }
            return;
        }
        
        if let Some((tab, engine, id)) = self.get_selected_resource() {
            self.spawn_resource_action(tab, engine, id, action.to_string());
        }
    }

    fn execute_resource_action(&self, resource_type: Tab, engine: String, id: String, action: String) {
        self.spawn_resource_action(resource_type, engine, id, action);
    }

    pub fn get_related_resources(&self, resource_type: &Tab, id: &str) -> Vec<(Tab, String, String)> {
        let mut related = Vec::new();
        if resource_type == &Tab::Images {
            let image_names = self.images.iter()
                .find(|i| i.id == id)
                .map(|i| i.get_names())
                .unwrap_or_default();

            for c in &self.running {
                if c.image == id || c.id == id || image_names.contains(&c.image) {
                     related.push((Tab::Running, c.engine.clone(), c.id.clone()));
                }
            }
            for c in &self.stopped {
                if c.image == id || c.id == id || image_names.contains(&c.image) {
                     related.push((Tab::Stopped, c.engine.clone(), c.id.clone()));
                }
            }
        }
        related
    }

    fn handle_primary_action(&mut self) {
        match self.active_tab {
            Tab::Running | Tab::Stopped | Tab::Pods => {
                self.logs_focused = true;
                if !self.container_logs.is_empty() && self.logs_state.selected().is_none() {
                    self.logs_state.select(Some(self.container_logs.len().saturating_sub(1)));
                }
            }
            Tab::Images
                if self.images.get(self.selected_index).is_some() => {
                    self.create_container_form = Some(CreateContainerForm::default());
                }
            _ => {}
        }
    }

    fn submit_create_container(&mut self) {
        let form = self.create_container_form.clone().unwrap();
        if let Some(img) = self.images.get(self.selected_index) {
            let target_engine = img.engine.clone();
            let client = self.engine_client.clone();
            let tx = self.action_tx.clone();
            let img_id = img.id.clone();
            tokio::spawn(async move {
                let result = client.run_container(
                    &target_engine,
                    &img_id,
                    &form.name,
                    &form.ports,
                    &form.env,
                    &form.command,
                ).await;
                if let Some(tx) = tx {
                    match result {
                        Ok(()) => { let _ = tx.send(Action::ActionComplete); }
                        Err(e) => { let _ = tx.send(Action::Error { message: e.to_string() }); }
                    }
                }
            });
        }
    }

    fn submit_create_pod(&mut self) {
        let form = self.create_pod_form.clone().unwrap();
        let engine = self.get_default_target_engine();
        let client = self.engine_client.clone();
        let tx = self.action_tx.clone();

        // Build share namespace string
        let mut shares = Vec::new();
        if form.share_pid {
            shares.push("pid");
        }
        if form.share_net {
            shares.push("net");
        }
        let share_str = shares.join(",");

        tokio::spawn(async move {
            let result = client.create_pod(&engine, &form.name, &form.network, &share_str).await;
            if let Some(tx) = tx {
                match result {
                    Ok(()) => { let _ = tx.send(Action::ActionComplete); }
                    Err(e) => { let _ = tx.send(Action::Error { message: e.to_string() }); }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use async_trait::async_trait;
    use crate::podman::SearchResult;

    struct MockEngine;
    
    #[async_trait]
    impl EngineClient for MockEngine {
        async fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>> {
            Ok(vec![Container {
                id: "1".into(),
                image: "img".into(),
                command: None,
                created: None,
                state: Some(serde_json::Value::String("running".into())),
                status: Some(serde_json::Value::String("Up".into())),
                names: Some(serde_json::Value::Array(vec!["test".into()])),
                name: None,
                ports: None,
                pod_id: None,
                engine: engines.get(0).cloned().unwrap_or_else(|| "mock".into()),
            }])
        }
        async fn get_images(&self, _engines: &[String]) -> Result<Vec<Image>> {
            Ok(vec![Image {
                id: "img1".into(),
                parent_id: None,
                repo_tags: Some(serde_json::Value::Array(vec!["alpine:latest".into()])),
                repository: None,
                tag: None,
                names: None,
                size: Some(5000),
                created: Some(serde_json::Value::Number(1678901234.into())),
                engine: "mock".into(),
            }])
        }
        async fn get_volumes(&self, _engines: &[String]) -> Result<Vec<Volume>> {
            Ok(vec![Volume {
                name: "vol1".into(),
                driver: "local".into(),
                mountpoint: "/v".into(),
                engine: "mock".into(),
            }])
        }
        async fn get_networks(&self, _engines: &[String]) -> Result<Vec<Network>> {
            Ok(vec![Network {
                name: "net1".into(),
                id: "n1".into(),
                driver: "bridge".into(),
                engine: "mock".into(),
            }])
        }
        async fn get_pods(&self, _engines: &[String]) -> Result<Vec<Pod>> {
            Ok(vec![])
        }
        async fn get_container_logs(&self, _engine: &str, _id: &str) -> Result<Vec<String>> {
            Ok(vec!["mock logs".into()])
        }
        async fn get_pod_logs(&self, _engine: &str, _pod_id: &str) -> Result<Vec<String>> {
            Ok(vec!["mock pod logs".into()])
        }
        async fn action_container(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        async fn run_container(
            &self,
            _engine: &str,
            _image: &str,
            _name: &str,
            _ports: &str,
            _env: &str,
            _command: &str,
        ) -> Result<()> {
            Ok(())
        }
        async fn create_pod(
            &self,
            _engine: &str,
            _name: &str,
            _network: &str,
            _share: &str,
        ) -> Result<()> {
            Ok(())
        }
        async fn search_images(&self, _engines: &[String], _term: &str) -> Result<Vec<SearchResult>> {
            Ok(vec![SearchResult {
                index: "1".into(),
                name: "search_res".into(),
                description: "desc".into(),
                stars: 10,
                official: "OK".into(),
            }])
        }
        async fn pull_image(&self, _engine: &str, _image: &str) -> Result<()> {
            Ok(())
        }
        async fn action_image(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        async fn action_volume(&self, _engine: &str, _name: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        async fn action_network(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        async fn action_pod(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        async fn get_container_inspect(&self, _engine: &str, _id: &str) -> Result<String> {
            Ok("{}".into())
        }
        async fn configure_registries(&self, _registries_csv: &str) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.active_tab, Tab::Running);
        assert!(!app.should_quit);
        assert_eq!(app.engine_view, EngineView::Both);
    }

    #[tokio::test]
    async fn test_app_update_navigation() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::with_client(Box::new(MockEngine));
        
        // Simulating data refresh response
        app.update(Action::DataRefreshed {
            running: vec![],
            stopped: vec![],
            images: vec![],
            volumes: vec![],
            networks: vec![],
            pods: vec![],
        });

        assert_eq!(app.selected_index, 0);
        assert_eq!(app.active_tab, Tab::Running);

        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Stopped);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Images);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Volumes);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Networks);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Pods);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert!(app.logs_focused);

        app.update(Action::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())));
        assert!(!app.logs_focused);

        assert_eq!(app.engine_view, EngineView::Both);
        app.update(Action::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::empty())));
        assert_eq!(app.engine_view, EngineView::Docker);
    }

    #[test]
    fn test_app_get_active_engines() {
        let mut app = App::new();
        app.available_engines = vec!["docker".to_string(), "podman".to_string()];
        app.engine_view = EngineView::Both;
        assert_eq!(app.get_active_engines().len(), 2);

        app.engine_view = EngineView::Docker;
        assert_eq!(app.get_active_engines(), vec!["docker".to_string()]);

        app.engine_view = EngineView::Podman;
        assert_eq!(app.get_active_engines(), vec!["podman".to_string()]);
    }

    #[test]
    fn test_error_handling() {
        let mut app = App::with_client(Box::new(MockEngine));
        app.update(Action::Error { message: "Test error".to_string() });
        assert_eq!(app.status_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_inspect_popup() {
        let mut app = App::with_client(Box::new(MockEngine));
        app.update(Action::InspectResult { output: "test output".to_string() });
        assert_eq!(app.inspect_popup, Some("test output".to_string()));
    }
}
