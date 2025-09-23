use std::{io, sync::mpsc};

enum Value {
    Int(u32),
    Char(char),
}

enum ValueKind {
    Int,
    Char,
}

pub fn input() -> (InputTx, InputRx) {
    let (tx, rx) = mpsc::channel();
    (
        InputTx { tx },
        InputRx {
            rx,
            current_op: None,
        },
    )
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

pub struct InputTx {
    tx: mpsc::Sender<(ValueKind, oneshot::Sender<Value>)>,
}

impl cowlang::Input for InputTx {
    fn input_int(&mut self) -> io::Result<u32> {
        let (syn, ack) = oneshot::channel();
        let _ = self.tx.send((ValueKind::Int, syn));

        match ack.recv() {
            Ok(Value::Int(int)) => Ok(int),
            _ => unreachable!(),
        }
    }

    fn input_char(&mut self) -> io::Result<char> {
        let (syn, ack) = oneshot::channel();
        let _ = self.tx.send((ValueKind::Char, syn));

        match ack.recv() {
            Ok(Value::Char(char)) => Ok(char),
            _ => unreachable!(),
        }
    }
}

pub struct InputRx {
    rx: mpsc::Receiver<(ValueKind, oneshot::Sender<Value>)>,
    current_op: Option<(ValueKind, oneshot::Sender<Value>, String)>,
}

impl InputRx {
    pub fn current_op_buf(&self) -> Option<&str> {
        self.current_op.as_ref().map(|(_, _, b)| b.as_str())
    }

    pub fn has_current_op(&self) -> bool {
        self.current_op.is_some()
    }

    pub fn push_char_to_current_op(&mut self, char: char) {
        if let Some((_, _, buf)) = self.current_op.as_mut() {
            buf.push(char);
        }
    }

    pub fn finish_current_op(&mut self) {
        if let Some((value_kind, syn, buf)) = self.current_op.take() {
            match value_kind {
                ValueKind::Int => {
                    // TODO: Handle.
                    let int = buf.parse::<u32>().unwrap();
                    let _ = syn.send(Value::Int(int));
                }
                ValueKind::Char => {
                    // TODO: Handle.
                    let char = buf.chars().next().unwrap();
                    let _ = syn.send(Value::Char(char));
                }
            }
        }
    }

    pub fn tick(&mut self) {
        if self.current_op.is_none()
            && let Ok((value_kind, syn)) = self.rx.try_recv()
        {
            self.current_op = Some((value_kind, syn, String::new()))
        }
    }
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
