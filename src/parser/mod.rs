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
                        return self.call(token);
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
        let primary = self.primary()?;
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
            TokenKind::Identifier => self.ident(),
            TokenKind::If => self.if_(),
            TokenKind::For => self.for_(),
            TokenKind::Semi => Ok(Stmt::Empty),
            _ => self.expr(),
        };

        stmt
    }

    fn for_(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::For)?;

        let iterator = self.tokens.expect(TokenKind::Identifier)?.value.unwrap();
        self.tokens.expect(TokenKind::In)?;

        let start = self.expr()?;
        self.tokens.expect(TokenKind::DotDot)?;
        let end = self.expr()?;

        let body = self.block()?;

        Ok(Stmt::For {
            iterator,
            start: start.into(),
            end: end.into(),
            body,
        })
    }

    fn if_(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.expect(TokenKind::If)?;

        self.tokens.expect(TokenKind::PareL)?;
        let condition = self.expr()?;
        self.tokens.expect(TokenKind::PareR)?;

        let consequence = self.block()?;

        let mut alternative = None;
        if self.tokens.try_expect(&TokenKind::Else).is_some() {
            let is_else_if = self
                .tokens
                .peek()
                .map_or(false, |t| t.kind == TokenKind::If);

            if is_else_if {
                alternative = Some(Box::new(self.if_()?));
            } else {
                let else_body = self.block()?;
                alternative = Some(Box::new(Stmt::Block(else_body)));
            }
        }

        Ok(Stmt::If {
            condition: condition.into(),
            consequence,
            alternative,
        })
    }

    fn ident(&mut self) -> Result<Stmt, ParserError> {
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

    fn call(&mut self, token: Token) -> Result<Stmt, ParserError> {
        let name = token.value.clone().unwrap();
        self.tokens.expect(TokenKind::PareL)?;

        if let Some(_) = self.tokens.try_expect(&TokenKind::PareR) {
            return Ok(Stmt::Call {
                name,
                args: vec![],
                token,
            });
        }

        let mut args = Vec::new();

        loop {
            let arg = self.expr()?;
            args.push(arg.into());

            if let Some(_) = self.tokens.try_expect(&TokenKind::PareR) {
                break;
            }

            self.tokens.expect(TokenKind::Comma)?;
        }

        self.tokens.try_expect(&TokenKind::Semi);

        Ok(Stmt::Call { name, args, token })
    }

    fn args(
        &mut self,
        with_types: bool,
        should_be_unique: bool,
    ) -> Result<(HashMap<String, Option<String>>, bool), ParserError> {
        self.tokens.expect(TokenKind::PareL)?;

        if let Some(_) = self.tokens.try_expect(&TokenKind::PareR) {
            return Ok((mapping!(), false));
        }

        let mut args = mapping!();
        let mut is_varadic = false;

        loop {
            if let Some(_) = self.tokens.try_expect(&TokenKind::Ellipsis) {
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
                if let Some(_) = self.tokens.try_expect(&TokenKind::Colon) {
                    Some(self.tokens.expect(TokenKind::Identifier)?.value.unwrap())
                } else {
                    None
                }
            } else {
                None
            };

            args.insert(name, type_);

            if let Some(_) = self.tokens.try_expect(&TokenKind::PareR) {
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

        let mut return_type = None;

        if let Some(_) = self.tokens.try_expect(&TokenKind::Colon) {
            return_type = Some(self.tokens.expect(TokenKind::Identifier)?.value.unwrap());
        }

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
                    return_type,
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
            return_type,
            args,
            body,
            is_varadic,
            external: None,
        })
    }
}

#[cfg(test)]
mod tests;
