use std::collections::VecDeque;

use crate::lexer::{Token, TokenKind};

// don't really like redefining another Buffer, but I'll write a generic one whenever.
pub(crate) struct Buffer {
    pub tokens: VecDeque<Token>,
}

impl Buffer {
    pub fn new(tokens: Vec<Token>) -> Buffer {
        Buffer {
            tokens: VecDeque::from(tokens),
        }
    }

    pub fn consume(&mut self) -> Token {
        self.tokens.pop_front().expect("Unexpected end of tokens")
    }

    pub fn get(&self, idx: usize) -> Option<&Token> {
        self.tokens.get(idx)
    }

    pub fn peek(&self) -> Option<&Token> {
        self.get(0)
    }

    pub fn try_expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if let Some(next_token) = self.peek() {
            if next_token.kind == *kind {
                return Some(self.consume());
            }
        }

        return None;
    }

    pub fn expect(&mut self, kind: TokenKind) -> Token {
        let n = self.try_expect(&kind);
        if let Some(next_token) = n {
            return next_token;
        }

        // TODO: better error handling
        panic!("Unexpected token encountered, expected: `{kind:#?}` but got: {n:#?}");
    }
}
