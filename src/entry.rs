use std::fs;

use crate::{
    codegen::{Codegen, CodegenBackend},
    lexer::{Lexer, Token},
    parser::Parser,
};

use super::colors::*;

macro_rules! print_line {
    ($line_num:expr, $msg:expr, $width:expr) => {{
        let line_num = $line_num;

        println!(
            "  {} {}",
            format!("{:width$} |", line_num, width = $width).blue(),
            $msg
        );
    }};

    ($msg:expr, $width:expr) => {{
        println!("{} {}", " ".repeat($width), $msg);
    }};

    ($msg:expr) => {{
        println!("   {}", $msg);
    }};
}

fn error(token: &Token, err_type: &str, message: &str, source: &str, file_name: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let error_line = token.line;

    let max_line_num = (error_line + 2).min(lines.len());
    let line_num_width = max_line_num.to_string().len();

    print_line!(format!("{} {}", "error:".red(), err_type));

    print_line!(format!(
        " {} {}:{}:{}",
        "-->".blue(),
        file_name,
        error_line,
        token.column,
    ));

    if error_line > 2 {
        print_line!(error_line - 2, lines[error_line - 3], line_num_width)
    }

    if error_line > 1 {
        print_line!(error_line - 1, lines[error_line - 2], line_num_width);
    }

    print_line!(error_line, lines[error_line - 1], line_num_width);

    print_line!(format!(
        "{} {}",
        " ".repeat(token.column + line_num_width),
        format!("^ {}", message).red()
    ));

    if error_line < lines.len() {
        print_line!(error_line + 1, lines[error_line], line_num_width);
    }

    if error_line + 1 < lines.len() {
        print_line!(error_line + 2, lines[error_line + 1], line_num_width);
    }

    std::process::exit(1);
}

pub fn compile(file: String) {
    let src = match fs::read_to_string(&file) {
        Ok(src) => src,
        Err(e) => panic!("{}", e),
    };

    let tokens = Lexer::new(&src).lex();

    let ast = match Parser::new(tokens).parse() {
        Ok(ast) => ast,
        Err(err) => {
            error(
                &err.token,
                &format!("{:?}", err.r#type),
                &err.message,
                &src,
                &file,
            );
            return;
        }
    };

    match Codegen::generate(CodegenBackend::C, ast, file.replace(".uma", "")) {
        Err(err) => error(&err.token, "SemanticError", &err.message, &src, &file),
        _ => (),
    };
}
