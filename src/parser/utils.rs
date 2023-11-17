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

    // pub fn get_primary(&mut self) -> Option<Vec<Token>> {
    //     match self.consume() {
    //         Some(t) if t.token_type == TokenType::PareL => {
    //             let mut scope = 0;
    //             let mut tokens = Vec::new();

    //             while self.tokens.len() > 0 {
    //                 if let Some(t) = self.consume() {
    //                     if t.token_type == TokenType::PareL { scope += 1 }

    //                     if t.token_type == TokenType::PareR {
    //                         if scope > 0 { scope -= 1 } else { break }
    //                     }

    //                     tokens.push(t)
    //                 } else { panic!("Unmatched parenthesis, no closing parenthesis found.") };
    //             }

    //             Some(tokens)
    //         },
    //         Some(t) => vec![t].into(),
    //         _ => None,
    //     }
    // }
}
