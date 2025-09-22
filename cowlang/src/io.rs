use std::io;

pub enum MaybeDynRead<'a> {
    Stdin(io::Stdin),
    Dyn(&'a mut dyn io::Read),
}

impl Default for MaybeDynRead<'_> {
    fn default() -> Self {
        Self::Stdin(io::stdin())
    }
}

impl io::Read for MaybeDynRead<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(stdin) => stdin.read(buf),
            Self::Dyn(read) => read.read(buf),
        }
    }
}

pub enum MaybeDynWrite<'a> {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    Dyn(&'a mut dyn io::Write),
}

impl MaybeDynWrite<'_> {
    pub fn default_stderr() -> Self {
        Self::Stderr(io::stderr())
    }
}

impl Default for MaybeDynWrite<'_> {
    fn default() -> Self {
        Self::Stdout(io::stdout())
    }
}

impl io::Write for MaybeDynWrite<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(stdout) => stdout.write(buf),
            Self::Stderr(stderr) => stderr.write(buf),
            Self::Dyn(write) => write.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout(stdout) => stdout.flush(),
            Self::Stderr(stderr) => stderr.flush(),
            Self::Dyn(write) => write.flush(),
        }
    }
}
