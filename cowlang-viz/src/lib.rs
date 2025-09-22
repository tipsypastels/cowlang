mod event;
mod io;
mod render;

use self::{
    event::{Event, Events},
    io::WriterRx,
};
use anyhow::{Context, Result};
use cowlang::{Cowlang, Program};
use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;

pub async fn vizualize<'a>(program: Program<'a>) -> Result<()> {
    let mut term = ratatui::init();

    let (mut writer_tx, writer_rx) = crate::io::writer();
    let mut interp = Cowlang::new(program);

    interp.with_writer(&mut writer_tx);

    let app = App {
        interp,
        writer_rx,
        events: Events::new(),
        quit: false,
    };

    let res = app.run(&mut term).await;

    ratatui::restore();
    res
}

struct App<'a> {
    interp: Cowlang<'a>,
    writer_rx: WriterRx,
    events: Events,
    quit: bool,
}

impl App<'_> {
    async fn run(mut self, term: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            self.draw(term)?;

            match self.events.next().await? {
                Event::Tick => {
                    self.tick();
                }
                Event::Term(crossterm::event::Event::Key(event)) if event.is_press() => {
                    match event.code {
                        KeyCode::Char('q') => {
                            self.quit = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn draw(&mut self, term: &mut DefaultTerminal) -> Result<()> {
        term.draw(|frame| {
            let render_app = crate::render::RenderApp {
                interp: &self.interp,
                writer_rx: &self.writer_rx,
            };
            crate::render::render(&render_app, frame);
        })
        .map(|_| ())
        .context("failed to draw frame")
    }

    fn tick(&mut self) {
        self.interp.advance();
        self.writer_rx.tick();
    }
}
