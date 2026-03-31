use crate::action::Action;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct EventHandler {
    pub _sender: mpsc::UnboundedSender<Action>,
    pub receiver: mpsc::UnboundedReceiver<Action>,
    stop_signal: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = Arc::clone(&stop_signal);

        std::thread::spawn(move || {
            let tick_rate = Duration::from_millis(tick_rate);
            let mut last_tick = Instant::now();
            while !stop_signal_clone.load(Ordering::SeqCst) {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::from_millis(0));

                if event::poll(timeout).expect("failed to poll new events") {
                    match event::read().expect("failed to read event") {
                        CrosstermEvent::Key(key) => {
                            if key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
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

        Self {
            _sender: sender,
            receiver,
            stop_signal,
        }
    }

    pub async fn next(&mut self) -> Option<Action> {
        self.receiver.recv().await
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
    }
}
