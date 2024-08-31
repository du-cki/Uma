use std::collections::HashMap;

use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        lhs: Box<Stmt>,
        op: Token,
        rhs: Box<Stmt>,
    },
    Identifier(String),
    Number(String),
    Float(String),
    String(std::string::String),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Var {
        name: String,
        value: Box<Stmt>,
        is_mut: bool,
    },
    Func {
        name: String,
        args: HashMap<String, Option<String>>,
        body: Block,
    },
    Call {
        name: String,
        args: Vec<Box<Stmt>>,
    },
    Return(Box<Stmt>),
    Expr(Expr),
    Empty,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Into<Expr> for Token {
    fn into(self) -> Expr {
        match self.kind {
            TokenKind::String => Expr::String(self.value.unwrap()),
            TokenKind::Number => Expr::Number(self.value.unwrap()),
            TokenKind::Float => Expr::Float(self.value.unwrap()),
            TokenKind::Identifier => Expr::Identifier(self.value.unwrap()),
            ref other => panic!("cannot convert `{:#?}` to an `Expr`: {:#?}", self, other),
        }
    }
}

impl Into<Stmt> for Token {
    fn into(self) -> Stmt {
        Stmt::Expr(self.into())
    }
}

impl Into<Stmt> for Expr {
    fn into(self) -> Stmt {
        Stmt::Expr(self)
    }
}

impl Into<Box<Stmt>> for Expr {
    fn into(self) -> Box<Stmt> {
        Box::new(self.into())
    }
}
