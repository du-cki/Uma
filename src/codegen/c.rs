use std::{fs::File, io::Write, process::Command};

use crate::parser::{Block, Expr, Stmt};

pub(crate) struct CBackend {
    headers: Vec<String>,
}

#[allow(unreachable_patterns)]
impl CBackend {
    pub fn generate(exprs: Vec<Stmt>, out: impl ToString) {
        let mut code = String::new();

        let mut backend = CBackend { headers: vec![] };

        for expr in exprs {
            code.push_str(&backend.stmt(&expr));
        }

        backend.compile(
            format!("{}\n\n{}", backend.headers.join("\n"), code),
            out.to_string(),
        );
    }

    pub fn generate_and_run(exprs: Vec<Stmt>, out: String) {
        Self::generate(exprs, &out);

        let output = Command::new(out)
            .output()
            .expect("failed to execute process");

        println!("{}", String::from_utf8_lossy(&output.stdout));
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

    fn add_header_if_not_exist(&mut self, header: String) {
        if !self.headers.contains(&header) {
            self.headers.push(header);
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
            Stmt::Variable {
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

                format!("{}({});\n", name, args_str)
            }
            Stmt::Function {
                name,
                args: _,
                is_varadic: _,
                external,
                body,
            } => {
                // let args_str = args
                //     .iter()
                //     .map(|(arg, _)| format!("int {}", arg))
                //     .collect::<Vec<_>>()
                //     .join(", ");

                if let Some(ext) = external {
                    self.add_header_if_not_exist(format!("#include <{}>", ext));
                    return "".to_string();
                }

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
            Expr::String(value) => format!("\"{}\"", value.replace("\n", r#"\n"#)),
            _ => unimplemented!(),
        }
    }
}
