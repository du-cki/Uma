use std::{fs::File, io::Write, process::Command};

use crate::parser::{Block, Expr, Stmt};

pub(crate) struct CBackend {
    headers: Vec<String>,
}

#[allow(unreachable_patterns)]
impl CBackend {
    pub fn generate(exprs: Vec<Stmt>, out: String) {
        let mut code = String::new();

        let mut backend = CBackend { headers: vec![] };

        for expr in exprs {
            code.push_str(&backend.stmt(&expr));
        }

        let r#final = format!("{}\n\n{}", backend.headers.join("\n"), code);

        backend.compile(r#final, out);
    }

    fn compile(&self, source: String, out: String) {
        let tmp_dir = std::env::temp_dir();

        let c_file_path = tmp_dir.join("output.c");
        let binary_path = std::env::current_dir().unwrap().join(out);

        let mut c_file = File::create(&c_file_path).unwrap();
        c_file.write_all(source.as_bytes()).unwrap();

        let output = Command::new("gcc")
            .arg(c_file_path.to_str().unwrap())
            .arg("-o")
            .arg(binary_path.to_str().unwrap())
            .output()
            .unwrap();

        if !output.status.success() {
            panic!("{:#?}", String::from_utf8_lossy(&output.stderr))
        }
    }

    fn get_dependant(&mut self, name: &str) {
        let dependant = match name {
            "print" => r#"
                #include <stdio.h>
                #define print(fmt, ...) printf(fmt "\n", ##__VA_ARGS__)
            "#
            .trim(),
            _ => unimplemented!(),
        };

        let header_found = self
            .headers
            .iter()
            .find(|h| h.contains(dependant))
            .is_some();

        if !header_found {
            self.headers.push(dependant.to_string())
        }
    }

    fn block(&mut self, block: &Block) -> String {
        let mut code = String::new();

        for stmt in &block.stmts {
            code.push_str(&self.stmt(stmt));
        }

        code
    }

    fn stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Var {
                name,
                value,
                is_mut,
            } => {
                let type_decl = if *is_mut { "int" } else { "const int" };

                format!("{} {} = {};\n", type_decl, name, self.stmt(value))
            }
            Stmt::Assignment { name, value } => {
                format!("{} = {};\n", name, self.stmt(value))
            }
            Stmt::Call { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| self.stmt(arg))
                    .collect::<Vec<_>>()
                    .join(", ");

                self.get_dependant(name);

                format!("{}({});\n", name, args_str)
            }
            Stmt::Func {
                name,
                args: _,
                body,
            } => {
                // let args_str = args
                //     .iter()
                //     .map(|(arg, _)| format!("int {}", arg))
                //     .collect::<Vec<_>>()
                //     .join(", ");

                let args_str = "";

                format!("void {}({}) {{\n{}\n}}\n", name, args_str, self.block(body))
            }
            Stmt::Expr(expr) => self.expr(expr),
            Stmt::Empty => "".to_string(),
            stmt => unimplemented!("{:?}", stmt),
        }
    }

    fn expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary { lhs, op, rhs } => {
                format!("({} {} {})", self.stmt(lhs), op.repr(), self.stmt(rhs))
            }
            Expr::Identifier(name) => name.to_string(),
            Expr::Number(num) => num.to_string(),
            Expr::Float(num) => num.to_string(),
            Expr::String(s) => format!("\"{}\"", s),
            _ => unimplemented!(),
        }
    }
}
