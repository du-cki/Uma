mod types;
mod utils;

use std::collections::VecDeque;

use crate::lexer::{Token, TokenKind};

use self::types::{Expr, Stmt};
use self::utils::Buffer;

pub struct Parser {
    buffer: Buffer,
}

#[allow(unused)]
impl Parser {
    pub fn new(buffer: Vec<Token>) -> Parser {
        Parser {
            buffer: Buffer::new(buffer),
        }
    }

    fn primary(&mut self) -> Box<Expr> {
        let token = self.buffer.consume();

        match token.kind {
            TokenKind::String | TokenKind::Number | TokenKind::Float | TokenKind::Identifier => {
                Box::new(token.into())
            }
            token => unimplemented!(),
        }
    }

    fn parse_expr(&mut self, mut lhs: Box<Expr>, precedence: i8) -> Box<Expr> {
        while let Some(token) = self.buffer.peek() {
            if token.kind.precedence() < precedence {
                return lhs;
            }

            let op = self.buffer.consume();

            let mut rhs = self.primary();
            if let Some(token) = self.buffer.peek() {
                if token.kind.precedence() < op.kind.precedence() {
                    rhs = self.parse_expr(rhs, op.kind.precedence() + 1);
                }
            }

            lhs = Box::new(Expr::Binary { lhs, op, rhs });
        }

        lhs
    }

    fn expr(&mut self) -> Box<Expr> {
        let mut primary = self.primary();
        let expr = self.parse_expr(primary, 0);

        expr
    }

    fn variable(&mut self) -> Stmt {
        self.buffer.expect(TokenKind::Let);

        let is_mut = self.buffer.try_expect(&TokenKind::Mut).is_some();
        let name = self.buffer.expect(TokenKind::Identifier).value.unwrap();

        self.buffer.expect(TokenKind::Equals);
        let value = self.expr();
        self.buffer.expect(TokenKind::Semi);

        Stmt::Var {
            name,
            value,
            is_mut,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut program: Vec<Stmt> = Vec::new();

        while let Some(token) = self.buffer.peek() {
            match token.kind {
                TokenKind::Let => program.push(self.variable()),
                _ => unimplemented!(),
            }
        }

        program
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn parse_mut_variable() {
        let tokens = Lexer::new(
            r#"
            let mut foo = "bar";
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).variable(),
            Stmt::Var {
                name: String::from("foo"),
                value: Box::new(Expr::String("bar".to_string())),
                is_mut: true
            }
        )
    }

    #[test]
    fn parse_basic_arithmetic() {
        let tokens = Lexer::new(
            r#"
            let foo = 9 + 10;
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).variable(),
            Stmt::Var {
                name: String::from("foo"),
                value: Box::new(Expr::Binary {
                    lhs: Box::new(Expr::Number("9".to_string())),
                    op: Token {
                        kind: TokenKind::Add,
                        value: None
                    },
                    rhs: Box::new(Expr::Number("10".to_string()))
                }),
                is_mut: false
            }
        )
    }

    #[test]
    fn parse_mut_variable_with_nested_arithmetic() {
        //        +
        //       / \
        //      +   \
        //     / \   *
        //    4   5 / \
        //         10  3
        let tokens = Lexer::new(
            r#"
            let mut x = (4 + 5) + 10 * 3;
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).variable(),
            Stmt::Var {
                name: String::from("x"),
                value: Box::new(Expr::Binary {
                    lhs: Box::new(Expr::Binary {
                        lhs: Box::new(Expr::Number("4".to_string())),
                        op: Token {
                            kind: TokenKind::Add,
                            value: None
                        },
                        rhs: Box::new(Expr::Number("5".to_string()))
                    }),
                    op: Token {
                        kind: TokenKind::Add,
                        value: None
                    },
                    rhs: Box::new(Expr::Binary {
                        lhs: Box::new(Expr::Number("10".to_string())),
                        op: Token {
                            kind: TokenKind::Multi,
                            value: None
                        },
                        rhs: Box::new(Expr::Number("3".to_string()))
                    })
                }),
                is_mut: true
            }
        )
    }
}
