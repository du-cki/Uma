use std::collections::VecDeque;

use crate::lexer::{Token, TokenKind};

pub trait Buffer<Item, Kind = TokenKind> {
    fn consume(&mut self) -> Item;

    fn get(&self, idx: usize) -> Option<&Item>;

    fn peek(&self) -> Option<&Item>;

    fn try_expect(&mut self, kind: &Kind) -> Option<Item>;

    fn expect(&mut self, kind: Kind) -> Item;
}

impl Buffer<Token> for VecDeque<Token> {
    fn consume(&mut self) -> Token {
        self.pop_front().expect("Unexpected end of tokens")
    }

    fn get(&self, idx: usize) -> Option<&Token> {
        self.get(idx)
    }

    fn peek(&self) -> Option<&Token> {
        self.get(0)
    }

    fn try_expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if let Some(next_token) = self.peek() {
            if next_token.kind == *kind {
                return Some(self.consume());
            }
        }

        return None;
    }

    fn expect(&mut self, kind: TokenKind) -> Token {
        if let Some(next_token) = self.try_expect(&kind) {
            return next_token;
        }

        // TODO: better error handling
        panic!("Unexpected token encountered, expected: `{kind:#?}`");
    }
}
