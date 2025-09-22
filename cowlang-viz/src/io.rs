use std::{fmt::Display, io, sync::mpsc};

pub fn writer() -> (WriterTx, WriterRx) {
    let (tx, rx) = mpsc::channel();
    (
        WriterTx { tx },
        WriterRx {
            out: Vec::new(),
            rx,
        },
    )
}

pub struct WriterTx {
    tx: mpsc::Sender<u8>,
}

impl io::Write for WriterTx {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &byte in buf {
            let _ = self.tx.send(byte);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct WriterRx {
    out: Vec<u8>,
    rx: mpsc::Receiver<u8>,
}

impl WriterRx {
    pub fn display(&self) -> impl Display {
        enum DisplayImpl<'a> {
            Str(&'a str),
            Bytes(&'a [u8]),
        }

        impl Display for DisplayImpl<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Str(s) => write!(f, "{s}"),
                    Self::Bytes(b) => write!(f, "{b:?}"),
                }
            }
        }

        match std::str::from_utf8(&self.out) {
            Ok(s) => DisplayImpl::Str(s),
            Err(_) => DisplayImpl::Bytes(&self.out),
        }
    }

    pub fn tick(&mut self) {
        for byte in self.rx.try_iter() {
            self.out.push(byte);
        }
    }
}
