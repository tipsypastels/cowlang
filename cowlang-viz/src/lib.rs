mod event;
mod io;
mod render;

use self::{
    event::{Event, Events},
    io::{InputRx, OutputRx},
};
use anyhow::{Context, Result};
use cowlang::{Cowlang, Program};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::DefaultTerminal;

pub struct Options<'a> {
    pub program: Program<'a>,
}

pub async fn vizualize<'a>(options: Options<'a>) -> Result<()> {
    let mut term = ratatui::init();

    let (mut input_tx, input_rx) = crate::io::input();
    let (mut output_tx, output_rx) = crate::io::output();

    let framerate = FramerateOption::default();
    let events = Events::new(framerate.fps());

    let interp = Cowlang::new(cowlang::Options {
        program: options.program,
        input: &mut input_tx,
        output: &mut output_tx,
    });

    let app = App {
        interp,
        input_rx,
        output_rx,
        output_with_spaces: false,
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
    input_rx: InputRx,
    output_rx: OutputRx,
    output_with_spaces: bool,
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
                    if self.input_rx.has_current_op() {
                        self.handle_input_key_event(event);
                    } else {
                        self.handle_key_event(event);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_input_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(char) => {
                self.input_rx.push_char_to_current_op(char);
            }
            KeyCode::Enter => {
                self.input_rx.finish_current_op();
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('s') => {
                self.output_with_spaces = !self.output_with_spaces;
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

    fn draw(&mut self, term: &mut DefaultTerminal) -> Result<()> {
        term.draw(|frame| {
            let render_app = crate::render::RenderApp {
                interp: &self.interp,
                input_rx: &self.input_rx,
                output_rx: &self.output_rx,
                output_with_spaces: self.output_with_spaces,
                framerate: self.framerate,
            };
            crate::render::render(&render_app, frame);
        })
        .map(|_| ())
        .context("failed to draw frame")
    }

    fn tick(&mut self) -> Result<()> {
        self.interp.advance()?;
        self.input_rx.tick();
        self.output_rx.tick();

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
