use crate::{FramerateOption, io::OutputRx};
use cowlang::{Command, Cowlang};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, Wrap},
};
use std::borrow::Cow;

pub struct RenderApp<'f, 'a> {
    pub interp: &'f Cowlang<'a>,
    pub output_rx: &'f OutputRx,
    pub output_with_spaces: bool,
    pub framerate: FramerateOption,
}

pub fn render(app: &RenderApp, frame: &mut Frame) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(area);

    render_left_col(app, cols[0], buf);
    render_right_col(app, cols[1], buf);
}

fn render_left_col(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let rows =
        Layout::vertical([Constraint::Percentage(75), Constraint::Percentage(25)]).split(area);

    render_memory(app, rows[0], buf);

    let bottom_cols = Layout::horizontal([Constraint::Percentage(85), Constraint::Percentage(15)])
        .spacing(1)
        .split(rows[1]);

    render_output(app, bottom_cols[0], buf);
    render_register(app, bottom_cols[1], buf);
}

fn render_memory(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let mut line = Line::default();

    for (i, int) in app.interp.memory().iter().enumerate() {
        let is_current = i == app.interp.memory_idx();
        let value = format!("{int}");
        let span = if is_current {
            Span::styled(value, Modifier::UNDERLINED)
        } else {
            Span::raw(value)
        };

        line.push_span(span);
        line.push_span(Span::raw(" "));
    }

    let block = Block::bordered()
        .title(Line::styled(" Memory ", Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(line).wrap(Wrap { trim: true }).block(block);

    paragraph.render(area, buf);
}

fn render_output(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let (value, controls) = if app.output_with_spaces {
        (app.output_rx.as_str_with_spaces(), " Hide dividers <S> ")
    } else {
        (app.output_rx.as_str(), " Show dividers <S> ")
    };

    let block = Block::bordered()
        .title(Line::styled(" Written ", Modifier::BOLD).centered())
        .title_bottom(Line::styled(controls, Modifier::BOLD))
        .padding(Padding::uniform(1));

    Paragraph::new(value)
        .block(block)
        .wrap(Wrap { trim: true })
        .render(area, buf);
}

fn render_register(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title(Line::styled(" Register ", Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    if let Some(register) = app.interp.register() {
        Paragraph::new(register.to_string())
            .block(block)
            .render(area, buf);
    } else {
        block.render(area, buf);
    }
}

fn render_right_col(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let rows =
        Layout::vertical([Constraint::Percentage(75), Constraint::Percentage(25)]).split(area);

    render_program(app, rows[0], buf);

    let bottom_cols = Layout::horizontal([Constraint::Percentage(85), Constraint::Percentage(15)])
        .spacing(1)
        .split(rows[1]);

    render_current_instruction(app, bottom_cols[0], buf);
    render_current_state(app, bottom_cols[1], buf);
}

fn render_program(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let mut line = Line::default();

    for (i, command) in app.interp.program().iter().enumerate() {
        let is_current = i == app.interp.program_idx();
        let value = format!("{command:?}");

        let mut style = Style::new();

        if is_current {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        match command {
            Command::moo => {
                style = style.bg(Color::Cyan);
            }
            Command::MOO => {
                style = style.fg(Color::Cyan);
            }
            _ => {}
        }

        line.push_span(Span::styled(value, style));
        line.push_span(Span::raw(" "));
    }

    let fps = app.framerate.fps();

    let block = Block::bordered()
        .title(Line::styled(" Program ", Modifier::BOLD).centered())
        .title_bottom(Line::styled(
            format!(" Change ticks/s <F> ({fps}) "),
            Modifier::BOLD,
        ))
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(line).wrap(Wrap { trim: true }).block(block);

    paragraph.render(area, buf);
}

fn render_current_instruction(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let command = app.interp.current_instruction();

    let cursor = || Span::styled("    ", Modifier::UNDERLINED);
    let current_value = || Span::styled("current value", Modifier::UNDERLINED);
    let check = |c: bool| {
        if c {
            Span::styled("✓", Color::Green)
        } else {
            Span::styled("×", Color::Red)
        }
    };

    let title = command
        .as_ref()
        .map(|c| Cow::Owned(format!("Instruction: {c:?}")))
        .unwrap_or("Instruction: ???".into());

    let line = match command {
        Some(Command::moo) => Line::from(vec![
            Span::raw("jump backwards to "),
            Span::styled("MOO", Color::Cyan),
        ]),
        Some(Command::mOo) => Line::from(vec![
            Span::raw("move the "),
            cursor(),
            Span::raw(" backward"),
        ]),
        Some(Command::moO) => Line::from(vec![
            Span::raw("move the "),
            cursor(),
            Span::raw(" forward"),
        ]),
        Some(Command::mOO) => Line::from(vec![
            Span::raw("evaluate the "),
            current_value(),
            Span::raw(" as an instruction"),
        ]),
        Some(Command::Moo) => Line::from(vec![
            Span::raw("if the "),
            current_value(),
            Span::raw(" is 0 ("),
            check(app.interp.current_value() == 0),
            Span::raw("), read it from stdin, else write it to stdout"),
        ]),
        Some(Command::MOo) => Line::from(vec![Span::raw("decrement the "), current_value()]),
        Some(Command::MoO) => Line::from(vec![Span::raw("increment the "), current_value()]),
        Some(Command::MOO) => Line::from(vec![
            Span::raw("if the "),
            current_value(),
            Span::raw(" is 0 ("),
            check(app.interp.current_value() == 0),
            Span::raw("), skip next command and jump to "),
            Span::styled("moo", Style::new().bg(Color::Cyan)),
        ]),
        Some(Command::OOO) => Line::from(vec![
            Span::raw("set the "),
            current_value(),
            Span::raw(" to 0"),
        ]),
        Some(Command::MMM) => Line::from(vec![
            Span::raw("if the register is empty ("),
            check(app.interp.register().is_none()),
            Span::raw("), set it to the "),
            current_value(),
            Span::raw(" vice-versa otherwise"),
        ]),
        Some(Command::OOM) => Line::from(vec![
            Span::raw("write the "),
            current_value(),
            Span::raw(" to stdout"),
        ]),
        Some(Command::oom) => Line::from(vec![
            Span::raw("read the "),
            current_value(),
            Span::raw(" from stdin"),
        ]),
        None => Line::raw("<no instruction>"),
    };

    let block = Block::bordered()
        .title(Line::styled(title, Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(line).block(block);

    paragraph.render(area, buf);
}

fn render_current_state(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let state = if app.interp.completed() {
        "Completed"
    } else {
        "Running"
    };

    let block = Block::bordered()
        .title(Line::styled(" State ", Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(state).block(block);

    paragraph.render(area, buf);
}
