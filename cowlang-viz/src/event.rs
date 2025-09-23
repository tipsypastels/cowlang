use anyhow::{Context, Result};
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Tick,
    Term(crossterm::event::Event),
}

pub struct Events {
    tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    abort_handle: tokio::task::AbortHandle,
}

impl Events {
    pub fn new(fps: f64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let actor = Actor { tx: tx.clone() };
        let task = tokio::spawn(async move { actor.run(fps).await });

        Self {
            tx,
            rx,
            abort_handle: task.abort_handle(),
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx.recv().await.context("failed to receive event")
    }

    pub fn set_fps(&mut self, fps: f64) {
        let actor = Actor {
            tx: self.tx.clone(),
        };

        self.abort_handle.abort();
        self.abort_handle = tokio::spawn(async move { actor.run(fps).await }).abort_handle();
    }
}

struct Actor {
    tx: mpsc::UnboundedSender<Event>,
}

impl Actor {
    async fn run(self, fps: f64) {
        let tick_rate = Duration::from_secs_f64(1.0 / fps);
        let mut reader = EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);

        loop {
            tokio::select! {
                _ = self.tx.closed() => {
                    break;
                }
                _ = tick.tick() => {
                    _ = self.tx.send(Event::Tick);
                }
                Some(Ok(event)) = reader.next().fuse() => {
                    _ = self.tx.send(Event::Term(event));
                }
            }
        }
    }
}
