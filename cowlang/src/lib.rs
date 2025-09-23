mod command;
mod io;

pub use self::{
    command::{Command, Program},
    io::{Input, Output},
};

pub struct Cowlang<'a> {
    memory: Vec<u32>,
    memory_idx: usize,
    program: Program<'a>,
    program_idx: usize,
    input: &'a mut dyn Input,
    output: &'a mut dyn Output,
    register: Option<u32>,
}

pub struct Options<'a> {
    pub program: Program<'a>,
    pub input: &'a mut dyn Input,
    pub output: &'a mut dyn Output,
}

impl<'a> Cowlang<'a> {
    pub fn new(options: Options<'a>) -> Self {
        Self {
            memory: vec![0],
            memory_idx: 0,
            program: options.program,
            program_idx: 0,
            input: options.input,
            output: options.output,
            register: None,
        }
    }

    pub fn memory(&self) -> &[u32] {
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

    pub fn current_value(&self) -> u32 {
        self.memory[self.memory_idx]
    }

    pub fn register(&self) -> Option<u32> {
        self.register
    }

    pub fn completed(&self) -> bool {
        self.program_idx >= self.program.len()
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while !self.completed() {
            self.advance()?;
        }
        Ok(())
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        if let Some(&command) = self.program.get(self.program_idx) {
            self.evaluate(command)?;
            self.program_idx += 1;
        }
        Ok(())
    }

    fn evaluate(&mut self, command: Command) -> Result<(), Error> {
        macro_rules! value {
            () => {
                self.memory[self.memory_idx]
            };
        }

        match command {
            Command::moo => {
                self.program_idx = self.program_idx.saturating_sub(1);

                let mut unmatched_moos = 1;

                while unmatched_moos > 0 {
                    if self.program_idx == 0 {
                        return Err(Error::BeginlessJumpBackward);
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
                    self.evaluate(command)?;
                }
            }
            Command::mOo => {
                self.memory_idx = self.memory_idx.saturating_sub(1);
            }
            Command::moO => {
                self.memory_idx = self.memory_idx.saturating_add(1);

                if self.memory_idx == self.memory.len() {
                    self.memory.push(0);
                }
            }
            Command::mOO => {
                let value = value!();

                if value == Command::mOO as u32 {
                    return Err(Error::RecursiveEval);
                }
                let Ok(executed_command) = Command::try_from(value) else {
                    return Err(Error::InvalidCommand);
                };

                self.evaluate(executed_command)?;
            }
            Command::Moo => {
                let value = &mut value!();

                if *value == 0 {
                    *value = self.input.input_char()? as u32;
                } else {
                    self.output
                        .output_char(char::from_u32(*value).ok_or(Error::UnwritableChar)?)?;
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
                            return Err(Error::EndlessJumpForward);
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
                self.output.output_int(value!())?;
            }
            Command::oom => {
                value!() = self.input.input_int()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    BeginlessJumpBackward,
    EndlessJumpForward,
    InvalidCommand,
    RecursiveEval,
    UnwritableChar,
    Io(std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeginlessJumpBackward => write!(f, "beginless jump backward"),
            Self::EndlessJumpForward => write!(f, "endless jump forward"),
            Self::InvalidCommand => write!(f, "invalid command"),
            Self::RecursiveEval => write!(f, "recursive evaluation"),
            Self::UnwritableChar => write!(f, "unwritable char"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
