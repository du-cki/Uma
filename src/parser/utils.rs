use std::collections::VecDeque;

use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub enum ErrorType {
    ExpectedToken,
    UnexpectedToken,
    DuplicateArgument,
}

#[derive(Debug)]
pub struct ParserError {
    pub r#type: ErrorType,
    pub token: Option<Token>,
    pub message: String,
}

impl ParserError {
    pub fn new(r#type: ErrorType, token: Option<Token>, message: impl ToString) -> ParserError {
        ParserError {
            r#type,
            token,
            message: message.to_string(),
        }
    }
}

pub trait Buffer<Item, Kind = TokenKind> {
    fn consume(&mut self) -> Item;

    fn get(&self, idx: usize) -> Option<&Item>;

    fn peek(&self) -> Option<Item>;

    fn try_expect(&mut self, kind: &Kind) -> Option<Item>;

    fn expect(&mut self, kind: Kind) -> Result<Item, ParserError>;
}

impl Buffer<Token> for VecDeque<Token> {
    fn consume(&mut self) -> Token {
        self.pop_front().expect("Unexpected end of tokens")
    }

    fn get(&self, idx: usize) -> Option<&Token> {
        self.get(idx)
    }

    fn peek(&self) -> Option<Token> {
        self.get(0).cloned()
    }

    fn try_expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if let Some(next_token) = self.peek() {
            if next_token.kind == *kind {
                return Some(self.consume());
            }
        }

        return None;
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        if let Some(next_token) = self.try_expect(&kind) {
            return Ok(next_token);
        }

        Err(ParserError {
            token: self.peek(),
            message: format!("Unexpected token encountered, expected: `{:#?}`", kind),
            r#type: ErrorType::ExpectedToken,
        })
    }
}
