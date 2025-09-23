use std::{borrow::Cow, ops::Deref, str::FromStr};

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Command {
    moo = 0,
    mOo = 1,
    moO = 2,
    mOO = 3,
    Moo = 4,
    MOo = 5,
    MoO = 6,
    MOO = 7,
    OOO = 8,
    MMM = 9,
    OOM = 10,
    oom = 11,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "moo" => Command::moo,
            "mOo" => Command::mOo,
            "moO" => Command::moO,
            "mOO" => Command::mOO,
            "Moo" => Command::Moo,
            "MOo" => Command::MOo,
            "MoO" => Command::MoO,
            "MOO" => Command::MOO,
            "OOO" => Command::OOO,
            "MMM" => Command::MMM,
            "OOM" => Command::OOM,
            "oom" => Command::oom,
            _ => return Err(()),
        })
    }
}

impl TryFrom<u32> for Command {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Command::moo,
            1 => Command::mOo,
            2 => Command::moO,
            3 => Command::mOO,
            4 => Command::Moo,
            5 => Command::MOo,
            6 => Command::MoO,
            7 => Command::MOO,
            8 => Command::OOO,
            9 => Command::MMM,
            10 => Command::OOM,
            11 => Command::oom,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Program<'a>(Cow<'a, [Command]>);

impl<'a> Program<'a> {
    pub fn new(commands: impl Into<Cow<'a, [Command]>>) -> Self {
        Self(commands.into())
    }

    pub fn parse(commands: &str) -> Self {
        Self::new(
            commands
                .split_whitespace()
                .filter_map(|s| Command::from_str(s).ok())
                .collect::<Vec<_>>(),
        )
    }
}

impl FromStr for Program<'_> {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program::parse(s))
    }
}

impl Deref for Program<'_> {
    type Target = [Command];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
