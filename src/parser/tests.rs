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
                        args: vec![Expr::Float(String::from("3.14")).into()],
                        token: Token::new(
                            TokenKind::Identifier,
                            Some(String::from("round")),
                            2,
                            32
                        )
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
            return_type: None,
            is_varadic: false,
            body: Block {
                stmts: vec![Stmt::Call {
                    name: String::from("print"),
                    args: vec![Expr::String(String::from("Hello, World!")).into()],
                    token: Token::new(TokenKind::Identifier, Some(String::from("print")), 3, 17)
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
            return_type: None,
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
                return_type: None,
                is_varadic: false,
                body: Block { stmts: vec![] }
            },
            Stmt::Function {
                name: String::from("println"),
                args: mapping!(
                    String::from("fmt") => Some(String::from("String"))
                ),
                external: Some(String::from("stdio.h")),
                return_type: None,
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
fn if_statement() {
    let tokens = Lexer::new(
        r#"
        if (x > 60) {
            let mut x = 1;
        }
    "#,
    )
    .lex();

    assert_eq!(
        Parser::new(tokens).if_().unwrap(),
        Stmt::If {
            condition: Expr::Binary {
                lhs: Expr::Identifier(String::from("x")).into(),
                op: Token {
                    kind: TokenKind::BinaryGt,
                    value: None,
                    line: 2,
                    column: 15
                },
                rhs: Expr::Number(String::from("60")).into()
            }
            .into(),
            consequence: Block {
                stmts: vec![Stmt::Variable {
                    name: String::from("x"),
                    value: Expr::Number(String::from("1")).into(),
                    is_mut: true,
                }],
            },
            alternative: None,
        }
    );
}

#[test]
fn if_else_if_else_statement() {
    let tokens = Lexer::new(
        r#"
        if (x >= 30) {
            let x = 1;
        } else if (x + 10 >= 60) {
            let x = 2;
        } else {
            let x = 3;
        }
    "#,
    )
    .lex();

    assert_eq!(
        Parser::new(tokens).if_().unwrap(),
        Stmt::If {
            condition: Expr::Binary {
                lhs: Expr::Identifier(String::from("x")).into(),
                op: Token {
                    kind: TokenKind::BinaryGte,
                    value: None,
                    line: 2,
                    column: 15
                },
                rhs: Expr::Number(String::from("30")).into()
            }
            .into(),
            consequence: Block {
                stmts: vec![Stmt::Variable {
                    name: String::from("x"),
                    value: Expr::Number(String::from("1")).into(),
                    is_mut: false,
                }],
            },
            alternative: Some(Box::new(Stmt::If {
                condition: Expr::Binary {
                    lhs: Expr::Binary {
                        lhs: Expr::Identifier(String::from("x")).into(),
                        op: Token {
                            kind: TokenKind::Add,
                            value: None,
                            line: 4,
                            column: 22
                        },
                        rhs: Expr::Number(String::from("10")).into()
                    }
                    .into(),
                    op: Token {
                        kind: TokenKind::BinaryGte,
                        value: None,
                        line: 4,
                        column: 27
                    },
                    rhs: Expr::Number(String::from("60")).into()
                }
                .into(),
                consequence: Block {
                    stmts: vec![Stmt::Variable {
                        name: String::from("x"),
                        value: Expr::Number(String::from("2")).into(),
                        is_mut: false,
                    }],
                },
                alternative: Some(Box::new(Stmt::Block(Block {
                    stmts: vec![Stmt::Variable {
                        name: String::from("x"),
                        value: Expr::Number(String::from("3")).into(),
                        is_mut: false,
                    }],
                }))),
            })),
        }
    );
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
            return_type: None,
            is_varadic: false,
            body: Block {
                stmts: vec![Stmt::Call {
                    name: String::from("print"),
                    args: vec![Expr::String(String::from("Hello, World!")).into()],
                    token: Token::new(TokenKind::Identifier, Some(String::from("print")), 3, 17)
                }
                .into()]
            }
        }]
    )
}
