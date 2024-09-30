mod types;
mod utils;

use std::collections::VecDeque;

use types::Block;

use crate::lexer::{Token, TokenKind};
use crate::mapping;

use self::types::{Expr, Stmt};
use self::utils::Buffer;

pub struct Parser {
    tokens: VecDeque<Token>,
}

#[allow(unused)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into(),
        }
    }

    fn primary(&mut self) -> Stmt {
        let token = self.tokens.consume();

        match token.kind {
            TokenKind::String | TokenKind::Number | TokenKind::Float => token.into(),
            TokenKind::Identifier => {
                if let Some(peeked) = self.tokens.peek() {
                    if peeked.kind == TokenKind::PareL {
                        return self.call(token.value.clone().unwrap());
                    }
                }

                token.into()
            }
            TokenKind::PareL => {
                let expr = self.expr();
                self.tokens.expect(TokenKind::PareR);

                expr
            }
            token => panic!("Unexpected token: {:?}", token),
        }
    }

    fn binary(&mut self, mut lhs: Stmt, precedence: i8) -> Stmt {
        while let Some(token) = self.tokens.peek() {
            if token.kind.precedence() < precedence {
                return lhs;
            }

            let op = self.tokens.consume();

            let mut rhs = self.primary();
            if let Some(token) = self.tokens.peek() {
                if op.kind.precedence() < token.kind.precedence() {
                    rhs = self.binary(rhs, op.kind.precedence() + 1);
                }
            }

            lhs = Expr::Binary {
                lhs: lhs.into(),
                op,
                rhs: rhs.into(),
            }
            .into();
        }

        lhs
    }

    fn expr(&mut self) -> Stmt {
        let mut primary = self.primary();
        let expr = self.binary(primary, 0);

        expr
    }

    fn block(&mut self) -> Block {
        self.tokens.expect(TokenKind::BraceL);

        let mut stmts = Vec::<Stmt>::new();

        while let Some(token) = self.tokens.peek().cloned() {
            if token.kind == TokenKind::BraceR {
                break;
            }

            stmts.push(self.stmt(token));
        }

        self.tokens.expect(TokenKind::BraceR);

        Block { stmts }
    }

    fn stmt(&mut self, token: Token) -> Stmt {
        let stmt = match token.kind {
            TokenKind::Let => self.variable(),
            TokenKind::Func => self.function(),
            TokenKind::Return => self.return_(),
            TokenKind::Semi => Stmt::Empty,
            _ => self.expr(),
        };

        stmt
    }

    fn return_(&mut self) -> Stmt {
        self.tokens.expect(TokenKind::Return);
        let expr = self.expr();

        self.tokens.try_expect(&TokenKind::Semi);

        Stmt::Return(expr.into())
    }

    fn variable(&mut self) -> Stmt {
        self.tokens.expect(TokenKind::Let);

        let is_mut = self.tokens.try_expect(&TokenKind::Mut).is_some();
        let name = self.tokens.expect(TokenKind::Identifier).value.unwrap();

        self.tokens.expect(TokenKind::Equals);
        let value = self.expr();
        self.tokens.try_expect(&TokenKind::Semi);

        Stmt::Var {
            name,
            value: value.into(),
            is_mut,
        }
    }

    fn call(&mut self, name: String) -> Stmt {
        self.tokens.expect(TokenKind::PareL);

        if let Some(token) = self.tokens.try_expect(&TokenKind::PareR) {
            return Stmt::Call { name, args: vec![] };
        }

        let mut args = Vec::new();

        loop {
            let arg = self.expr();
            args.push(arg.into());

            if let Some(token) = self.tokens.try_expect(&TokenKind::PareR) {
                break;
            }

            self.tokens.expect(TokenKind::Comma);
        }

        self.tokens.try_expect(&TokenKind::Semi);

        Stmt::Call { name, args }
    }

    fn function(&mut self) -> Stmt {
        self.tokens.expect(TokenKind::Func);

        let name = self.tokens.expect(TokenKind::Identifier).value.unwrap();

        self.tokens.expect(TokenKind::PareL);

        let mut args = mapping!();

        while let Some(arg) = self.tokens.try_expect(&TokenKind::Identifier) {
            let name = arg.value.unwrap();

            if args.contains_key(&name) {
                panic!("Duplicate argument: {}", name);
            }

            let r#type = {
                if let Some(token) = self.tokens.try_expect(&TokenKind::Colon) {
                    Some(self.tokens.expect(TokenKind::Identifier).value.unwrap())
                } else {
                    None
                }
            };

            args.insert(name, r#type);

            if self.tokens.try_expect(&TokenKind::PareR).is_some() {
                break;
            }

            self.tokens.expect(TokenKind::Comma);
        }

        if args.is_empty() {
            self.tokens.expect(TokenKind::PareR);
        }

        let body = self.block();
        self.tokens.try_expect(&TokenKind::Semi);

        Stmt::Func { name, args, body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::Lexer;
    use crate::mapping;

    #[test]
    fn mut_variable() {
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
                value: Expr::String(String::from("bar")).into(),
                is_mut: true
            }
        )
    }

    #[test]
    fn basic_arithmetic() {
        let tokens = Lexer::new(
            r#"
            let foo = 9 + 10 * round(3.14);
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).variable(),
            Stmt::Var {
                name: String::from("foo"),
                value: Expr::Binary {
                    lhs: Expr::Number(String::from("9")).into(),
                    op: Token::new(TokenKind::Add, None, 1, 25),
                    rhs: Expr::Binary {
                        lhs: Expr::Number(String::from("10")).into(),
                        op: Token::new(TokenKind::Multi, None, 1, 30),
                        rhs: Stmt::Call {
                            name: String::from("round"),
                            args: vec![Expr::Float(String::from("3.14")).into()]
                        }
                        .into()
                    }
                    .into()
                }
                .into(),
                is_mut: false
            }
        )
    }

    #[test]
    fn mut_variable_with_nested_arithmetic() {
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
                value: Expr::Binary {
                    lhs: Expr::Binary {
                        lhs: Expr::Number(String::from("4")).into(),
                        op: Token::new(TokenKind::Add, None, 1, 28),
                        rhs: Expr::Number(String::from("5")).into()
                    }
                    .into(),
                    op: Token::new(TokenKind::Add, None, 1, 33),
                    rhs: Expr::Binary {
                        lhs: Expr::Number(String::from("10")).into(),
                        op: Token::new(TokenKind::Multi, None, 1, 38),
                        rhs: Expr::Number(String::from("3")).into()
                    }
                    .into()
                }
                .into(),
                is_mut: true
            }
        )
    }

    #[test]
    fn function_body() {
        let tokens = Lexer::new(
            r#"
            func main() {
                print("Hello, World!");
            }
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).function(),
            Stmt::Func {
                name: String::from("main"),
                args: mapping!(),
                body: Block {
                    stmts: vec![Stmt::Call {
                        name: String::from("print"),
                        args: vec![Expr::String(String::from("Hello, World!")).into()]
                    }
                    .into()]
                }
            }
        )
    }

    #[test]
    fn function_args() {
        let tokens = Lexer::new(
            r#"
            func sum(x: Int, y: Int) {
                return x + y
            }
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).function(),
            Stmt::Func {
                name: String::from("sum"),
                args: mapping!(
                    String::from("x") => Some(String::from("Int")),
                    String::from("y") => Some(String::from("Int"))
                ),
                body: Block {
                    stmts: vec![Stmt::Return(
                        Expr::Binary {
                            lhs: Expr::Identifier(String::from("x")).into(),
                            op: Token::new(TokenKind::Add, None, 2, 26),
                            rhs: Expr::Identifier(String::from("y")).into()
                        }
                        .into()
                    )]
                }
            }
        )
    }
}
