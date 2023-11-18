use std::collections::VecDeque;

use crate::lexer::{Token, TokenType};
use super::types::Expr;

// don't really like redefining another Buffer, but I'll write a generic one whenever.
pub(crate) struct Buffer {
    pub tokens: VecDeque<Token>
}

impl Buffer {
    pub fn new(tokens: Vec<Token>) -> Buffer {
        Buffer {
            tokens: VecDeque::from(tokens)
        }
    }

    pub fn consume(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    pub fn get(&self, idx: usize) -> Option<&Token> {
        self.tokens.get(idx)
    }

    pub fn peek(&self) -> Option<&Token> {
        self.get(0)
    }
}
