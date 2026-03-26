use crate::action::Action;
use crate::events::EventHandler;
use crate::podman::{Container, Image, LocalPodman, Network, PodmanClient, SearchResult, Volume};
use crate::ui;
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{backend::Backend, Terminal};

#[derive(Default, Clone)]
pub struct CreateContainerForm {
    pub name: String,
    pub command: String,
    pub ports: String,
    pub active_field: usize, // 0: Name, 1: Command, 2: Ports
}

#[derive(Default, Clone)]
pub struct SearchImageForm {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub is_searching: bool,
}

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
    pub show_confirmation: bool,
    pub create_container_form: Option<CreateContainerForm>,
    pub search_image_form: Option<SearchImageForm>,
    pub is_pulling: bool,
    pub container_logs: String,
    pub logs_focused: bool,
    pub logs_scroll: u16,
    pub pending_exec: Option<String>,
    podman: Box<dyn PodmanClient>,
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
            show_confirmation: false,
            create_container_form: None,
            search_image_form: None,
            is_pulling: false,
            container_logs: String::new(),
            logs_focused: false,
            logs_scroll: 0,
            pending_exec: None,
            podman: Box::new(LocalPodman),
        }
    }

    #[allow(dead_code)]
    pub fn with_client(client: Box<dyn PodmanClient>) -> Self {
        Self {
            should_quit: false,
            active_tab: Tab::Running,
            running: Vec::new(),
            stopped: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
            selected_index: 0,
            show_confirmation: false,
            create_container_form: None,
            search_image_form: None,
            is_pulling: false,
            container_logs: String::new(),
            logs_focused: false,
            logs_scroll: 0,
            pending_exec: None,
            podman: client,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut events = EventHandler::new(250);
        
        self.refresh_data();

        while !self.should_quit {
            terminal.draw(|f| ui::draw(f, self))?;

            if let Some(action) = events.next().await {
                self.update(action);
            }

            if let Some(cmd) = self.pending_exec.take() {
                crossterm::terminal::disable_raw_mode()?;
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::LeaveAlternateScreen,
                    crossterm::event::DisableMouseCapture
                )?;
                
                let mut child = std::process::Command::new("podman")
                    .arg("exec")
                    .arg("-it")
                    .arg(&cmd)
                    .arg("/bin/sh")
                    .spawn()?;
                let _ = child.wait()?;
                
                crossterm::terminal::enable_raw_mode()?;
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::EnterAlternateScreen,
                    crossterm::event::EnableMouseCapture
                )?;
                terminal.clear()?;
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
                        form.is_searching = true;
                        if let Ok(results) = self.podman.search_images(&form.query) {
                            form.results = results;
                            form.selected = 0;
                        }
                        form.is_searching = false;
                    } else if let Some(res) = form.results.get(form.selected) {
                        self.is_pulling = true;
                        let _ = self.podman.pull_image(&res.name);
                        self.is_pulling = false;
                        self.search_image_form = None;
                        self.refresh_data();
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    if !form.results.is_empty() && form.selected < form.results.len().saturating_sub(1) {
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
                    form.active_field = (form.active_field + 1) % 3;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    form.active_field = form.active_field.checked_sub(1).unwrap_or(2);
                }
                KeyCode::Backspace => {
                    let field = match form.active_field {
                        0 => &mut form.name,
                        1 => &mut form.command,
                        _ => &mut form.ports,
                    };
                    field.pop();
                }
                KeyCode::Char(c) => {
                    let field = match form.active_field {
                        0 => &mut form.name,
                        1 => &mut form.command,
                        _ => &mut form.ports,
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
                    self.handle_action("stop");
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.show_confirmation = false;
                }
                _ => {}
            }
            return;
        }

        if self.logs_focused {
            match key.code {
                KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('q') => {
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
                            self.pending_exec = Some(c.id.clone());
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
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                if matches!(self.active_tab, Tab::Running | Tab::Stopped) {
                    self.logs_focused = true;
                    return;
                }
                self.active_tab = match self.active_tab {
                    Tab::Running => Tab::Stopped,
                    Tab::Stopped => Tab::Images,
                    Tab::Images => Tab::Volumes,
                    Tab::Volumes => Tab::Networks,
                    Tab::Networks => Tab::Running,
                };
                self.selected_index = 0;
                self.fetch_logs();
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                self.active_tab = match self.active_tab {
                    Tab::Running => Tab::Networks,
                    Tab::Stopped => Tab::Running,
                    Tab::Images => Tab::Stopped,
                    Tab::Volumes => Tab::Images,
                    Tab::Networks => Tab::Volumes,
                };
                self.selected_index = 0;
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
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.fetch_logs();
                }
            }
            KeyCode::Char('r') => self.refresh_data(),
            KeyCode::Char('s') => {
                if matches!(self.active_tab, Tab::Running) && self.running.get(self.selected_index).is_some() {
                    self.show_confirmation = true;
                } else {
                    self.handle_action("stop");
                }
            }
            KeyCode::Char('/') => {
                if matches!(self.active_tab, Tab::Images) {
                    self.search_image_form = Some(SearchImageForm::default());
                }
            }
            KeyCode::Char('x') | KeyCode::Char('e') | KeyCode::Char('i') => {
                if matches!(self.active_tab, Tab::Running) {
                    if let Some(c) = self.running.get(self.selected_index) {
                        self.pending_exec = Some(c.id.clone());
                    }
                }
            }
            KeyCode::Char('S') | KeyCode::Char('u') => self.handle_action("start"),
            KeyCode::Char('d') | KeyCode::Delete => self.handle_action("rm"),
            KeyCode::Enter => self.handle_primary_action(),
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
        if let Ok(c) = self.podman.get_containers() {
            self.running = c.iter().filter(|x| x.is_running()).cloned().collect();
            self.stopped = c.iter().filter(|x| !x.is_running()).cloned().collect();
        }
        if let Ok(i) = self.podman.get_images() {
            self.images = i;
        }
        if let Ok(v) = self.podman.get_volumes() {
            self.volumes = v;
        }
        if let Ok(n) = self.podman.get_networks() {
            self.networks = n;
        }
        let max = match self.active_tab {
            Tab::Running => self.running.len().saturating_sub(1),
            Tab::Stopped => self.stopped.len().saturating_sub(1),
            Tab::Images => self.images.len().saturating_sub(1),
            Tab::Volumes => self.volumes.len().saturating_sub(1),
            Tab::Networks => self.networks.len().saturating_sub(1),
        };
        if self.selected_index > max {
            self.selected_index = max;
        }
        self.fetch_logs();
    }

    fn fetch_logs(&mut self) {
        self.logs_scroll = 0;
        self.container_logs.clear();
        match self.active_tab {
            Tab::Running => {
                if let Some(c) = self.running.get(self.selected_index) {
                    if let Ok(logs) = self.podman.get_container_logs(&c.id) {
                        self.container_logs = logs;
                    }
                }
            }
            Tab::Stopped => {
                if let Some(c) = self.stopped.get(self.selected_index) {
                    if let Ok(logs) = self.podman.get_container_logs(&c.id) {
                        self.container_logs = logs;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_action(&mut self, action: &str) {
        match self.active_tab {
            Tab::Running => {
                if let Some(c) = self.running.get(self.selected_index) {
                    let _ = self.podman.action_container(&c.id, action);
                    self.refresh_data();
                }
            }
            Tab::Stopped => {
                if let Some(c) = self.stopped.get(self.selected_index) {
                    let _ = self.podman.action_container(&c.id, action);
                    self.refresh_data();
                }
            }
            _ => {}
        }
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
            let _ = self.podman.run_container(&img.id, &form.name, &form.ports, &form.command);
            self.refresh_data();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPodman;
    impl PodmanClient for MockPodman {
        fn get_containers(&self) -> Result<Vec<Container>> {
            Ok(vec![Container {
                id: "1".into(),
                image: "img".into(),
                command: None,
                created: None,
                state: Some("running".into()),
                status: Some("Up".into()),
                names: Some(serde_json::Value::Array(vec![serde_json::Value::String("test".into())])),
                name: None,
            }])
        }
        fn get_images(&self) -> Result<Vec<Image>> {
            Ok(vec![])
        }
        fn get_volumes(&self) -> Result<Vec<Volume>> {
            Ok(vec![])
        }
        fn get_networks(&self) -> Result<Vec<Network>> {
            Ok(vec![])
        }
        fn get_container_logs(&self, _id: &str) -> Result<String> {
            Ok("".into())
        }
        fn action_container(&self, _id: &str, _action: &str) -> Result<()> {
            Ok(())
        }
        fn run_container(&self, _image: &str, _name: &str, _ports: &str, _command: &str) -> Result<()> {
            Ok(())
        }
        fn search_images(&self, _term: &str) -> Result<Vec<SearchResult>> {
            Ok(vec![])
        }
        fn pull_image(&self, _image: &str) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_app_update_navigation() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::with_client(Box::new(MockPodman));
        app.refresh_data();
        
        assert_eq!(app.selected_index, 0);
        app.update(Action::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty())));
        assert_eq!(app.selected_index, 0); // max is 0
        
        app.update(Action::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())));
        assert!(matches!(app.active_tab, Tab::Stopped));
    }
}
