use std::{io, sync::mpsc};

enum Value {
    Int(u32),
    Char(char),
}

pub struct InputTemp;

impl cowlang::Input for InputTemp {
    fn input_int(&mut self) -> io::Result<u32> {
        todo!()
    }

    fn input_char(&mut self) -> io::Result<char> {
        todo!()
    }
}

pub fn output() -> (OutputTx, OutputRx) {
    let (tx, rx) = mpsc::channel();
    (
        OutputTx { tx },
        OutputRx {
            buf: String::new(),
            buf_with_spaces: String::new(),
            rx,
        },
    )
}

pub struct OutputTx {
    tx: mpsc::Sender<Value>,
}

impl cowlang::Output for OutputTx {
    fn output_int(&mut self, int: u32) -> io::Result<()> {
        let _ = self.tx.send(Value::Int(int));
        Ok(())
    }

    fn output_char(&mut self, char: char) -> io::Result<()> {
        let _ = self.tx.send(Value::Char(char));
        Ok(())
    }
}

pub struct OutputRx {
    buf: String,
    buf_with_spaces: String,
    rx: mpsc::Receiver<Value>,
}

impl OutputRx {
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    pub fn as_str_with_spaces(&self) -> &str {
        &self.buf_with_spaces
    }

    pub fn tick(&mut self) {
        for value in self.rx.try_iter() {
            match value {
                Value::Int(int) => {
                    let mut s = int.to_string();
                    self.buf.push_str(&s);
                    s.push(' ');
                    self.buf_with_spaces.push_str(&s);
                }
                Value::Char(char) => {
                    self.buf.push(char);
                    self.buf_with_spaces.push(char);
                    self.buf_with_spaces.push(' ');
                }
            }
        }
    }
}
