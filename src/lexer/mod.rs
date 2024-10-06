mod tokens;
mod utils;

pub use self::tokens::{Token, TokenKind};
pub use self::utils::Buffer;

fn match_keyword_to_token(keyword: &str, line: usize, column: usize) -> Option<Token> {
    let token = {
        match keyword {
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "func" => Some(TokenKind::Func),
            "return" => Some(TokenKind::Return),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "none" => Some(TokenKind::None),
            _ => None,
        }
    };

    token.map(|kind| Token::new(kind, None, line, column))
}

pub struct Lexer<'a> {
    buffer: Buffer<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(cnt: &'a str) -> Lexer<'a> {
        Lexer {
            buffer: Buffer::new(cnt),
        }
    }

    fn parse_identifier_or_keyword(&mut self) -> Token {
        let mut out = String::new();

        let column = self.buffer.column;

        while self.buffer.current.is_alphanumeric() || self.buffer.current == '_' {
            out.push(self.buffer.current);
            self.buffer.next();
        }

        match_keyword_to_token(&out, self.buffer.line, column).unwrap_or(Token::new(
            TokenKind::Identifier,
            Some(out),
            self.buffer.line,
            column,
        ))
    }

    fn parse_number(&mut self) -> Token {
        let mut out = String::new();
        let column = self.buffer.column;

        while self.buffer.current.is_ascii_digit()
            || self.buffer.current == '_'
            || self.buffer.current == '.'
        {
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
            Token::new(TokenKind::Float, Some(out), self.buffer.line, column)
        } else {
            Token::new(TokenKind::Number, Some(out), self.buffer.line, column)
        }
    }

    fn parse_string(&mut self, delimeter: char) -> Token {
        let mut out = String::new();
        self.buffer.next();

        let column = self.buffer.column - 1;

        while self.buffer.current != delimeter {
            if self.buffer.current == '\\' {
                self.buffer.next();

                let curr = match self.buffer.current {
                    'n' => '\n',
                    't' => '\t',
                    '\\' => '\\',
                    other => other,
                };

                out.push(curr);
                self.buffer.next();

                continue;
            }

            out.push(self.buffer.current);
            self.buffer.next();
        }

        self.buffer.next();

        Token::new(TokenKind::String, Some(out), self.buffer.line, column)
    }

    fn parse_character(&mut self) -> Token {
        let kind = {
            match self.buffer.current {
                ':' => TokenKind::Colon,
                ';' => TokenKind::Semi,
                '=' => TokenKind::Equals,
                '.' => TokenKind::Dot,
                ',' => TokenKind::Comma,
                '{' => TokenKind::BraceL,
                '}' => TokenKind::BraceR,
                '(' => TokenKind::PareL,
                ')' => TokenKind::PareR,
                '[' => TokenKind::BracketL,
                ']' => TokenKind::BracketR,
                '+' => TokenKind::Add,
                '-' => TokenKind::Sub,
                '/' => TokenKind::Div,
                '*' => TokenKind::Multi,
                '^' => TokenKind::Expo,
                _ => panic!(
                    "Unexpected character `{}` found while parsing.",
                    self.buffer.current
                ),
            }
        };

        Token::new(kind, None, self.buffer.line, self.buffer.column)
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.buffer.eof {
            let curr = self.buffer.current;

            let token = match curr {
                'a'..='z' | 'A'..='Z' | '_' => self.parse_identifier_or_keyword(),
                '0'..='9' => self.parse_number(),
                '\'' | '"' => self.parse_string(curr),
                c if c.is_whitespace() => {
                    self.buffer.next();
                    continue;
                }
                _ => {
                    let token = self.parse_character();
                    self.buffer.next();

                    token
                }
            };

            tokens.push(token);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function() {
        let lexed = Lexer::new(
            r#"
            func main() {
                print("Hello, World!");
            }
        "#,
        )
        .lex();

        assert_eq!(
            lexed,
            vec![
                Token::new(TokenKind::Func, None, 2, 13),
                Token::new(TokenKind::Identifier, Some("main".to_string()), 2, 18),
                Token::new(TokenKind::PareL, None, 2, 22),
                Token::new(TokenKind::PareR, None, 2, 23),
                Token::new(TokenKind::BraceL, None, 2, 25),
                Token::new(TokenKind::Identifier, Some("print".to_string()), 3, 17),
                Token::new(TokenKind::PareL, None, 3, 22),
                Token::new(TokenKind::String, Some("Hello, World!".to_string()), 3, 23),
                Token::new(TokenKind::PareR, None, 3, 38),
                Token::new(TokenKind::Semi, None, 3, 39),
                Token::new(TokenKind::BraceR, None, 4, 13),
            ]
        )
    }

    #[test]
    fn variable() {
        let lexed = Lexer::new(
            r#"
            let x = "Hello, World!";
        "#,
        )
        .lex();

        assert_eq!(
            lexed,
            vec![
                Token::new(TokenKind::Let, None, 2, 13),
                Token::new(TokenKind::Identifier, Some("x".to_string()), 2, 17),
                Token::new(TokenKind::Equals, None, 2, 19),
                Token::new(TokenKind::String, Some("Hello, World!".to_string()), 2, 21),
                Token::new(TokenKind::Semi, None, 2, 36),
            ]
        )
    }

    #[test]
    fn num_parsing() {
        let mut parsed_int = Lexer::new("1_000_000;");

        assert_eq!(
            parsed_int.parse_number(),
            Token::new(TokenKind::Number, Some("1000000".to_string()), 1, 0)
        );

        // check if it consumes things that weren't an integer
        assert_eq!(parsed_int.buffer.current, ';')
    }

    #[test]
    fn float_parsing() {
        let parsed_float = Lexer::new("3.14156;").parse_number();

        assert_eq!(
            parsed_float,
            Token::new(TokenKind::Float, Some("3.14156".to_string()), 1, 0)
        )
    }

    #[test]
    fn string_parsing() {
        let parsed = Lexer::new(r#"'Hello\n\\n,\'"" World!!'"#).parse_string('\'');

        assert_eq!(
            parsed,
            Token::new(
                TokenKind::String,
                Some("Hello\n\\n,\'\"\" World!!".to_string()),
                1,
                0
            )
        )
    }

    #[test]
    fn test_buffer() {
        let mut buffer = Buffer::new("Lot");

        assert_eq!(buffer.eof, false);
        assert_eq!(buffer.current, 'L');
        assert_eq!(buffer.next(), Some('o'));
        assert_eq!(buffer.next(), Some('t'));
        assert_eq!(buffer.current, 't');
        assert_eq!(buffer.next(), None);
        assert_eq!(buffer.eof, true)
    }
}
