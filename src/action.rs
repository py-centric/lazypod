use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Tick,
    Quit,
    Key(KeyEvent),
    Mouse(MouseEvent),
}
