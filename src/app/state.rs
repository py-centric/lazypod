/// Active resource tab in the left panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Running,
    Stopped,
    Images,
    Volumes,
    Networks,
    Pods,
}

impl Tab {
    /// Ordered list of all tabs for keyboard navigation.
    #[must_use]
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Running,
            Tab::Stopped,
            Tab::Images,
            Tab::Volumes,
            Tab::Networks,
            Tab::Pods,
        ]
    }

    /// Next tab in cyclic order.
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Tab::Running => Tab::Stopped,
            Tab::Stopped => Tab::Images,
            Tab::Images => Tab::Volumes,
            Tab::Volumes => Tab::Networks,
            Tab::Networks => Tab::Pods,
            Tab::Pods => Tab::Running,
        }
    }

    /// Previous tab in cyclic order.
    #[must_use]
    pub fn prev(&self) -> Self {
        match self {
            Tab::Running => Tab::Pods,
            Tab::Stopped => Tab::Running,
            Tab::Images => Tab::Stopped,
            Tab::Volumes => Tab::Images,
            Tab::Networks => Tab::Volumes,
            Tab::Pods => Tab::Networks,
        }
    }
}

/// Active engine filter view.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum EngineView {
    #[default]
    Both,
    Docker,
    Podman,
}

impl EngineView {
    /// Cycle to the next engine filter.
    pub fn next(&mut self) {
        *self = match self {
            EngineView::Both => EngineView::Docker,
            EngineView::Docker => EngineView::Podman,
            EngineView::Podman => EngineView::Both,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_cycle() {
        let mut tab = Tab::Running;
        tab = tab.next();
        assert_eq!(tab, Tab::Stopped);
        tab = tab.next();
        assert_eq!(tab, Tab::Images);
        tab = tab.prev();
        assert_eq!(tab, Tab::Stopped);
        tab = tab.prev();
        assert_eq!(tab, Tab::Running);
        tab = tab.prev();
        assert_eq!(tab, Tab::Pods);
    }

    #[test]
    fn test_engine_view_cycle() {
        let mut view = EngineView::Both;
        view.next();
        assert_eq!(view, EngineView::Docker);
        view.next();
        assert_eq!(view, EngineView::Podman);
        view.next();
        assert_eq!(view, EngineView::Both);
    }
}
