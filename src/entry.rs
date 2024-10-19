use std::fs;

use crate::{
    // codegen::{c::CBackend, CodeGenerator},
    codegen::{Codegen, CodegenBackend},
    lexer::Lexer,
    parser::{Parser, ParserError},
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

fn error(err: ParserError, source: &str, file_name: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let error_line = err.token.line;

    let max_line_num = (error_line + 2).min(lines.len());
    let line_num_width = max_line_num.to_string().len();

    print_line!(format!(
        "{} {:?}",
        "error:".red(),
        err.r#type,
    ));

    print_line!(format!(
        " {} {}:{}:{}",
        "-->".blue(),
        file_name,
        error_line,
        err.token.column,
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
        " ".repeat(err.token.column + line_num_width),
        format!("^ {}", err.message).red()
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
    let parsed = Parser::new(tokens).parse();

    if parsed.is_err() {
        return error(parsed.unwrap_err(), &src, &file);
    }

    Codegen::generate(CodegenBackend::C, parsed.unwrap(), file.replace(".uma", ""));
}
