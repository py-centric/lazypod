#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Running,
    Stopped,
    Images,
    Volumes,
    Networks,
    Pods,
}

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
