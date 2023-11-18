use std::str::Chars;

pub struct Buffer<'a> {
    pub data: Chars<'a>,

    pub eof: bool,
    pub current: char,
    pub ln: usize,
    pub idx: usize
}

impl<'a> Buffer<'a> {
    pub fn new(raw_data: &'a str) -> Buffer<'a> {
        let mut data = raw_data.chars();
        let current = data.next().unwrap_or('\0');

        let eof = {
            if current == '\0' {
                true
            } else {
                false
            }
        };

        Buffer {
            data,
            eof,
            current,
            ln: 0,
            idx: 0,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let c = match self.data.next() {
            None => {
                self.eof = true;
                return None;
            },
            Some(c) => c
        };

        if c == '\n' {
            self.ln += 1;
        }

        self.idx += 1;
        self.current = c;

        Some(c)
    }
}
