mod types;
mod utils;

use std::collections::VecDeque;

use crate::lexer::{Token, TokenType};

use self::types::{Expr, Stmt};
use self::utils::Buffer;

pub struct Parser {
    tokens: Buffer,
}

#[allow(unused)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: Buffer::new(tokens),
        }
    }

    fn try_expect(&mut self, token_type: &TokenType) -> Option<Token> {
        if let Some(next_token) = self.tokens.peek() {
            if next_token.token_type == *token_type {
                return Some(self.tokens.consume().unwrap());
            }
        }

        return None;
    }

    fn expect(&mut self, token_type: TokenType) -> Token {
        let n = self.try_expect(&token_type);
        if let Some(next_token) = n {
            return next_token;
        }

        // TODO: better error handling
        panic!("Unexpected token encountered, expected: `{token_type:#?}` but got: {n:#?}");
    }

    fn shunt_infix(&mut self) -> VecDeque<Token> {
        let mut output_stack = VecDeque::<Token>::new();
        let mut operand_stack = Vec::<Token>::new();

        while let Some(token) = self.tokens.peek() {
            if token.token_type == TokenType::Semi {
                break;
            }

            let r = self.tokens.consume().unwrap(); // Would never error because we're peeking ahead.

            match r.token_type {
                ref t if t.is_op() => {
                    if let Some(top_op) = operand_stack.last() {
                        if top_op.token_type.precedence() >= t.precedence() {
                            output_stack.push_back(operand_stack.pop().unwrap());
                            continue;
                        }
                    }

                    operand_stack.push(r)
                }
                TokenType::Number | TokenType::String | TokenType::Float => {
                    output_stack.push_back(r)
                }
                TokenType::Identifier => output_stack.push_back(r),
                TokenType::PareL => operand_stack.push(r),
                TokenType::PareR => {
                    while let Some(op) = operand_stack.pop() {
                        if op.token_type == TokenType::PareL {
                            break;
                        }

                        output_stack.push_back(op);
                    }
                }
                _ => unimplemented!(),
            };
        }

        while let Some(op) = operand_stack.pop() {
            if op.token_type == TokenType::PareL {
                panic!("mismatched parenthesis")
            }

            output_stack.push_back(op);
        }

        return output_stack;
    }

    fn shunt_postfix(&self, tokens: &mut VecDeque<Token>) -> Expr {
        let mut stack = Vec::new();

        while let Some(token) = tokens.pop_front() {
            match token.token_type {
                ref t if t.is_op() => {
                    let rhs = stack.pop().expect("not enough operands");
                    let lhs = stack.pop().expect("not enough operands");

                    stack.push(Expr::Binary {
                        lhs: Box::new(lhs),
                        op: token,
                        rhs: Box::new(rhs),
                    });
                }
                _ => stack.push(token.into()),
            }
        }

        if stack.len() > 1 {
            panic!("invalid expression, remaining operands in the stack");
        }

        stack.pop().unwrap()
    }

    fn shunt(&mut self) -> Expr {
        let mut infix = self.shunt_infix();

        self.shunt_postfix(&mut infix)
    }

    fn call(&mut self, name: String) -> Stmt {
        self.expect(TokenType::PareL);
        self.tokens.consume();

        let mut args = Vec::new();

        while let Some(token) = self.tokens.peek() {
            if token.token_type != TokenType::PareR {
                if token.token_type == TokenType::Comma {
                    continue;
                }

                args.push(self.shunt());
            }
        }

        self.expect(TokenType::Semi);
        Stmt::Call { name, args }
    }

    fn variable(&mut self) -> Stmt {
        self.expect(TokenType::Let);
        let is_mut = self.try_expect(&TokenType::Mut).is_some();

        let name = self.expect(TokenType::Identifier).value.unwrap();

        self.expect(TokenType::Equals);
        let value = self.shunt();
        self.expect(TokenType::Semi);

        Stmt::Var {
            name,
            value,
            is_mut,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut program: Vec<Stmt> = Vec::new();

        while let Some(token) = self.tokens.peek() {
            match token.token_type {
                TokenType::Let => program.push(self.variable()),
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
                value: Expr::String("bar".to_string()),
                is_mut: true
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
                value: Expr::Binary {
                    lhs: Box::new(Expr::Binary {
                        lhs: Box::new(Expr::Number("4".to_string())),
                        op: Token {
                            token_type: TokenType::Add,
                            value: None
                        },
                        rhs: Box::new(Expr::Number("5".to_string()))
                    }),
                    op: Token {
                        token_type: TokenType::Add,
                        value: None
                    },
                    rhs: Box::new(Expr::Binary {
                        lhs: Box::new(Expr::Number("10".to_string())),
                        op: Token {
                            token_type: TokenType::Multi,
                            value: None
                        },
                        rhs: Box::new(Expr::Number("3".to_string()))
                    })
                },
                is_mut: true
            }
        )
    }
}
