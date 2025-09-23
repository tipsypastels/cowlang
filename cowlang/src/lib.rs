mod command;
mod io;

pub use self::command::{Command, Program};

use self::io::*;
use std::io::{Read, Write};

pub struct Cowlang<'a> {
    memory: Vec<u8>,
    memory_idx: usize,
    program: Program<'a>,
    program_idx: usize,
    reader: MaybeDynRead<'a>,
    writer: MaybeDynWrite<'a>,
    register: Option<u8>,
    aborted: bool,
}

impl<'a> Cowlang<'a> {
    pub fn new(program: Program<'a>) -> Self {
        Self {
            memory: vec![0],
            memory_idx: 0,
            program,
            program_idx: 0,
            reader: MaybeDynRead::default(),
            writer: MaybeDynWrite::default(),
            register: None,
            aborted: false,
        }
    }

    pub fn with_reader(&mut self, reader: &'a mut dyn Read) -> &mut Self {
        self.reader = MaybeDynRead::Dyn(reader);
        self
    }

    pub fn with_writer(&mut self, writer: &'a mut dyn Write) -> &mut Self {
        self.writer = MaybeDynWrite::Dyn(writer);
        self
    }

    pub fn with_stderr_writer(&mut self) -> &mut Self {
        self.writer = MaybeDynWrite::default_stderr();
        self
    }

    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn memory_idx(&self) -> usize {
        self.memory_idx
    }

    pub fn program(&self) -> &[Command] {
        &self.program
    }

    pub fn program_idx(&self) -> usize {
        self.program_idx
    }

    pub fn current_instruction(&self) -> Option<Command> {
        self.program.get(self.program_idx).copied()
    }

    pub fn current_value(&self) -> u8 {
        self.memory[self.memory_idx]
    }

    pub fn register(&self) -> Option<u8> {
        self.register
    }

    pub fn aborted(&self) -> bool {
        self.aborted
    }

    pub fn completed(&self) -> bool {
        self.aborted || self.program_idx >= self.program.len()
    }

    pub fn run(&mut self) {
        while !self.completed() {
            self.advance();
        }
    }

    pub fn advance(&mut self) {
        if self.aborted {
            return;
        }
        if let Some(&command) = self.program.get(self.program_idx) {
            self.evaluate(command);
            self.program_idx += 1;
        }
    }

    fn evaluate(&mut self, command: Command) {
        // eprintln!("running command {command:?}");

        macro_rules! value {
            () => {
                self.memory[self.memory_idx]
            };
        }

        macro_rules! abort {
            ($reason:literal) => {
                // eprintln!(concat!("aborting, ", $reason));
                self.aborted = true;
                return;
            };
        }

        match command {
            Command::moo => {
                self.program_idx = self.program_idx.saturating_sub(1);

                let mut unmatched_moos = 1;

                while unmatched_moos > 0 {
                    if self.program_idx == 0 {
                        abort!("beginless backwards jump");
                    }

                    self.program_idx -= 1;

                    match self.current_instruction() {
                        Some(Command::moo) => {
                            unmatched_moos += 1;
                        }
                        Some(Command::MOO) => {
                            unmatched_moos -= 1;
                        }
                        _ => {}
                    }
                }

                if let Some(command) = self.current_instruction() {
                    self.evaluate(command);
                }
            }
            Command::mOo => {
                self.memory_idx = self.memory_idx.saturating_sub(1);
            }
            Command::moO => {
                self.memory_idx = self.memory_idx.saturating_add(1);

                if self.memory_idx == self.memory.len() {
                    // eprintln!("growing memory");
                    self.memory.push(0);
                }
            }
            Command::mOO => {
                let value = value!();

                if value == Command::mOO as u8 {
                    abort!("recursive exec");
                }
                let Ok(executed_command) = Command::try_from(value) else {
                    abort!("invalid command");
                };

                self.evaluate(executed_command);
            }
            Command::Moo => {
                let value = &mut value!();

                if *value == 0 {
                    let mut input = [0u8; 1];

                    match self.reader.read_exact(&mut input) {
                        Ok(()) => {
                            // eprintln!("read a char: {}", input[0]);
                            *value = input[0];
                        }
                        Err(_) => {
                            // eprintln!("failed to read a char: {e}")
                        }
                    }
                } else {
                    let char = char::from(*value);

                    match self
                        .writer
                        .write_fmt(format_args!("{char}"))
                        .and_then(|()| self.writer.flush())
                    {
                        Ok(()) => {
                            // eprintln!("wrote a char: {char}");
                        }
                        Err(_) => {
                            // eprintln!("failed to write a char: {e}");
                        }
                    }
                }
            }
            Command::MOo => {
                value!() = value!().saturating_sub(1);
            }
            Command::MoO => {
                value!() = value!().saturating_add(1);
            }
            Command::MOO => {
                if value!() == 0 {
                    #[allow(non_snake_case)]
                    let mut unmatched_MOOs = 1;
                    let mut prev_command;

                    self.program_idx = self.program_idx.saturating_add(1);

                    while unmatched_MOOs > 0 {
                        let Some(command) = self.current_instruction() else {
                            abort!("endless forward jump");
                        };

                        prev_command = command;
                        self.program_idx += 1;

                        match self.current_instruction() {
                            Some(Command::moo) => {
                                unmatched_MOOs -= 1;

                                if matches!(prev_command, Command::MOO) {
                                    unmatched_MOOs -= 1;
                                }
                            }
                            Some(Command::MOO) => {
                                unmatched_MOOs += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Command::OOO => {
                value!() = 0;
            }
            Command::MMM => match self.register.take() {
                Some(register_value) => {
                    value!() = register_value;
                }
                None => {
                    self.register = Some(value!());
                }
            },
            Command::OOM => {
                match self
                    .writer
                    .write_all(&[value!()])
                    .and_then(|()| self.writer.flush())
                {
                    Ok(()) => {
                        // eprintln!("wrote an int");
                    }
                    Err(_) => {
                        // eprintln!("failed to write an int: {e}");
                    }
                }
            }
            Command::oom => {
                let mut input = [0u8; 1];

                match self.reader.read_exact(&mut input) {
                    Ok(()) => {
                        // eprintln!("read an int: {}", input[0]);
                        value!() = input[0];
                    }
                    Err(_) => {
                        // eprintln!("failed to read an int: {e}")
                    }
                }
            }
        }
    }
}
