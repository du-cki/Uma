mod types;
mod utils;

use std::collections::{HashMap, VecDeque};

pub use types::Block;
pub use utils::{ErrorType, ParserError};

use crate::lexer::{Token, TokenKind};
use crate::mapping;

pub use self::types::{Expr, Stmt};
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
            kind => ParserError::new(
                ErrorType::UnexpectedToken,
                token,
                format!("Unexpected token occurred `{:#?}`", kind),
            ),
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
            TokenKind::Identifier => self.ident(token),
            TokenKind::Semi => Ok(Stmt::Empty),
            _ => self.expr(),
        };

        stmt
    }

    fn ident(&mut self, token: Token) -> Result<Stmt, ParserError> {
        if let Some(eq) = self.tokens.get(1) {
            if eq.kind == TokenKind::Equals {
                return self.assignment();
            }
        }

        self.expr()
    }

    fn assignment(&mut self) -> Result<Stmt, ParserError> {
        let token = self.tokens.consume(); // identifier
        let name = token.value.unwrap();

        self.tokens.consume(); // equals

        let value = self.expr()?;

        self.tokens.expect(TokenKind::Semi)?;

        return Ok(Stmt::Assignment {
            name,
            value: value.into(),
        });
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

        Ok(Stmt::Variable {
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

    fn args(
        &mut self,
        with_types: bool,
        should_be_unique: bool,
    ) -> Result<(HashMap<String, Option<String>>, bool), ParserError> {
        self.tokens.expect(TokenKind::PareL)?;

        if let Some(token) = self.tokens.try_expect(&TokenKind::PareR) {
            return Ok((mapping!(), false));
        }

        let mut args = mapping!();
        let mut is_varadic = false;

        loop {
            if let Some(elip) = self.tokens.try_expect(&TokenKind::Ellipsis) {
                self.tokens.try_expect(&TokenKind::Comma);
                self.tokens.expect(TokenKind::PareR)?;

                is_varadic = true;

                break;
            }

            let arg = self.tokens.expect(TokenKind::Identifier)?;
            let name = arg.value.clone().unwrap();

            if should_be_unique && args.contains_key(arg.value.as_ref().unwrap()) {
                return ParserError::new(
                    ErrorType::DuplicateArgument,
                    arg,
                    format!("Duplicate argument `{}`", name),
                );
            }

            let type_: Option<String> = if with_types {
                if let Some(token) = self.tokens.try_expect(&TokenKind::Colon) {
                    Some(self.tokens.expect(TokenKind::Identifier)?.value.unwrap())
                } else {
                    None
                }
            } else {
                None
            };

            args.insert(name, type_);

            if let Some(token) = self.tokens.try_expect(&TokenKind::PareR) {
                break;
            }

            self.tokens.expect(TokenKind::Comma)?;
        }

        return Ok((args, is_varadic));
    }

    fn attribute(&mut self) -> Result<Stmt, ParserError> {
        let name = self.tokens.expect(TokenKind::Identifier)?.value.unwrap();

        self.tokens.expect(TokenKind::PareL)?;
        let value = self.tokens.expect(TokenKind::String)?.value.unwrap();
        self.tokens.expect(TokenKind::PareR)?;

        Ok(Stmt::Attribute { name, value })
    }

    fn function(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::Func)?;

        let name = self.tokens.expect(TokenKind::Identifier)?.value.unwrap();
        let (args, is_varadic) = self.args(true, true)?;

        let external = self.tokens.try_expect(&TokenKind::At);
        if external.is_some() {
            let attr = self.attribute()?;

            if let Stmt::Attribute {
                name: attr_name,
                value,
            } = attr
            {
                if attr_name != "requires" {
                    return ParserError::new(
                        ErrorType::InvalidAttribute,
                        external.unwrap(),
                        format!("Invalid attribute `{}`, expected `requires`", attr_name),
                    );
                }

                return Ok(Stmt::Function {
                    name,
                    args,
                    external: Some(value),
                    is_varadic,
                    body: Block { stmts: vec![] },
                });
            }
        }

        let body = self.block()?;
        self.tokens.try_expect(&TokenKind::Semi);

        Ok(Stmt::Function {
            name,
            args,
            body,
            external: None,
            is_varadic,
        })
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
            Stmt::Variable {
                name: String::from("foo"),
                value: Expr::String(String::from("bar")).into(),
                is_mut: true
            }
        )
    }

    #[test]
    fn variable_re_assignment() {
        let tokens = Lexer::new(
            r#"
            let mut foo = "bar";

            foo = "baz";
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).parse().unwrap(),
            vec![
                Stmt::Variable {
                    name: String::from("foo"),
                    value: Expr::String(String::from("bar")).into(),
                    is_mut: true
                },
                Stmt::Assignment {
                    name: String::from("foo"),
                    value: Expr::String(String::from("baz")).into()
                }
            ]
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
            Stmt::Variable {
                name: String::from("foo"),
                value: Expr::Binary {
                    lhs: Expr::Number(String::from("9")).into(),
                    op: Token::new(TokenKind::Add, None, 2, 25),
                    rhs: Expr::Binary {
                        lhs: Expr::Number(String::from("10")).into(),
                        op: Token::new(TokenKind::Multi, None, 2, 30),
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
            Stmt::Variable {
                name: String::from("x"),
                value: Expr::Binary {
                    lhs: Expr::Binary {
                        lhs: Expr::Number(String::from("4")).into(),
                        op: Token::new(TokenKind::Add, None, 2, 28),
                        rhs: Expr::Number(String::from("5")).into()
                    }
                    .into(),
                    op: Token::new(TokenKind::Add, None, 2, 33),
                    rhs: Expr::Binary {
                        lhs: Expr::Number(String::from("10")).into(),
                        op: Token::new(TokenKind::Multi, None, 2, 38),
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
            Stmt::Function {
                name: String::from("main"),
                args: mapping!(),
                external: None,
                is_varadic: false,
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
            Stmt::Function {
                name: String::from("sum"),
                args: mapping!(
                    String::from("x") => Some(String::from("Int")),
                    String::from("y") => Some(String::from("Int"))
                ),
                external: None,
                is_varadic: false,
                body: Block {
                    stmts: vec![Stmt::Return(
                        Expr::Binary {
                            lhs: Expr::Identifier(String::from("x")).into(),
                            op: Token::new(TokenKind::Add, None, 3, 26),
                            rhs: Expr::Identifier(String::from("y")).into()
                        }
                        .into()
                    )]
                }
            }
        )
    }

    #[test]
    fn attribute_funcs() {
        let tokens = Lexer::new(
            r#"
            func printf(fmt: String) @requires("stdio.h")

            func println(fmt: String, ...) @requires("stdio.h")
        "#,
        )
        .lex();

        assert_eq!(
            Parser::new(tokens).parse().unwrap(),
            vec![
                Stmt::Function {
                    name: String::from("printf"),
                    args: mapping!(
                        String::from("fmt") => Some(String::from("String"))
                    ),
                    external: Some(String::from("stdio.h")),
                    is_varadic: false,
                    body: Block { stmts: vec![] }
                },
                Stmt::Function {
                    name: String::from("println"),
                    args: mapping!(
                        String::from("fmt") => Some(String::from("String"))
                    ),
                    external: Some(String::from("stdio.h")),
                    is_varadic: true,
                    body: Block { stmts: vec![] }
                },
            ]
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
            vec![Stmt::Function {
                name: String::from("main"),
                args: mapping!(),
                external: None,
                is_varadic: false,
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
