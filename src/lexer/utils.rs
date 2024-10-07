use std::{iter::Peekable, str::Chars};

#[derive(Debug)]
pub struct Buffer<'a> {
    pub data: Peekable<Chars<'a>>,
    pub eof: bool,
    pub current: char,
    pub line: usize,
    pub column: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(raw: &'a str) -> Buffer<'a> {
        let mut data = raw.chars().peekable();
        let current = data.next().unwrap_or('\0');

        Buffer {
            data,
            eof: current == '\0',
            current,
            line: 1 + (current == '\n') as usize,
            column: 0,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let c = match self.data.next() {
            None => {
                self.eof = true;
                return None;
            }
            Some(c) => c,
        };

        if c == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        self.current = c;

        Some(c)
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.data.peek()
    }
}
