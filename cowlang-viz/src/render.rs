use crate::io::WriterRx;
use cowlang::{Command, Cowlang};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, Wrap},
};
use std::borrow::Cow;

pub struct RenderApp<'f, 'a> {
    pub interp: &'f Cowlang<'a>,
    pub writer_rx: &'f WriterRx,
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

    render_writer_output(app, bottom_cols[0], buf);
    render_register(app, bottom_cols[1], buf);
}

fn render_memory(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let mut line = Line::default();

    for (i, byte) in app.interp.memory().iter().enumerate() {
        let is_current = i == app.interp.memory_idx();
        let value = format!("{byte:08b}");
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

fn render_writer_output(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title(Line::styled(" Written ", Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    Paragraph::new(format!("{}", app.writer_rx.display()))
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

    let block = Block::bordered()
        .title(Line::styled(" Program ", Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(line).wrap(Wrap { trim: true }).block(block);

    paragraph.render(area, buf);
}

fn render_current_instruction(app: &RenderApp, area: Rect, buf: &mut Buffer) {
    let command = app.interp.program().get(app.interp.program_idx()).copied();
    let title = command
        .as_ref()
        .map(|c| Cow::Owned(format!("Instruction: {c:?}")))
        .unwrap_or("Instruction: ???".into());

    let line = match command {
        Some(Command::moo) => Line::from(vec![
            Span::raw("jumps backwards to the nearest "),
            Span::styled("MOO", Color::Cyan),
        ]),
        Some(Command::mOo) => Line::from(vec![
            Span::raw("moves the "),
            Span::styled("cursor", Modifier::UNDERLINED),
            Span::raw(" backward"),
        ]),
        Some(Command::moO) => Line::from(vec![
            Span::raw("moves the "),
            Span::styled("cursor", Modifier::UNDERLINED),
            Span::raw(" forward"),
        ]),
        Some(Command::mOO) => Line::from(vec![
            Span::raw("evaluate the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw(" as an instruction"),
        ]),
        Some(Command::Moo) => Line::from(vec![
            Span::raw("if the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw("is 0, read it from stdin, else write it to stdout"),
        ]),
        Some(Command::MOo) => Line::from(vec![
            Span::raw("decrement the "),
            Span::styled("current value", Modifier::UNDERLINED),
        ]),
        Some(Command::MoO) => Line::from(vec![
            Span::raw("increment the "),
            Span::styled("current value", Modifier::UNDERLINED),
        ]),
        Some(Command::MOO) => Line::from(vec![
            Span::raw("if the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw("is 0, skip the next command and jump to the next "),
            Span::styled("moo", Style::new().bg(Color::Cyan)),
        ]),
        Some(Command::OOO) => Line::from(vec![
            Span::raw("set the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw("to 0"),
        ]),
        Some(Command::MMM) => Line::from(vec![
            Span::raw("if the register is empty, set it to the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw("vice-versa otherwise"),
        ]),
        Some(Command::OOM) => Line::from(vec![
            Span::raw("write the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw("to stdout"),
        ]),
        Some(Command::oom) => Line::from(vec![
            Span::raw("read the "),
            Span::styled("current value", Modifier::UNDERLINED),
            Span::raw("from stdout"),
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
    } else if app.interp.aborted() {
        "Aborted"
    } else if app.interp.skipping() {
        "Skipping"
    } else {
        "Running"
    };

    let block = Block::bordered()
        .title(Line::styled(" State ", Modifier::BOLD).centered())
        .padding(Padding::uniform(1));

    let paragraph = Paragraph::new(state).block(block);

    paragraph.render(area, buf);
}
