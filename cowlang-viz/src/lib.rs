mod event;
mod io;
mod render;

use self::{
    event::{Event, Events},
    io::OutputRx,
};
use anyhow::{Context, Result};
use cowlang::{Cowlang, Program};
use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;

pub struct Options<'a> {
    pub program: Program<'a>,
}

pub async fn vizualize<'a>(options: Options<'a>) -> Result<()> {
    let mut term = ratatui::init();

    let (mut output_tx, output_rx) = crate::io::output();

    let framerate = FramerateOption::default();
    let events = Events::new(framerate.fps());

    let mut input = crate::io::InputTemp;

    let interp = Cowlang::new(cowlang::Options {
        program: options.program,
        input: &mut input,
        output: &mut output_tx,
    });

    let app = App {
        interp,
        writer_rx: output_rx,
        writer_with_spaces: false,
        framerate,
        events,
        quit: false,
    };

    let res = app.run(&mut term).await;

    ratatui::restore();
    res
}

struct App<'a> {
    interp: Cowlang<'a>,
    writer_rx: OutputRx,
    writer_with_spaces: bool,
    framerate: FramerateOption,
    events: Events,
    quit: bool,
}

impl App<'_> {
    async fn run(mut self, term: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            self.draw(term)?;

            match self.events.next().await? {
                Event::Tick => {
                    self.tick()?;
                }
                Event::Term(crossterm::event::Event::Key(event)) if event.is_press() => {
                    match event.code {
                        KeyCode::Char('s') => {
                            self.writer_with_spaces = !self.writer_with_spaces;
                        }
                        KeyCode::Char('f') => {
                            self.framerate = self.framerate.next();
                            self.events.set_fps(self.framerate.fps());
                        }
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
                writer_with_spaces: self.writer_with_spaces,
                framerate: self.framerate,
            };
            crate::render::render(&render_app, frame);
        })
        .map(|_| ())
        .context("failed to draw frame")
    }

    fn tick(&mut self) -> Result<()> {
        self.interp.advance()?;
        self.writer_rx.tick();

        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone)]
enum FramerateOption {
    #[default]
    _1,
    _5,
    _10,
    _30,
}

impl FramerateOption {
    fn fps(self) -> f64 {
        match self {
            Self::_1 => 1.0,
            Self::_5 => 5.0,
            Self::_10 => 10.0,
            Self::_30 => 30.0,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::_1 => Self::_5,
            Self::_5 => Self::_10,
            Self::_10 => Self::_30,
            Self::_30 => Self::_1,
        }
    }
}
