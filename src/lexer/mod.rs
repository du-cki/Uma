mod tokens;

use std::{str::Chars, iter::Peekable};

use self::tokens::Token;

struct Buffer<'a> {
    data: Peekable<Chars<'a>>,

    eof: bool,
    current: char,
    // count: usize,
    ln: usize,
    idx: usize
}

impl<'a> Buffer<'a> {
    fn new(raw_data: &'a str) -> Buffer<'a> {
        let mut data = raw_data.chars().peekable();
        let current = data.next().unwrap_or('\0');

        Buffer {
            data,
            // count,
            eof: false,
            current,
            ln: 0,
            idx: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
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

    fn peek(&mut self) -> Option<char> {
        self.data.peek().copied()
    }
}

struct Lexer<'a> {
    buffer: Buffer<'a>
}

fn match_keyword_to_token(thing: &str) -> Option<Token> {
    match thing {
        "let" => Some(Token::Let),
        "false" | "true" => Some(Token::Bool(thing.to_string())),
        "none" => Some(Token::None),
        "if" => Some(Token::If),
        "else" => Some(Token::Else),
        "func" => Some(Token::Func),
        "return" => Some(Token::Return),
        _ => None
    }
}

impl<'a> Lexer<'a> {
    pub fn new(cnt: &'a str) -> Lexer<'a> {
        Lexer {
            buffer: Buffer::new(cnt)
        }
    }

    fn parse_idx_or_kw(&mut self) -> Token {
        let mut out = String::new();

        while self.buffer.current.is_alphanumeric() || self.buffer.current == '_' {
            out.push(self.buffer.current);
            self.buffer.next();
        }

        match_keyword_to_token(&out)
            .unwrap_or(Token::Identifier(out))
    }

    fn parse_number(&mut self) -> Token {
        let mut out = String::new();

        while self.buffer.current.is_numeric() || self.buffer.current == '_' || self.buffer.current == '.' {
            let curr = self.buffer.current;

            if curr == '_' {
                self.buffer.next();
                continue;
            }

            if curr == '.' {
                if out.contains('.') {
                    panic!("More than one decimal point found.");
                }
            }

            out.push(curr);
            self.buffer.next();
        }

        if out.contains('.') {
            Token::Float(out)
        } else {
            Token::Number(out)
        }
    }

    fn parse_string(&mut self, delimeter: char) -> Token {
        let mut out = String::new();
        self.buffer.next();

        while self.buffer.current != delimeter {
            if self.buffer.current == '\\' {
                self.buffer.next();

                let curr = match self.buffer.current {
                    'n' => '\n',
                    't' => '\t',
                    '\\' => '\\',
                    other => other
                };

                out.push(curr);
                self.buffer.next();

                continue;
            }

            out.push(self.buffer.current);
            self.buffer.next();
        }

        self.buffer.next();
        Token::String(out)
    }

    fn parse_character(&mut self) -> Token {
        match self.buffer.current {
            ';' => Token::Semi,
            '=' => Token::Equals,
            '.' => Token::Dot,
            ',' => Token::Comma,
            '{' => Token::BraceL,
            '}' => Token::BraceR,
            '(' => Token::PareL,
            ')' => Token::PareR,
            '[' => Token::BracketL,
            ']' => Token::BracketR,
            '+' => Token::Add,
            '-' => Token::Sub,
            '/' => Token::Div,
            '*' => Token::Multi,
            _ => panic!("Unexpected character `{}` found while parsing.", self.buffer.current)
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.buffer.eof {
            let curr = self.buffer.current;

            match curr {
                'a'..='z' | 'A'..='Z' | '_' => tokens.push(
                    self.parse_idx_or_kw()
                ),
                '0'..='9' => tokens.push(
                    self.parse_number()
                ),
                '\'' | '"' => tokens.push(
                    self.parse_string(curr)
                ),
                c if c.is_whitespace() => {
                    self.buffer.next();
                },
                _ => {
                    tokens.push(
                        self.parse_character()
                    );

                    self.buffer.next();
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_function() {
        let lexed = Lexer::new(r#"
            func name_of_function(argument1, argument2) {
                argument1 + argument2
            }
        "#).lex();

        assert_eq!(
            lexed,
            vec![
                Token::Func,
                Token::Identifier("name_of_function".to_string()),
                Token::PareL,
                Token::Identifier("argument1".to_string()),
                Token::Comma,
                Token::Identifier("argument2".to_string()),
                Token::PareR,
                Token::BraceL,
                Token::Identifier("argument1".to_string()),
                Token::Add,
                Token::Identifier("argument2".to_string()),
                Token::BraceR,
            ]
        )
    }


    #[test]
    fn lex_variable() {
        let lexed = Lexer::new(r#"
            let x = "Hello, World!";
        "#).lex();

        assert_eq!(
            lexed,
            vec![
                Token::Let,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::String("Hello, World!".to_string()),
                Token::Semi
            ]
        )
    }

    #[test]
    fn test_num_parsing() {
        let mut parsed_int = Lexer::new("1_000_000;");

        assert_eq!(
            parsed_int.parse_number(),
            Token::Number("1000000".to_string())
        );

        assert_eq!( // check if it consumes things that weren't an integer
            parsed_int.buffer.current,
            ';'
        );
    }

    #[test]
    fn test_float_parsing() {
        let parsed_float = Lexer::new("3.14156;").parse_number();

        assert_eq!(
            parsed_float,
            Token::Float("3.14156".to_string())
        )
    }

    #[test]
    fn test_string_parsing() {
        let parsed = Lexer::new(r#"'Hello\n\\n,\'"" World!!'"#).parse_string('\'');

        assert_eq!(
            parsed,
            Token::String("Hello\n\\n,\'\"\" World!!".to_string())
        )
    }

    #[test]
    fn test_buffer() {
        let mut buffer = Buffer::new("Lorem Epsium!");

        assert_eq!(buffer.current, 'L');
        assert_eq!(buffer.next(), Some('o'));
        assert_eq!(buffer.peek(), Some('r'));
        assert_eq!(buffer.next(), Some('r'));
        assert_eq!(buffer.current, 'r');
    }
}
