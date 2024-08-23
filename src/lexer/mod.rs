mod tokens;
mod utils;

pub use self::tokens::TokenType;

pub use self::utils::Buffer;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<String>,
}

fn match_keyword_to_token(keyword: &str) -> Option<Token> {
    let token = {
        match keyword {
            "let" => Some(TokenType::Let),
            "mut" => Some(TokenType::Mut),
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "func" => Some(TokenType::Func),
            "return" => Some(TokenType::Return),
            "true" => Some(TokenType::True),
            "false" => Some(TokenType::False),
            "none" => Some(TokenType::None),
            _ => None,
        }
    };

    Some(Token {
        token_type: token?,
        value: None,
    })
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

    fn parse_idx_or_kw(&mut self) -> Token {
        let mut out = String::new();

        while self.buffer.current.is_alphanumeric() || self.buffer.current == '_' {
            out.push(self.buffer.current);
            self.buffer.next();
        }

        match_keyword_to_token(&out).unwrap_or(Token {
            token_type: TokenType::Identifier,
            value: Some(out),
        })
    }

    fn parse_number(&mut self) -> Token {
        let mut out = String::new();

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
            Token {
                value: Some(out),
                token_type: TokenType::Float,
            }
        } else {
            Token {
                value: Some(out),
                token_type: TokenType::Number,
            }
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

        Token {
            value: Some(out),
            token_type: TokenType::String,
        }
    }

    fn parse_character(&mut self) -> Token {
        let token_type = {
            match self.buffer.current {
                ';' => TokenType::Semi,
                '=' => TokenType::Equals,
                '.' => TokenType::Dot,
                ',' => TokenType::Comma,
                '{' => TokenType::BraceL,
                '}' => TokenType::BraceR,
                '(' => TokenType::PareL,
                ')' => TokenType::PareR,
                '[' => TokenType::BracketL,
                ']' => TokenType::BracketR,
                '+' => TokenType::Add,
                '-' => TokenType::Sub,
                '/' => TokenType::Div,
                '*' => TokenType::Multi,
                '^' => TokenType::Expo,
                _ => panic!(
                    "Unexpected character `{}` found while parsing.",
                    self.buffer.current
                ),
            }
        };

        Token {
            value: None,
            token_type,
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.buffer.eof {
            let curr = self.buffer.current;

            match curr {
                'a'..='z' | 'A'..='Z' | '_' => tokens.push(self.parse_idx_or_kw()),
                '0'..='9' => tokens.push(self.parse_number()),
                '\'' | '"' => tokens.push(self.parse_string(curr)),
                c if c.is_whitespace() => {
                    self.buffer.next();
                }
                _ => {
                    tokens.push(self.parse_character());

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
                Token {
                    token_type: TokenType::Func,
                    value: None
                },
                Token {
                    token_type: TokenType::Identifier,
                    value: Some("main".to_string())
                },
                Token {
                    token_type: TokenType::PareL,
                    value: None,
                },
                Token {
                    token_type: TokenType::PareR,
                    value: None
                },
                Token {
                    token_type: TokenType::BraceL,
                    value: None
                },
                Token {
                    token_type: TokenType::Identifier,
                    value: Some("print".to_string())
                },
                Token {
                    token_type: TokenType::PareL,
                    value: None
                },
                Token {
                    token_type: TokenType::String,
                    value: Some("Hello, World!".to_string())
                },
                Token {
                    token_type: TokenType::PareR,
                    value: None
                },
                Token {
                    token_type: TokenType::Semi,
                    value: None
                },
                Token {
                    token_type: TokenType::BraceR,
                    value: None
                },
            ]
        )
    }

    #[test]
    fn lex_variable() {
        let lexed = Lexer::new(
            r#"
            let x = "Hello, World!";
        "#,
        )
        .lex();

        assert_eq!(
            lexed,
            vec![
                Token {
                    token_type: TokenType::Let,
                    value: None
                },
                Token {
                    token_type: TokenType::Identifier,
                    value: Some("x".to_string())
                },
                Token {
                    token_type: TokenType::Equals,
                    value: None
                },
                Token {
                    token_type: TokenType::String,
                    value: Some("Hello, World!".to_string())
                },
                Token {
                    token_type: TokenType::Semi,
                    value: None
                },
            ]
        )
    }

    #[test]
    fn num_parsing() {
        let mut parsed_int = Lexer::new("1_000_000;");

        assert_eq!(
            parsed_int.parse_number(),
            Token {
                token_type: TokenType::Number,
                value: Some("1000000".to_string())
            }
        );

        assert_eq!(
            // check if it consumes things that weren't an integer
            parsed_int.buffer.current,
            ';'
        )
    }

    #[test]
    fn float_parsing() {
        let parsed_float = Lexer::new("3.14156;").parse_number();

        assert_eq!(
            parsed_float,
            Token {
                token_type: TokenType::Float,
                value: Some("3.14156".to_string())
            }
        )
    }

    #[test]
    fn string_parsing() {
        let parsed = Lexer::new(r#"'Hello\n\\n,\'"" World!!'"#).parse_string('\'');

        assert_eq!(
            parsed,
            Token {
                token_type: TokenType::String,
                value: Some("Hello\n\\n,\'\"\" World!!".to_string())
            }
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
