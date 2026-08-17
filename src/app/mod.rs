pub mod dispatcher;
pub mod forms;
pub mod input;
pub mod mouse;
pub mod state;

use anyhow::Result;
use ratatui::widgets::ListState;
use ratatui::{backend::Backend, Terminal};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::action::Action;
use crate::events::EventHandler;
use crate::podman::{Container, EngineClient, Image, LocalEngines, Network, Pod, Volume};
use crate::ui;

pub use forms::*;
pub use state::{EngineView, Tab};

/// Central application state holding resource inventories, form states, and UI navigation indices.
#[allow(clippy::struct_excessive_bools)]
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
    pub filter_dangling_images: bool,
    pub tag_image_form: Option<TagImageForm>,
    pub image_history: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Initialize a new App detecting available local container engines.
    #[must_use]
    pub fn new() -> Self {
        let mut available_engines = Vec::new();
        if std::process::Command::new("podman")
            .arg("--version")
            .output()
            .is_ok()
        {
            available_engines.push("podman".to_string());
        }
        if std::process::Command::new("docker")
            .arg("--version")
            .output()
            .is_ok()
        {
            available_engines.push("docker".to_string());
        }
        if available_engines.is_empty() {
            available_engines = vec!["podman".to_string(), "docker".to_string()];
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
            filter_dangling_images: false,
            tag_image_form: None,
            image_history: Vec::new(),
        }
    }

    /// Get filtered list of images based on `filter_dangling_images` toggle.
    #[must_use]
    pub fn get_filtered_images(&self) -> Vec<&Image> {
        if self.filter_dangling_images {
            self.images.iter().filter(|i| i.is_dangling()).collect()
        } else {
            self.images.iter().collect()
        }
    }

    /// Construct an App with a custom or mock `EngineClient` (for testing).
    #[must_use]
    pub fn with_client(client: Box<dyn EngineClient>) -> Self {
        let mut app = Self::new();
        app.engine_client = Arc::new(client);
        app
    }

    /// Get active engines filtered by current `EngineView` and system availability.
    #[must_use]
    pub fn get_active_engines(&self) -> Vec<String> {
        let desired = match self.engine_view {
            EngineView::Both => vec!["docker".to_string(), "podman".to_string()],
            EngineView::Docker => vec!["docker".to_string()],
            EngineView::Podman => vec!["podman".to_string()],
        };
        desired
            .into_iter()
            .filter(|e| self.available_engines.contains(e))
            .collect()
    }

    /// Get the default engine for actions (prefers first active engine, fallback to docker).
    #[must_use]
    pub fn get_default_target_engine(&self) -> String {
        self.get_active_engines()
            .into_iter()
            .next()
            .unwrap_or_else(|| "docker".into())
    }

    /// Switch active tab, preserving selection indices.
    pub fn switch_to_tab(&mut self, new_tab: Tab) {
        self.save_current_index();
        self.active_tab = new_tab;
        self.load_current_index();
        self.logs_focused = false;
        self.logs_state.select(None);
    }

    /// Save the selected index for the current tab.
    pub fn save_current_index(&mut self) {
        match self.active_tab {
            Tab::Running => self.running_index = self.selected_index,
            Tab::Stopped => self.stopped_index = self.selected_index,
            Tab::Images => self.images_index = self.selected_index,
            Tab::Volumes => self.volumes_index = self.selected_index,
            Tab::Networks => self.networks_index = self.selected_index,
            Tab::Pods => self.pods_index = self.selected_index,
        }
    }

    /// Restore the selected index for the current tab.
    pub fn load_current_index(&mut self) {
        self.selected_index = match self.active_tab {
            Tab::Running => self.running_index,
            Tab::Stopped => self.stopped_index,
            Tab::Images => self.images_index,
            Tab::Volumes => self.volumes_index,
            Tab::Networks => self.networks_index,
            Tab::Pods => self.pods_index,
        };
    }

    /// Get count of items in the given tab.
    #[must_use]
    pub fn get_list_len_for_tab(&self, tab: &Tab) -> usize {
        match tab {
            Tab::Running => self.running.len(),
            Tab::Stopped => self.stopped.len(),
            Tab::Images => self.get_filtered_images().len(),
            Tab::Volumes => self.volumes.len(),
            Tab::Networks => self.networks.len(),
            Tab::Pods => self.pods.len(),
        }
    }

    /// Get the currently selected resource identifier tuple (Tab, engine, ID/Name).
    #[must_use]
    pub fn get_selected_resource(&self) -> Option<(Tab, String, String)> {
        match self.active_tab {
            Tab::Running | Tab::Stopped => {
                let list = if self.active_tab == Tab::Running {
                    &self.running
                } else {
                    &self.stopped
                };
                list.get(self.selected_index)
                    .map(|c| (self.active_tab.clone(), c.engine.clone(), c.id.clone()))
            }
            Tab::Images => self
                .get_filtered_images()
                .get(self.selected_index)
                .map(|i| (Tab::Images, i.engine.clone(), i.id.clone())),
            Tab::Volumes => self
                .volumes
                .get(self.selected_index)
                .map(|v| (Tab::Volumes, v.engine.clone(), v.name.clone())),
            Tab::Networks => self
                .networks
                .get(self.selected_index)
                .map(|n| (Tab::Networks, n.engine.clone(), n.id.clone())),
            Tab::Pods => self
                .pods
                .get(self.selected_index)
                .map(|p| (Tab::Pods, p.engine.clone(), p.id.clone())),
        }
    }

    /// Find related resources (e.g. running/stopped containers dependent on an image).
    pub fn get_related_resources(
        &self,
        resource_type: &Tab,
        id: &str,
    ) -> Vec<(Tab, String, String)> {
        let mut related = Vec::new();
        if resource_type == &Tab::Images {
            let image_names = self
                .images
                .iter()
                .find(|i| i.id == id)
                .map_or_else(Vec::new, super::podman::models::Image::get_names);

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

    /// Propagate error message to UI status bar and tracing log.
    pub fn propagate_error(&mut self, message: String) {
        tracing::warn!("Runtime error: {message}");
        self.status_message = Some(message);
    }

    /// Execute the primary interactive run loop.
    ///
    /// # Errors
    /// Returns an error if terminal rendering or raw mode setup fails.
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut events = EventHandler::new(250);
        self.action_tx = Some(events.sender.clone());

        dispatcher::trigger_refresh_data(self);

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

                let mut args = vec!["exec".to_string(), "-it".to_string(), cmd];
                let custom_cmd = self
                    .exec_form
                    .take()
                    .map_or_else(|| "/bin/sh".to_string(), |f| f.command);

                let custom_args =
                    shlex::split(&custom_cmd).unwrap_or_else(|| vec!["/bin/sh".to_string()]);
                if custom_args.is_empty() {
                    args.push("/bin/sh".to_string());
                } else {
                    args.extend(custom_args);
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
                dispatcher::trigger_refresh_data(self);
            }
        }

        Ok(())
    }

    /// Process an Action message from the event channel.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Tick => (),
            Action::DataRefreshed {
                running,
                stopped,
                images,
                volumes,
                networks,
                pods,
            } => {
                self.running = running;
                self.stopped = stopped;
                self.images = images;
                self.volumes = volumes;
                self.networks = networks;
                self.pods = pods;
                self.running_index =
                    std::cmp::min(self.running_index, self.running.len().saturating_sub(1));
                self.stopped_index =
                    std::cmp::min(self.stopped_index, self.stopped.len().saturating_sub(1));
                self.images_index =
                    std::cmp::min(self.images_index, self.images.len().saturating_sub(1));
                self.volumes_index =
                    std::cmp::min(self.volumes_index, self.volumes.len().saturating_sub(1));
                self.networks_index =
                    std::cmp::min(self.networks_index, self.networks.len().saturating_sub(1));
                self.pods_index = std::cmp::min(self.pods_index, self.pods.len().saturating_sub(1));

                self.load_current_index();
                dispatcher::trigger_fetch_logs(self);
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
            Action::ImageHistoryRefreshed { history } => {
                self.image_history = history;
            }
            Action::PruneComplete { message } => {
                self.status_message = Some(message);
                dispatcher::trigger_refresh_data(self);
            }
            Action::PullComplete => {
                self.is_pulling = false;
                self.direct_pull_form = None;
                self.search_image_form = None;
                dispatcher::trigger_refresh_data(self);
            }
            Action::ActionComplete => {
                self.status_message = None;
                dispatcher::trigger_refresh_data(self);
            }
            Action::Error { message } => {
                self.propagate_error(message);
            }
            Action::Key(k) => input::handle_key(self, k),
            Action::Mouse(m) => mouse::handle_mouse(self, m),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use async_trait::async_trait;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
                engine: engines.first().cloned().unwrap_or_else(|| "mock".into()),
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
                size: Some(serde_json::json!(5000)),
                virtual_size: None,
                created: Some(serde_json::Value::Number(1_678_901_234.into())),
                dangling: None,
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
        async fn search_images(
            &self,
            _engines: &[String],
            _term: &str,
        ) -> Result<Vec<SearchResult>> {
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
        async fn prune_images(&self, _engines: &[String], _all: bool) -> Result<String> {
            Ok("Pruned 0 B".into())
        }
        async fn tag_image(&self, _engine: &str, _image_id: &str, _target_tag: &str) -> Result<()> {
            Ok(())
        }
        async fn get_image_history(&self, _engine: &str, _image_id: &str) -> Result<Vec<String>> {
            Ok(vec!["LAYER 1 (10MB)".into(), "LAYER 2 (5MB)".into()])
        }
    }

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.active_tab, Tab::Running);
        assert!(!app.should_quit);
        assert_eq!(app.engine_view, EngineView::Both);
        assert!(!app.filter_dangling_images);
        assert!(app.tag_image_form.is_none());
        assert!(app.image_history.is_empty());
    }

    #[test]
    fn test_dangling_filter_and_tagging() {
        let mut app = App::new();
        let tagged = Image {
            id: "img1".into(),
            parent_id: None,
            repo_tags: Some(serde_json::json!(["alpine:latest"])),
            repository: Some("alpine".into()),
            tag: Some("latest".into()),
            names: None,
            size: None,
            virtual_size: None,
            created: None,
            dangling: None,
            engine: "podman".into(),
        };
        let dangling = Image {
            id: "img2".into(),
            parent_id: None,
            repo_tags: None,
            repository: None,
            tag: None,
            names: Some(serde_json::json!(["<none>:<none>"])),
            size: None,
            virtual_size: None,
            created: None,
            dangling: None,
            engine: "podman".into(),
        };

        app.images = vec![tagged, dangling];
        assert_eq!(app.get_filtered_images().len(), 2);
        assert_eq!(app.get_list_len_for_tab(&Tab::Images), 2);

        app.filter_dangling_images = true;
        assert_eq!(app.get_filtered_images().len(), 1);
        assert_eq!(app.get_list_len_for_tab(&Tab::Images), 1);
        assert_eq!(app.get_filtered_images()[0].id, "img2");

        app.active_tab = Tab::Images;
        app.selected_index = 0;
        assert_eq!(
            app.get_selected_resource(),
            Some((Tab::Images, "podman".into(), "img2".into()))
        );
    }

    #[tokio::test]
    async fn test_app_update_navigation() {
        let mut app = App::with_client(Box::new(MockEngine));

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

        app.update(Action::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        assert_eq!(app.active_tab, Tab::Stopped);
        app.update(Action::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        assert_eq!(app.active_tab, Tab::Images);
        app.update(Action::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        assert_eq!(app.active_tab, Tab::Volumes);
        app.update(Action::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        assert_eq!(app.active_tab, Tab::Networks);
        app.update(Action::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        assert_eq!(app.active_tab, Tab::Pods);
        app.update(Action::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::empty(),
        )));
        assert!(app.logs_focused);

        app.update(Action::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::empty(),
        )));
        assert!(!app.logs_focused);

        assert_eq!(app.engine_view, EngineView::Both);
        app.update(Action::Key(KeyEvent::new(
            KeyCode::Char('E'),
            KeyModifiers::empty(),
        )));
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
        app.update(Action::Error {
            message: "Test error".to_string(),
        });
        assert_eq!(app.status_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_inspect_popup() {
        let mut app = App::with_client(Box::new(MockEngine));
        app.update(Action::InspectResult {
            output: "test output".to_string(),
        });
        assert_eq!(app.inspect_popup, Some("test output".to_string()));
    }
}
