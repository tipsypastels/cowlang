use std::io;

pub trait Input {
    fn input_int(&mut self) -> io::Result<u32>;
    fn input_char(&mut self) -> io::Result<char>;
}

pub trait Output {
    fn output_int(&mut self, int: u32) -> io::Result<()>;
    fn output_char(&mut self, char: char) -> io::Result<()>;
}
