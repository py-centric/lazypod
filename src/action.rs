use crossterm::event::{KeyEvent, MouseEvent};
use crate::podman::{Container, Image, Network, Pod, SearchResult, Volume};

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Action {
    Tick,
    Quit,
    Key(KeyEvent),
    Mouse(MouseEvent),
    DataRefreshed {
        running: Vec<Container>,
        stopped: Vec<Container>,
        images: Vec<Image>,
        volumes: Vec<Volume>,
        networks: Vec<Network>,
        pods: Vec<Pod>,
    },
    LogsRefreshed {
        logs: Vec<String>,
    },
    SearchResults {
        results: Vec<SearchResult>,
    },
    InspectResult {
        output: String,
    },
    PullComplete,
    ActionComplete,
    Error {
        message: String,
    },
}
