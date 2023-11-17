use crate::lexer::{Token, TokenType};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>
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
        value: Expr,
        is_mut: bool
    },
    Func {
        arguments: Vec<Expr>,
        body: Block
    },
    If {
        cnt: (Expr, Block),
        else_block: Option<Block>
    }
}

#[derive(Debug, PartialEq)]
pub struct Block {
    stmts: Vec<Stmt>
}


impl Into<Expr> for Token {
    fn into(self) -> Expr {
        match self.token_type {
            TokenType::String => Expr::String(self.value.unwrap()),
            TokenType::Number => Expr::Number(self.value.unwrap()),
            TokenType::Float => Expr::Float(self.value.unwrap()),
            TokenType::Identifier => Expr::Identifier(self.value.unwrap()),
            other => unimplemented!("got unimplemented `Token` while converting `Token` to an `Expr`: {:#?}", other),
        }
    }
}

