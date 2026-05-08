use spinoff::{spinners, Color, Spinner as Inner, Streams};
use std::io::{self, IsTerminal};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const TICK: Duration = Duration::from_millis(100);

pub struct Spinner {
    stop: Sender<()>,
    handle: Option<JoinHandle<()>>,
}

impl Spinner {
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();
        if !io::stderr().is_terminal() {
            return Self {
                stop: tx,
                handle: None,
            };
        }
        let handle = thread::spawn(move || run(rx));
        Self {
            stop: tx,
            handle: Some(handle),
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn run(rx: mpsc::Receiver<()>) {
    let start = Instant::now();
    if rx.recv_timeout(TICK).is_ok() {
        return;
    }
    let mut sp = Inner::new_with_stream(spinners::Dots9, "", Color::White, Streams::Stderr);
    loop {
        let elapsed = start.elapsed();
        let display = if elapsed.as_millis() < 1000 {
            format!("{}ms", elapsed.as_millis())
        } else {
            format!("{:.1}s", elapsed.as_secs_f64())
        };
        sp.update_text(display);
        if rx.recv_timeout(TICK).is_ok() {
            break;
        }
    }
    sp.clear();
}
