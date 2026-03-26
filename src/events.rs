use crate::action::Action;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct EventHandler {
    pub _sender: mpsc::UnboundedSender<Action>,
    pub receiver: mpsc::UnboundedReceiver<Action>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        
        std::thread::spawn(move || {
            let tick_rate = Duration::from_millis(tick_rate);
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_millis(0));
                
                if event::poll(timeout).expect("failed to poll new events") {
                    match event::read().expect("failed to read event") {
                        CrosstermEvent::Key(key) => {
                            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                let _ = _sender.send(Action::Quit);
                                break;
                            } else {
                                let _ = _sender.send(Action::Key(key));
                            }
                        }
                        CrosstermEvent::Mouse(mouse) => {
                            let _ = _sender.send(Action::Mouse(mouse));
                        }
                        _ => {}
                    }
                }
                
                if last_tick.elapsed() >= tick_rate {
                    let _ = _sender.send(Action::Tick);
                    last_tick = Instant::now();
                }
            }
        });
        
        Self { _sender: sender, receiver }
    }

    pub async fn next(&mut self) -> Option<Action> {
        self.receiver.recv().await
    }
}
