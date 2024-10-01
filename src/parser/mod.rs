mod types;
mod utils;

use std::collections::VecDeque;

use types::Block;
pub use utils::{ErrorType, ParserError};

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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut stmts = Vec::<Stmt>::new();

        while let Some(token) = self.tokens.peek() {
            stmts.push(self.stmt(token)?);
        }

        Ok(stmts)
    }

    fn primary(&mut self) -> Result<Stmt, ParserError> {
        let token = self.tokens.consume();

        match token.clone().kind {
            TokenKind::String | TokenKind::Number | TokenKind::Float => Ok(token.into()),
            TokenKind::Identifier => {
                if let Some(peeked) = self.tokens.peek() {
                    if peeked.kind == TokenKind::PareL {
                        return self.call(token.value.unwrap());
                    }
                }

                Ok(token.into())
            }
            TokenKind::PareL => {
                let expr = self.expr()?;
                self.tokens.expect(TokenKind::PareR)?;

                Ok(expr)
            }
            kind => Err(ParserError::new(
                ErrorType::UnexpectedToken,
                token,
                format!("Unexpected token occurred `{:#?}`", kind),
            )),
        }
    }

    fn binary(&mut self, mut lhs: Stmt, precedence: i8) -> Result<Stmt, ParserError> {
        while let Some(token) = self.tokens.peek() {
            if token.kind.precedence() < precedence {
                return Ok(lhs);
            }

            let op = self.tokens.consume();

            let mut rhs = self.primary()?;

            if let Some(token) = self.tokens.peek() {
                if op.kind.precedence() < token.kind.precedence() {
                    rhs = self.binary(rhs, op.kind.precedence() + 1)?;
                }
            }

            lhs = Expr::Binary {
                lhs: lhs.into(),
                op,
                rhs: rhs.into(),
            }
            .into();
        }

        Ok(lhs)
    }

    fn expr(&mut self) -> Result<Stmt, ParserError> {
        let mut primary = self.primary()?;
        let expr = self.binary(primary, 0)?;

        Ok(expr)
    }

    fn block(&mut self) -> Result<Block, ParserError> {
        self.tokens.expect(TokenKind::BraceL)?;

        let mut stmts = Vec::<Stmt>::new();

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::BraceR {
                break;
            }

            stmts.push(self.stmt(token)?);
        }

        self.tokens.expect(TokenKind::BraceR)?;
        Ok(Block { stmts })
    }

    fn stmt(&mut self, token: Token) -> Result<Stmt, ParserError> {
        let stmt = match token.kind {
            TokenKind::Func => self.function(),
            TokenKind::Let => self.variable(),
            TokenKind::Return => self.return_(),
            TokenKind::Semi => Ok(Stmt::Empty),
            _ => self.expr(),
        };

        stmt
    }

    fn return_(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::Return)?;
        let expr = self.expr()?;

        self.tokens.try_expect(&TokenKind::Semi);

        Ok(Stmt::Return(expr.into()))
    }

    fn variable(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::Let)?;

        let is_mut = self.tokens.try_expect(&TokenKind::Mut).is_some();
        let name = self.tokens.expect(TokenKind::Identifier)?.value.unwrap();

        self.tokens.expect(TokenKind::Equals)?;
        let value = self.expr()?;

        self.tokens.try_expect(&TokenKind::Semi);

        Ok(Stmt::Var {
            name,
            value: value.into(),
            is_mut,
        })
    }

    fn call(&mut self, name: String) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::PareL)?;

        if let Some(token) = self.tokens.try_expect(&TokenKind::PareR) {
            return Ok(Stmt::Call { name, args: vec![] });
        }

        let mut args = Vec::new();

        loop {
            let arg = self.expr()?;
            args.push(arg.into());

            if let Some(token) = self.tokens.try_expect(&TokenKind::PareR) {
                break;
            }

            self.tokens.expect(TokenKind::Comma)?;
        }

        self.tokens.try_expect(&TokenKind::Semi);

        Ok(Stmt::Call { name, args })
    }

    fn function(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::Func)?;

        let name = self.tokens.expect(TokenKind::Identifier)?.value.unwrap();

        self.tokens.expect(TokenKind::PareL)?;

        let mut args = mapping!();

        while let Some(arg) = self.tokens.try_expect(&TokenKind::Identifier) {
            let name = arg.value.clone().unwrap();

            if args.contains_key(&name) {
                return Err(ParserError::new(
                    ErrorType::DuplicateArgument,
                    arg,
                    "Duplicate argument",
                ));
            }

            let r#type = {
                if let Some(token) = self.tokens.try_expect(&TokenKind::Colon) {
                    Some(self.tokens.expect(TokenKind::Identifier)?.value.unwrap())
                } else {
                    None
                }
            };

            args.insert(name, r#type);

            if self.tokens.try_expect(&TokenKind::PareR).is_some() {
                break;
            }

            self.tokens.expect(TokenKind::Comma)?;
        }

        if args.is_empty() {
            self.tokens.expect(TokenKind::PareR)?;
        }

        let body = self.block()?;
        self.tokens.try_expect(&TokenKind::Semi);

        Ok(Stmt::Func { name, args, body })
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
            Parser::new(tokens).variable().unwrap(),
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
            Parser::new(tokens).variable().unwrap(),
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
            Parser::new(tokens).variable().unwrap(),
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
            Parser::new(tokens).function().unwrap(),
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
            Parser::new(tokens).function().unwrap(),
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

    #[test]
    fn syntax_error() {
        let tokens = Lexer::new(
            r#"
            let = "Hello, World!";
        "#,
        )
        .lex();

        let result = Parser::new(tokens).variable();

        assert!(result.is_err());

        assert_eq!(result.err().unwrap().r#type, ErrorType::ExpectedToken);
    }

    #[test]
    fn program() {
        let tokens = Lexer::new(
            r#"
            func main() {
                print("Hello, World!");
            }
        "#,
        )
        .lex();

        let parsed = Parser::new(tokens).parse();

        assert_eq!(
            parsed.unwrap(),
            vec![Stmt::Func {
                name: String::from("main"),
                args: mapping!(),
                body: Block {
                    stmts: vec![Stmt::Call {
                        name: String::from("print"),
                        args: vec![Expr::String(String::from("Hello, World!")).into()]
                    }
                    .into()]
                }
            }]
        )
    }
}
