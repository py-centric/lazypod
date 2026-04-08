use crate::action::Action;
use crate::events::EventHandler;
use crate::podman::{Container, EngineClient, Image, LocalEngines, Network, SearchResult, Volume};
use crate::ui;
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{backend::Backend, Terminal};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum EngineView {
    #[default]
    Both,
    Docker,
    Podman,
}

impl EngineView {
    pub fn next(&mut self) {
        *self = match self {
            EngineView::Both => EngineView::Docker,
            EngineView::Docker => EngineView::Podman,
            EngineView::Podman => EngineView::Both,
        };
    }
}

#[derive(Default, Clone)]
pub struct CreateContainerForm {
    pub name: String,
    pub command: String,
    pub ports: String,
    pub env: String,
    pub active_field: usize, // 0: Name, 1: Command, 2: Ports, 3: Env
}

#[derive(Default, Clone)]
pub struct SearchImageForm {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub is_searching: bool,
}

#[derive(Default, Clone)]
pub struct DirectPullForm {
    pub image: String,
}

#[derive(Default, Clone)]
pub struct ConfigureRegistriesForm {
    pub registries: String,
}

#[derive(Default, Clone)]
pub struct ExecForm {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Running,
    Stopped,
    Images,
    Volumes,
    Networks,
}

pub struct App {
    pub should_quit: bool,
    pub active_tab: Tab,
    pub running: Vec<Container>,
    pub stopped: Vec<Container>,
    pub images: Vec<Image>,
    pub volumes: Vec<Volume>,
    pub networks: Vec<Network>,
    pub selected_index: usize,
    pub running_index: usize,
    pub stopped_index: usize,
    pub images_index: usize,
    pub volumes_index: usize,
    pub networks_index: usize,
    pub show_confirmation: bool,
    pub create_container_form: Option<CreateContainerForm>,
    pub search_image_form: Option<SearchImageForm>,
    pub is_pulling: bool,
    pub container_logs: String,
    pub logs_focused: bool,
    pub logs_scroll: u16,
    pub pending_exec: Option<(String, String)>, // (engine, id)
    pub engine_view: EngineView,
    pub engine_client: Box<dyn EngineClient>,
    pub show_help_tooltip: bool,
    pub direct_pull_form: Option<DirectPullForm>,
    pub configure_registries_form: Option<ConfigureRegistriesForm>,
    pub exec_form: Option<ExecForm>,
    pub pending_action: Option<(Tab, String, String, String)>, // (resource_type, engine, id, action)
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            active_tab: Tab::Running,
            running: Vec::new(),
            stopped: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
            selected_index: 0,
            running_index: 0,
            stopped_index: 0,
            images_index: 0,
            volumes_index: 0,
            networks_index: 0,
            show_confirmation: false,
            create_container_form: None,
            search_image_form: None,
            is_pulling: false,
            container_logs: String::new(),
            logs_focused: false,
            logs_scroll: 0,
            pending_exec: None,
            engine_view: EngineView::Both,
            engine_client: Box::new(LocalEngines),
            show_help_tooltip: false,
            direct_pull_form: None,
            configure_registries_form: None,
            exec_form: None,
            pending_action: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_client(client: Box<dyn EngineClient>) -> Self {
        Self {
            should_quit: false,
            active_tab: Tab::Running,
            running: Vec::new(),
            stopped: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
            selected_index: 0,
            running_index: 0,
            stopped_index: 0,
            images_index: 0,
            volumes_index: 0,
            networks_index: 0,
            show_confirmation: false,
            create_container_form: None,
            search_image_form: None,
            is_pulling: false,
            container_logs: String::new(),
            logs_focused: false,
            logs_scroll: 0,
            pending_exec: None,
            engine_view: EngineView::Both,
            engine_client: client,
            show_help_tooltip: false,
            direct_pull_form: None,
            configure_registries_form: None,
            exec_form: None,
            pending_action: None,
        }
    }

    pub fn get_active_engines(&self) -> Vec<String> {
        match self.engine_view {
            EngineView::Both => vec!["docker".to_string(), "podman".to_string()],
            EngineView::Docker => vec!["docker".to_string()],
            EngineView::Podman => vec!["podman".to_string()],
        }
    }

    pub fn get_default_target_engine(&self) -> String {
        self.get_active_engines()
            .into_iter()
            .next()
            .unwrap_or_else(|| "docker".into())
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut events = EventHandler::new(250);

        self.refresh_data();

        while !self.should_quit {
            terminal.draw(|f| ui::draw(f, self))?;

            if let Some(action) = events.next().await {
                self.update(action);
            }

            if let Some((engine, cmd)) = self.pending_exec.take() {
                // Stop the event handler thread before handing over the terminal
                drop(events);

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

                // Restart the event handler after returning from the shell
                events = EventHandler::new(250);
                self.refresh_data();
            }
        }

        Ok(())
    }

    pub fn update(&mut self, action: Action) {
        let key = match action {
            Action::Quit => {
                self.should_quit = true;
                return;
            }
            Action::Tick => return,
            Action::Key(k) => k,
            Action::Mouse(m) => {
                self.handle_mouse(m);
                return;
            }
        };

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
            let _ = self.engine_client.pull_image(&target, &img);
            self.is_pulling = false;
            self.direct_pull_form = None;
            self.refresh_data();
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
            let _ = self.engine_client.configure_registries(&regs);
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
                return; // ignore input while pulling
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
            if let Ok(results) = self.engine_client.search_images(&engines, &query) {
                if let Some(form) = &mut self.search_image_form {
                    form.results = results;
                    form.selected = 0;
                }
            }
            if let Some(form) = &mut self.search_image_form {
                form.is_searching = false;
            }
            return;
        }

        if let Some(name) = action_pull {
            self.is_pulling = true;
            let target_engine = self.get_default_target_engine();
            let _ = self.engine_client.pull_image(&target_engine, &name);
            self.is_pulling = false;
            self.search_image_form = None;
            self.refresh_data();
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
                        self.refresh_data();
                    }
                }
                KeyCode::Char('a') => {
                    self.show_confirmation = false;
                    if let Some((resource_type, engine, id, action)) = self.pending_action.take() {
                        // Delete related resources first
                        let related = self.get_related_resources(&resource_type, &id);
                        for (r_type, r_engine, r_id) in related {
                            self.execute_resource_action(r_type, r_engine, r_id, action.clone());
                        }
                        // Then delete the primary resource
                        self.execute_resource_action(resource_type, engine, id, action);
                        self.refresh_data();
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
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.logs_scroll = self.logs_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let lines_count = self.container_logs.lines().count() as u16;
                    let max_scroll = (lines_count + 15).saturating_sub(1);
                    if self.logs_scroll < max_scroll {
                        self.logs_scroll += 1;
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
                    self.active_tab = Tab::Running;
                    self.selected_index = self.running_index;
                } else {
                    match self.active_tab {
                        Tab::Running => {
                            self.running_index = self.selected_index;
                            self.active_tab = Tab::Stopped;
                            self.selected_index = self.stopped_index;
                        }
                        Tab::Stopped => {
                            self.stopped_index = self.selected_index;
                            self.active_tab = Tab::Images;
                            self.selected_index = self.images_index;
                        }
                        Tab::Images => {
                            self.images_index = self.selected_index;
                            self.active_tab = Tab::Volumes;
                            self.selected_index = self.volumes_index;
                        }
                        Tab::Volumes => {
                            self.volumes_index = self.selected_index;
                            self.active_tab = Tab::Networks;
                            self.selected_index = self.networks_index;
                        }
                        Tab::Networks => {
                            self.networks_index = self.selected_index;
                            self.logs_focused = true;
                        }
                    }
                }
                self.fetch_logs();
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                if matches!(self.active_tab, Tab::Running | Tab::Stopped) {
                    self.logs_focused = true;
                    return;
                }
                match self.active_tab {
                    Tab::Running => {
                        self.running_index = self.selected_index;
                        self.active_tab = Tab::Stopped;
                        self.selected_index = self.stopped_index;
                    }
                    Tab::Stopped => {
                        self.stopped_index = self.selected_index;
                        self.active_tab = Tab::Images;
                        self.selected_index = self.images_index;
                    }
                    Tab::Images => {
                        self.images_index = self.selected_index;
                        self.active_tab = Tab::Volumes;
                        self.selected_index = self.volumes_index;
                    }
                    Tab::Volumes => {
                        self.volumes_index = self.selected_index;
                        self.active_tab = Tab::Networks;
                        self.selected_index = self.networks_index;
                    }
                    Tab::Networks => {
                        self.networks_index = self.selected_index;
                        self.active_tab = Tab::Running;
                        self.selected_index = self.running_index;
                    }
                };
                self.fetch_logs();
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                if self.logs_focused {
                    self.logs_focused = false;
                    return;
                }
                match self.active_tab {
                    Tab::Running => {
                        self.running_index = self.selected_index;
                        self.active_tab = Tab::Networks;
                        self.selected_index = self.networks_index;
                    }
                    Tab::Stopped => {
                        self.stopped_index = self.selected_index;
                        self.active_tab = Tab::Running;
                        self.selected_index = self.running_index;
                    }
                    Tab::Images => {
                        self.images_index = self.selected_index;
                        self.active_tab = Tab::Stopped;
                        self.selected_index = self.stopped_index;
                    }
                    Tab::Volumes => {
                        self.volumes_index = self.selected_index;
                        self.active_tab = Tab::Images;
                        self.selected_index = self.images_index;
                    }
                    Tab::Networks => {
                        self.networks_index = self.selected_index;
                        self.active_tab = Tab::Volumes;
                        self.selected_index = self.volumes_index;
                    }
                };
                self.fetch_logs();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = match self.active_tab {
                    Tab::Running => self.running.len().saturating_sub(1),
                    Tab::Stopped => self.stopped.len().saturating_sub(1),
                    Tab::Images => self.images.len().saturating_sub(1),
                    Tab::Volumes => self.volumes.len().saturating_sub(1),
                    Tab::Networks => self.networks.len().saturating_sub(1),
                };
                if self.selected_index < max {
                    self.selected_index += 1;
                    self.fetch_logs();
                } else if !self.logs_focused {
                    // Boundary: move to next pane
                    match self.active_tab {
                        Tab::Running => {
                            self.running_index = self.selected_index;
                            self.active_tab = Tab::Stopped;
                            self.selected_index = self.stopped_index;
                        }
                        Tab::Stopped => {
                            self.stopped_index = self.selected_index;
                            self.active_tab = Tab::Images;
                            self.selected_index = self.images_index;
                        }
                        Tab::Images => {
                            self.images_index = self.selected_index;
                            self.active_tab = Tab::Volumes;
                            self.selected_index = self.volumes_index;
                        }
                        Tab::Volumes => {
                            self.volumes_index = self.selected_index;
                            self.active_tab = Tab::Networks;
                            self.selected_index = self.networks_index;
                        }
                        Tab::Networks => {
                            self.networks_index = self.selected_index;
                            self.logs_focused = true;
                            return;
                        }
                    }
                    self.fetch_logs();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.fetch_logs();
                } else if !self.logs_focused {
                    // Boundary: move to previous pane
                    match self.active_tab {
                        Tab::Running => {
                            self.running_index = self.selected_index;
                            self.logs_focused = true;
                            return;
                        }
                        Tab::Stopped => {
                            self.stopped_index = self.selected_index;
                            self.active_tab = Tab::Running;
                            self.selected_index = self.running_index;
                        }
                        Tab::Images => {
                            self.images_index = self.selected_index;
                            self.active_tab = Tab::Stopped;
                            self.selected_index = self.stopped_index;
                        }
                        Tab::Volumes => {
                            self.volumes_index = self.selected_index;
                            self.active_tab = Tab::Images;
                            self.selected_index = self.images_index;
                        }
                        Tab::Networks => {
                            self.networks_index = self.selected_index;
                            self.active_tab = Tab::Volumes;
                            self.selected_index = self.volumes_index;
                        }
                    }
                    self.fetch_logs();
                }
            }
            KeyCode::Char('r') => self.refresh_data(),
            KeyCode::Char('E') => {
                self.engine_view.next();
                self.selected_index = 0;
                self.refresh_data();
            }
            KeyCode::Char('s') => {
                match self.active_tab {
                    Tab::Running => self.handle_action("stop"),
                    Tab::Stopped => self.handle_action("start"),
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
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                if matches!(self.active_tab, Tab::Running) {
                    if self.running.get(self.selected_index).is_some() {
                        self.exec_form = Some(ExecForm {
                            command: String::new(),
                        });
                        self.logs_focused = false;
                    }
                }
            }
            KeyCode::Char('S') | KeyCode::Char('u') => self.handle_action("start"),
            KeyCode::Char('d') | KeyCode::Delete => self.handle_action("rm"),
            KeyCode::Enter => self.handle_primary_action(),
            KeyCode::Char('?') => self.show_help_tooltip = true,
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        if self.show_confirmation || self.create_container_form.is_some() {
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollDown => {
                if self.logs_focused {
                    let lines_count = self.container_logs.lines().count() as u16;
                    let max_scroll = (lines_count + 15).saturating_sub(1);
                    if self.logs_scroll < max_scroll {
                        self.logs_scroll += 1;
                    }
                    return;
                }
                let max = match self.active_tab {
                    Tab::Running => self.running.len().saturating_sub(1),
                    Tab::Stopped => self.stopped.len().saturating_sub(1),
                    Tab::Images => self.images.len().saturating_sub(1),
                    Tab::Volumes => self.volumes.len().saturating_sub(1),
                    Tab::Networks => self.networks.len().saturating_sub(1),
                };
                if self.selected_index < max {
                    self.selected_index += 1;
                    self.fetch_logs();
                }
            }
            MouseEventKind::ScrollUp => {
                if self.logs_focused {
                    self.logs_scroll = self.logs_scroll.saturating_sub(1);
                    return;
                }
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.fetch_logs();
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Ok((cols, rows)) = crossterm::terminal::size() {
                    let left_panel_width = (cols as f32 * 0.3) as u16;
                    if mouse.column >= left_panel_width {
                        if matches!(self.active_tab, Tab::Running | Tab::Stopped) {
                            self.logs_focused = true;
                        }
                    } else {
                        self.logs_focused = false;
                        let available_rows = rows.saturating_sub(1);
                        let h = available_rows / 5;

                        if mouse.row < h {
                            self.active_tab = Tab::Running;
                            let idx = mouse.row.saturating_sub(1); // 1 for top border
                            let max = self.running.len().saturating_sub(1);
                            self.selected_index = std::cmp::min(idx as usize, max);
                            self.logs_focused = true;
                        } else if mouse.row < h * 2 {
                            self.active_tab = Tab::Stopped;
                            let idx = (mouse.row - h).saturating_sub(1);
                            let max = self.stopped.len().saturating_sub(1);
                            self.selected_index = std::cmp::min(idx as usize, max);
                        } else if mouse.row < h * 3 {
                            self.active_tab = Tab::Images;
                            let idx = (mouse.row - h * 2).saturating_sub(1);
                            let max = self.images.len().saturating_sub(1);
                            self.selected_index = std::cmp::min(idx as usize, max);
                        } else if mouse.row < h * 4 {
                            self.active_tab = Tab::Volumes;
                            let idx = (mouse.row - h * 3).saturating_sub(1);
                            let max = self.volumes.len().saturating_sub(1);
                            self.selected_index = std::cmp::min(idx as usize, max);
                        } else if mouse.row < available_rows {
                            self.active_tab = Tab::Networks;
                            let idx = (mouse.row - h * 4).saturating_sub(1);
                            let max = self.networks.len().saturating_sub(1);
                            self.selected_index = std::cmp::min(idx as usize, max);
                        }
                        self.fetch_logs();
                    }
                }
            }
            _ => {}
        }
    }

    fn refresh_data(&mut self) {
        let engines = self.get_active_engines();
        if let Ok(c) = self.engine_client.get_containers(&engines) {
            self.running = c.iter().filter(|x| x.is_running()).cloned().collect();
            self.stopped = c.iter().filter(|x| !x.is_running()).cloned().collect();
        } else {
            self.running.clear();
            self.stopped.clear();
        }
        if let Ok(i) = self.engine_client.get_images(&engines) {
            self.images = i;
        } else {
            self.images.clear();
        }
        if let Ok(v) = self.engine_client.get_volumes(&engines) {
            self.volumes = v;
        } else {
            self.volumes.clear();
        }
        if let Ok(n) = self.engine_client.get_networks(&engines) {
            self.networks = n;
        } else {
            self.networks.clear();
        }

        self.running_index = std::cmp::min(self.running_index, self.running.len().saturating_sub(1));
        self.stopped_index = std::cmp::min(self.stopped_index, self.stopped.len().saturating_sub(1));
        self.images_index = std::cmp::min(self.images_index, self.images.len().saturating_sub(1));
        self.volumes_index = std::cmp::min(self.volumes_index, self.volumes.len().saturating_sub(1));
        self.networks_index = std::cmp::min(self.networks_index, self.networks.len().saturating_sub(1));

        self.selected_index = match self.active_tab {
            Tab::Running => self.running_index,
            Tab::Stopped => self.stopped_index,
            Tab::Images => self.images_index,
            Tab::Volumes => self.volumes_index,
            Tab::Networks => self.networks_index,
        };

        self.fetch_logs();
    }

    fn fetch_logs(&mut self) {
        self.logs_scroll = 0;
        self.container_logs.clear();
        match self.active_tab {
            Tab::Running => {
                if let Some(c) = self.running.get(self.selected_index) {
                    if let Ok(logs) = self.engine_client.get_container_logs(&c.engine, &c.id) {
                        self.container_logs = logs;
                    }
                }
            }
            Tab::Stopped => {
                if let Some(c) = self.stopped.get(self.selected_index) {
                    if let Ok(logs) = self.engine_client.get_container_logs(&c.engine, &c.id) {
                        self.container_logs = logs;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_action(&mut self, action: &str) {
        if action == "stop" || action == "rm" {
            let res = match self.active_tab {
                Tab::Running => self
                    .running
                    .get(self.selected_index)
                    .map(|c| (c.engine.clone(), c.id.clone())),
                Tab::Stopped => self
                    .stopped
                    .get(self.selected_index)
                    .map(|c| (c.engine.clone(), c.id.clone())),
                Tab::Images => self
                    .images
                    .get(self.selected_index)
                    .map(|i| (i.engine.clone(), i.id.clone())),
                Tab::Volumes => self
                    .volumes
                    .get(self.selected_index)
                    .map(|v| (v.engine.clone(), v.name.clone())),
                Tab::Networks => self
                    .networks
                    .get(self.selected_index)
                    .map(|n| (n.engine.clone(), n.id.clone())),
            };

            if let Some((engine, id)) = res {
                self.pending_action = Some((self.active_tab.clone(), engine, id, action.to_string()));
                self.show_confirmation = true;
            }
            return;
        }
        match self.active_tab {
            Tab::Running => {
                if let Some(c) = self.running.get(self.selected_index) {
                    let _ = self
                        .engine_client
                        .action_container(&c.engine, &c.id, action);
                    self.refresh_data();
                }
            }
            Tab::Stopped => {
                if let Some(c) = self.stopped.get(self.selected_index) {
                    let _ = self
                        .engine_client
                        .action_container(&c.engine, &c.id, action);
                    self.refresh_data();
                }
            }
            Tab::Images => {
                if let Some(i) = self.images.get(self.selected_index) {
                    let _ = self.engine_client.action_image(&i.engine, &i.id, action);
                    self.refresh_data();
                }
            }
            Tab::Volumes => {
                if let Some(v) = self.volumes.get(self.selected_index) {
                    let _ = self.engine_client.action_volume(&v.engine, &v.name, action);
                    self.refresh_data();
                }
            }
            Tab::Networks => {
                if let Some(n) = self.networks.get(self.selected_index) {
                    let _ = self.engine_client.action_network(&n.engine, &n.id, action);
                    self.refresh_data();
                }
            }
        }
    }

    fn execute_resource_action(&self, resource_type: Tab, engine: String, id: String, action: String) {
        match resource_type {
            Tab::Running | Tab::Stopped => {
                let _ = self.engine_client.action_container(&engine, &id, &action);
            }
            Tab::Images => {
                let _ = self.engine_client.action_image(&engine, &id, &action);
            }
            Tab::Volumes => {
                let _ = self.engine_client.action_volume(&engine, &id, &action);
            }
            Tab::Networks => {
                let _ = self.engine_client.action_network(&engine, &id, &action);
            }
        }
    }

    pub fn get_related_resources(&self, resource_type: &Tab, id: &str) -> Vec<(Tab, String, String)> {
        let mut related = Vec::new();
        match resource_type {
            Tab::Images => {
                // Try to find the image to get its names
                let image_names = self.images.iter()
                    .find(|i| i.id == id)
                    .map(|i| i.get_names())
                    .unwrap_or_default();

                // Find all containers using this image ID or any of its names
                // Check running containers
                for c in &self.running {
                    if c.image == id || c.id == id || image_names.contains(&c.image) {
                         related.push((Tab::Running, c.engine.clone(), c.id.clone()));
                    }
                }
                // Check stopped containers
                for c in &self.stopped {
                    if c.image == id || c.id == id || image_names.contains(&c.image) {
                         related.push((Tab::Stopped, c.engine.clone(), c.id.clone()));
                    }
                }
            }
            _ => {} // Future work: handle Volumes -> Containers etc.
        }
        related
    }

    fn handle_primary_action(&mut self) {
        match self.active_tab {
            Tab::Running | Tab::Stopped => {
                self.logs_focused = true;
            }
            Tab::Images => {
                if self.images.get(self.selected_index).is_some() {
                    self.create_container_form = Some(CreateContainerForm::default());
                }
            }
            _ => {}
        }
    }

    fn submit_create_container(&mut self) {
        let form = self.create_container_form.clone().unwrap();
        if let Some(img) = self.images.get(self.selected_index) {
            let target_engine = self.get_default_target_engine();
            let _ = self.engine_client.run_container(
                &target_engine,
                &img.id,
                &form.name,
                &form.ports,
                &form.env,
                &form.command,
            );
            self.refresh_data();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEngine;
    impl EngineClient for MockEngine {
        fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>> {
            Ok(vec![Container {
                id: "1".into(),
                image: "img".into(),
                command: None,
                created: None,
                state: Some(serde_json::Value::String("running".into())),
                status: Some(serde_json::Value::String("Up".into())),
                names: Some(serde_json::Value::Array(vec!["test".into()])),
                name: None,
                engine: engines.get(0).cloned().unwrap_or_else(|| "mock".into()),
            }])
        }
        fn get_images(&self, _engines: &[String]) -> Result<Vec<Image>> {
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
        fn get_volumes(&self, _engines: &[String]) -> Result<Vec<Volume>> {
            Ok(vec![Volume {
                name: "vol1".into(),
                driver: "local".into(),
                mountpoint: "/v".into(),
                engine: "mock".into(),
            }])
        }
        fn get_networks(&self, _engines: &[String]) -> Result<Vec<Network>> {
            Ok(vec![Network {
                name: "net1".into(),
                id: "n1".into(),
                driver: "bridge".into(),
                engine: "mock".into(),
            }])
        }
        fn get_container_logs(&self, _engine: &str, _id: &str) -> Result<String> {
            Ok("mock logs".into())
        }
        fn action_container(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        fn run_container(
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
        fn search_images(&self, _engines: &[String], _term: &str) -> Result<Vec<SearchResult>> {
            Ok(vec![SearchResult {
                index: "1".into(),
                name: "search_res".into(),
                description: "desc".into(),
                stars: 10,
                official: "OK".into(),
            }])
        }
        fn pull_image(&self, _engine: &str, _image: &str) -> Result<()> {
            Ok(())
        }
        fn action_image(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        fn action_volume(&self, _engine: &str, _name: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        fn action_network(&self, _engine: &str, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        fn configure_registries(&self, _registries_csv: &str) -> Result<()> {
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

    #[test]
    fn test_app_update_navigation() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::with_client(Box::new(MockEngine));
        app.refresh_data();

        assert_eq!(app.selected_index, 0);
        assert_eq!(app.active_tab, Tab::Running);

        // Tab through all panes: Running -> Stopped -> Images -> Volumes -> Networks -> Logs
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Stopped);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Images);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Volumes);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert_eq!(app.active_tab, Tab::Networks);
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert!(app.logs_focused);

        // Back out from Logs
        app.update(Action::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())));
        assert!(!app.logs_focused);

        // Cycle engine view
        assert_eq!(app.engine_view, EngineView::Both);
        app.update(Action::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::empty())));
        assert_eq!(app.engine_view, EngineView::Docker);
    }

    #[test]
    fn test_app_refresh_data() {
        let mut app = App::with_client(Box::new(MockEngine));
        app.refresh_data();
        assert_eq!(app.running.len(), 1);
        assert_eq!(app.images.len(), 1);
        assert_eq!(app.volumes.len(), 1);
        assert_eq!(app.networks.len(), 1);
    }

    #[test]
    fn test_app_get_active_engines() {
        let mut app = App::new();
        app.engine_view = EngineView::Both;
        assert_eq!(app.get_active_engines().len(), 2);

        app.engine_view = EngineView::Docker;
        assert_eq!(app.get_active_engines(), vec!["docker".to_string()]);

        app.engine_view = EngineView::Podman;
        assert_eq!(app.get_active_engines(), vec!["podman".to_string()]);
    }
}
