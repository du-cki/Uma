use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Box<Expr>>,
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
        value: Box<Expr>,
        is_mut: bool,
    },
    Func {
        name: String,
        args: Vec<String>,
        body: Block,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    If {
        cnt: (Expr, Block),
        else_block: Option<Block>,
    },
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
