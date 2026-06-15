use crossterm::event::{KeyEvent, MouseEvent};
use crate::podman::{Container, Image, Network, SearchResult, Volume};

#[derive(Debug, Clone, PartialEq)]
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
    },
    LogsRefreshed {
        logs: Vec<String>,
    },
    SearchResults {
        results: Vec<SearchResult>,
    },
    PullComplete,
    ActionComplete,
}
