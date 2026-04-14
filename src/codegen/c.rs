use std::{
    fs::File,
    io::{self, Write},
    process::{self, Command},
};

use crate::parser::{Block, Expr, Stmt};

pub(crate) struct CBackend {
    headers: Vec<String>,
}

impl CBackend {
    pub fn generate(exprs: Vec<Stmt>, out: impl ToString) -> String {
        let mut code = String::new();
        let mut backend = CBackend { headers: vec![] };

        for expr in exprs {
            code.push_str(&backend.stmt(&expr, true));
        }

        backend.compile(
            format!("{}\n\n{}", backend.headers.join("\n"), code),
            out.to_string(),
        )
    }

    pub fn generate_and_run(exprs: Vec<Stmt>, out: String) {
        let result = Self::generate(exprs, &out);

        let output = Command::new(result)
            .output()
            .expect("failed to execute process");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stdout().flush().unwrap();
    }

    fn compile(&self, source: String, out: String) -> String {
        let tmp_dir = std::env::temp_dir();

        let c_buffer_fp = tmp_dir.join("output.c");
        let output_fp = std::env::current_dir().unwrap().join(out);

        let mut c_file = File::create(&c_buffer_fp).unwrap();
        c_file.write_all(source.as_bytes()).unwrap();

        let output = Command::new("gcc")
            .arg(&c_buffer_fp)
            .arg("-o")
            .arg(&output_fp)
            .output()
            .unwrap();

        if !output.status.success() {
            io::stderr().write_all(&output.stderr).unwrap();
            io::stderr().flush().unwrap();

            process::exit(1);
        }

        return output_fp.display().to_string();
    }

    fn add_header_if_not_exist(&mut self, header: String) {
        if !self.headers.contains(&header) {
            self.headers.push(header);
        }
    }

    fn infer_type(&self, expr: &Stmt) -> String {
        match expr {
            Stmt::Expr(expr) => match expr {
                Expr::String(_) => "char*".to_string(),
                Expr::Number(_) => "int".to_string(),
                Expr::Binary { lhs, op: _, rhs } => {
                    let l = self.infer_type(lhs);

                    if l == self.infer_type(rhs) {
                        return l;
                    }

                    panic!("Mismatched types found.")
                }
                _ => "auto".to_string(),
            },
            Stmt::Function {
                name,
                return_type,
                args,
                ..
            } => {
                let func_rt = return_type.clone().unwrap_or("void".to_string());

                let args_str = args
                    .iter()
                    .map(|(arg, typ)| format!("{} {}", typ.as_ref().unwrap(), arg))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{} {}({})", func_rt, name, args_str)
            }
            Stmt::Call { .. } => String::from("__auto_type"),
            _ => unreachable!(),
        }
    }

    fn block(&mut self, block: &Block) -> String {
        let mut code = String::new();

        for stmt in &block.stmts {
            code.push_str(&self.stmt(stmt, true));
        }

        code
    }

    fn stmt(&mut self, stmt: &Stmt, with_semi: bool) -> String {
        match stmt {
            Stmt::Block(block) => format!("{{\n{}}}\n", self.block(block)),
            Stmt::Variable {
                name,
                value,
                is_mut,
            } => {
                let type_decl = if *is_mut {
                    self.infer_type(value)
                } else {
                    format!("const {}", self.infer_type(value))
                };

                format!("{} {} = {};\n", type_decl, name, self.stmt(value, false))
            }
            Stmt::Assignment { name, value } => {
                format!("{} = {};\n", name, self.stmt(value, false))
            }
            Stmt::Call { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| self.stmt(arg, false))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "{}({}){}",
                    name,
                    args_str,
                    with_semi.then(|| ";\n").unwrap_or("")
                )
            }
            Stmt::Function { external, body, .. } => {
                if let Some(ext) = external {
                    self.add_header_if_not_exist(format!("#include <{}>", ext));

                    return "".to_string();
                }

                let func_proto = self.infer_type(stmt);

                format!("{} {{\n{}\n}}\n", func_proto, self.block(body))
            }
            Stmt::Return(stmt) => {
                let expr = self.stmt(stmt, false);

                format!("return {};", expr)
            }
            Stmt::If {
                condition,
                consequence,
                alternative,
            } => {
                let cond_str = self.stmt(condition, false);
                let consequence_str = self.block(consequence);

                let mut code = format!("if ({}) {{\n{}}}", cond_str, consequence_str);

                if let Some(alt) = alternative {
                    match &**alt {
                        Stmt::If { .. } => {
                            code.push_str(&format!(" else {}", self.stmt(alt, false)));
                        }
                        Stmt::Block(block) => {
                            code.push_str(&format!(" else {{\n{}}}\n", self.block(block)));
                        }
                        _ => unreachable!("alternative must be an If or a Block"),
                    }
                } else {
                    code.push('\n');
                }

                code
            }
            Stmt::Expr(expr) => self.expr(expr),
            Stmt::Empty => "".to_string(),
            stmt => unimplemented!("{:?}", stmt),
        }
    }

    fn expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary { lhs, op, rhs } => {
                format!(
                    "({} {} {})",
                    self.stmt(lhs, false),
                    op.repr(),
                    self.stmt(rhs, false)
                )
            }
            Expr::Identifier(name) => name.to_string(),
            Expr::Number(num) => num.to_string(),
            Expr::Float(num) => num.to_string(),
            Expr::String(value) => format!("\"{}\"", value.replace("\n", r#"\n"#)),
        }
    }
}
