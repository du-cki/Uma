use c::CBackend;

use crate::parser::Stmt;

pub mod c;

pub enum CodegenBackend {
    C,
}

pub struct Codegen;

impl Codegen {
    pub fn generate(backend: CodegenBackend, exprs: Vec<Stmt>, out: String) {
        match backend {
            CodegenBackend::C => CBackend::generate_and_run(exprs, out),
        };
    }
}
