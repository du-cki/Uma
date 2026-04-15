pub mod c;

use crate::{lexer::Token, parser::Stmt};
use c::CBackend;

#[derive(Debug, PartialEq)]
pub struct CodegenError {
    pub message: String,
    pub token: Token,
}

impl CodegenError {
    pub fn new(message: impl Into<String>, token: Token) -> Self {
        Self {
            message: message.into(),
            token,
        }
    }
}

pub enum CodegenBackend {
    C,
}

pub struct Codegen;

impl Codegen {
    pub fn generate(
        backend: CodegenBackend,
        exprs: Vec<Stmt>,
        out: &String,
    ) -> Result<(), CodegenError> {
        match backend {
            CodegenBackend::C => CBackend::generate_and_run(exprs, out)?,
        };

        Ok(())
    }
}
