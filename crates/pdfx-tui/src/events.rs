use crossbeam_channel::{bounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

pub struct EventHandler {
    pub receiver: Receiver<AppEvent>,
    _sender: Sender<AppEvent>,
    is_running: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let (sender, receiver) = bounded(100);
        let is_running = Arc::new(AtomicBool::new(true));

        let tx = sender.clone();
        let running = Arc::clone(&is_running);

        thread::spawn(move || {
            let tick_rate = Duration::from_millis(tick_rate_ms);

            while running.load(Ordering::Relaxed) {
                if event::poll(tick_rate).unwrap_or(false) {
                    let ev = match event::read() {
                        Ok(Event::Key(key)) => Some(AppEvent::Key(key)),
                        Ok(Event::Mouse(mouse)) => Some(AppEvent::Mouse(mouse)),
                        Ok(Event::Resize(w, h)) => Some(AppEvent::Resize(w, h)),
                        _ => None,
                    };
                    if let Some(app_ev) = ev {
                        if tx.send(app_ev).is_err() {
                            break;
                        }
                    }
                } else if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        Self {
            receiver,
            _sender: sender,
            is_running,
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
