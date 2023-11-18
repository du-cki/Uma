mod types;
mod utils;

use std::collections::VecDeque;

use crate::lexer::{Token, TokenType};

use self::utils::Buffer;
use self::types::{Stmt, Expr};

pub struct Parser {
    tokens: Buffer
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: Buffer::new(tokens)
        }
    }

    fn try_expect(&mut self, token_type: &TokenType) -> Option<Token> {
        if let Some(next_token) = self.tokens.peek() {
            if next_token.token_type == *token_type {
                return Some(self.tokens.consume().unwrap());
            }
        }

        return None
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
        let mut output_queue = VecDeque::<Token>::new();
        let mut op_stack     = Vec::<Token>::new();

        while let Some(token) = self.tokens.peek() {
            if token.token_type == TokenType::Semi {
                break;
            }

            let r = self.tokens.consume().unwrap(); // Would never error because we're peeking ahead.

            match r.token_type {
                ref t if t.is_op() => {
                    if let Some(top_op) = op_stack.last() {
                        if top_op.token_type.precedence() >= t.precedence() {
                            output_queue.push_back(op_stack.pop().unwrap());
                            continue;
                        }
                    }

                    op_stack.push(r)
                },
                TokenType::Number | TokenType::String | TokenType::Identifier => output_queue.push_back(r),
                TokenType::PareL => op_stack.push(r),
                TokenType::PareR => {
                    while let Some(op) = op_stack.pop() {
                        if op.token_type == TokenType::PareL { break; }

                        output_queue.push_back(op);
                    }
                }
                _ => unimplemented!()
            };
        }

        while let Some(op) = op_stack.pop() {
            if op.token_type == TokenType::PareL { panic!("mismatched parenthesis") }
            output_queue.push_back(op);
        }

        return output_queue
    }

    fn shunt_postfix(&self, tokens: &mut VecDeque<Token>) -> Expr {
        let mut stack = Vec::<Expr>::new();

        while let Some(token) = tokens.pop_front() {
            match token.token_type {
                ref t if t.is_op() => {
                    let (rhs, lhs) = (stack.pop(), stack.pop());

                    match (lhs, rhs) {
                        (Some(lhs_expr), Some(rhs_expr)) => {
                            stack.push(Expr::Binary {
                                lhs: Box::new(lhs_expr.into()),
                                op: token,
                                rhs: Box::new(rhs_expr.into())
                            });
                        },
                        // in cases of there being only an operator left in the stack.
                        (Some(_), None) => {
                            if stack.len() >= 2 {
                                let (lhs, rhs) = (stack.pop().unwrap(), stack.pop().unwrap());

                                stack.push(Expr::Binary {
                                    lhs: Box::new(lhs),
                                    op: token,
                                    rhs: Box::new(rhs)
                                });
                            } else {
                                panic!("not enough operands")
                            }
                        },
                        _ => unimplemented!()
                    }
                },
                TokenType::Number | TokenType::String | TokenType::Identifier => stack.push(token.into()),
                _ => unreachable!()
            }
        }

        if stack.len() > 1 {
            panic!("invalid expression, there are remaining in the stack")
        }

        stack.pop().unwrap()
    }

    fn shunt(&mut self) -> Expr {
        let mut infix = self.shunt_infix();

        self.shunt_postfix(&mut infix)
    }

    fn parse_variable(&mut self) -> Stmt {
        self.expect(TokenType::Let);
        let is_mut = self.try_expect(&TokenType::Mut).is_some();

        // TODO: this will only accept a single identifier (var name) with the current logic.
        // Which is fine, but soon, I'd prefer it if it could do additional functionality, I.E.
        // Destructuring the item, whether its a tuple, enum or a struct.
        //
        // I'm not implementing it currently as I don't even have the fundamental datatypes
        // implemented currently, but I plan to rectify that in the future.
        let name = self.expect(TokenType::Identifier).value.unwrap();

        self.expect(TokenType::Equals);
        let value: Expr = 'var: {
            let val = self.tokens.consume().unwrap();

            if let Some(token) = self.tokens.peek() {
                if token.token_type != TokenType::Semi {
                    // To make this work first, I need to parse the "scopes" (the brackets)
                    // then parse the characters until it hits the `;` or error if an `\n` occurs before,
                    // after which, I build the AST from. An rough output would be like this:
                    // ((1 + 2) + 2) * 10 / 2
                    //
                    // would be:
                    //        *
                    //       / \
                    //      +   \
                    //     / \   ÷
                    //    +   2 / \
                    //   / \   10  2
                    //  1   2
                    self.tokens.tokens.push_front(val);
                    break 'var self.shunt();
                }
            }

            val.into()
        };

        self.expect(TokenType::Semi);
        Stmt::Var { name, value, is_mut }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut program: Vec<Stmt> = Vec::new();

        while let Some(token) = self.tokens.peek() {
            match token.token_type {
                TokenType::Let => program.push(self.parse_variable()),
                _ => unimplemented!()
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
        let tokens = Lexer::new(r#"
            let mut foo = "bar";
        "#).lex();

        assert_eq!(
            Parser::new(tokens).parse_variable(),
            Stmt::Var {
                name: String::from("foo"),
                value: Expr::String("bar".to_string()),
                is_mut: true
            }
        )
    }

    #[test]
    fn parse_variable_with_arithmetic() {
        //    +
        //   / \
        //  9  10
        let tokens = Lexer::new(r#"
            let x = 9 + 10;
        "#).lex();

        assert_eq!(
            Parser::new(tokens).parse_variable(),
            Stmt::Var {
                name: String::from("x"),
                value: Expr::Binary {
                    lhs: Box::new(Expr::Number(String::from("9"))),
                    op: Token {
                        token_type: TokenType::Add,
                        value: None
                    },
                    rhs: Box::new(Expr::Number(String::from("10")))
                },
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
        let tokens = Lexer::new(r#"
            let mut x = (4 + 5) + 10 * 3;
        "#).lex();

        assert_eq!(
            Parser::new(tokens).parse_variable(),
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
