use c::CBackend;

use crate::parser::Stmt;

pub mod c;

pub enum CodegenBackend {
    C,
}

pub struct Codegen;

#[allow(unreachable_patterns)]
impl Codegen {
    pub fn generate(backend: CodegenBackend, exprs: Vec<Stmt>, out: String) {
        match backend {
            CodegenBackend::C => CBackend::generate(exprs, out),
            _ => unimplemented!(),
        };
    }
}
