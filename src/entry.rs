use std::fs;

use crate::{
    lexer::Lexer,
    parser::{Parser, ParserError},
};

use super::colors::*;

fn println(num: usize, s: &str) {
    println!("  {} {}", format!("{} |", num).blue(), s);
}

fn error(err: &ParserError, source: &str, file_name: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let error_line = err.token.line;

    println!(
        " {} {}:{}:{}:",
        "-->".blue(),
        file_name,
        error_line,
        err.token.column
    );

    if error_line > 2 {
        println(error_line - 2, lines[error_line - 3]);
    }

    if error_line > 1 {
        println(error_line - 1, lines[error_line - 2]);
    }

    println(error_line, lines[error_line - 1]);

    println!(
        "{} {} {}",
        "    |".blue(),
        " ".repeat(err.token.column),
        format!("^ {}", err.message).red()
    );

    if error_line < lines.len() {
        println(error_line + 1, lines[error_line]);
    }

    if error_line + 1 < lines.len() {
        println(error_line + 2, lines[error_line + 1]);
    }

    std::process::exit(1);
}

pub fn compile(file: String) {
    let src = match fs::read_to_string(&file) {
        Ok(src) => src,
        Err(e) => panic!("{}", e),
    };

    let tokens = Lexer::new(&src).lex();
    let parsed = Parser::new(tokens).parse();

    if parsed.is_err() {
        error(&parsed.unwrap_err(), &src, &file);
    }

    println!("parsed successfully")
}
