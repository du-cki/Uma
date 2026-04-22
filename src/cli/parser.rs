use std::{env, process};

use crate::{
    cli::utils::{ArgMatches, ArgValue},
    colors::*,
};

#[derive(PartialEq)]
pub enum Action {
    StoreTrue,
    StoreFalse,
    StoreValue,
    Append,
    Positional,
}

pub struct Arg {
    pub(crate) name: String,
    pub(crate) short: Option<String>,
    pub(crate) long: Option<String>,
    pub(crate) help: String,
    pub(crate) action: Action,
    pub(crate) is_required: bool,
    pub(crate) default_val: Option<String>,
}

impl Arg {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            short: None,
            long: None,
            help: String::new(),
            action: Action::StoreValue,
            is_required: false,
            default_val: None,
        }
    }

    pub fn short(mut self, short: &str) -> Self {
        self.short = Some(short.to_string());
        self
    }

    pub fn long(mut self, long: &str) -> Self {
        self.long = Some(long.to_string());
        self
    }

    pub fn help(mut self, help: &str) -> Self {
        self.help = help.to_string();
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    pub fn required(mut self, req: bool) -> Self {
        self.is_required = req;
        self
    }

    pub fn default(mut self, val: &str) -> Self {
        self.default_val = Some(val.to_string());
        self
    }
}

pub struct ArgParser {
    program_name: String,
    description: String,
    version: String,
    args: Vec<Arg>,
}

impl ArgParser {
    pub fn new(name: &str) -> Self {
        Self {
            program_name: name.to_string(),
            description: String::new(),
            version: String::new(),
            args: Vec::new(),
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn version(mut self, ver: &str) -> Self {
        self.version = ver.to_string();
        self
    }

    pub fn add_arg(&mut self, arg: Arg) {
        self.args.push(arg);
    }

    pub fn parse(&self) -> ArgMatches {
        let args: Vec<String> = env::args().skip(1).collect();
        self.parse_from(args)
    }

    pub(crate) fn parse_from(&self, raw_args: Vec<String>) -> ArgMatches {
        let mut matches = ArgMatches::new();
        let mut iter = raw_args.into_iter();

        let mut positionals = self.args.iter().filter(|a| a.action == Action::Positional);
        let mut force_positionals = false;

        for arg in &self.args {
            if let Some(def) = &arg.default_val {
                matches
                    .values
                    .insert(arg.name.clone(), ArgValue::String(def.clone()));
            }
        }

        while let Some(token) = iter.next() {
            if token == "-h" || token == "--help" {
                self.print_help();
                process::exit(0)
            }

            if token == "-v" || token == "--version" {
                println!("{}", self.version);
                process::exit(0)
            }

            if token == "--" {
                force_positionals = true;
                continue;
            }

            if token.starts_with('-') && !force_positionals {
                let matched_arg = self.args.iter().find(|a| {
                    a.short.as_deref() == Some(&token) || a.long.as_deref() == Some(&token)
                });

                match matched_arg {
                    Some(arg) => match arg.action {
                        Action::StoreTrue => {
                            matches
                                .values
                                .insert(arg.name.clone(), ArgValue::Bool(true));
                        }
                        Action::StoreFalse => {
                            matches
                                .values
                                .insert(arg.name.clone(), ArgValue::Bool(false));
                        }
                        Action::StoreValue => {
                            let val = iter.next().unwrap_or_else(|| {
                                self.error_and_exit(&format!(
                                    "Argument '{}' requires a value.",
                                    token
                                ));
                            });

                            matches
                                .values
                                .insert(arg.name.clone(), ArgValue::String(val));
                        }

                        Action::Append => {
                            let val = iter.next().unwrap_or_else(|| {
                                self.error_and_exit(&format!(
                                    "Argument '{}' requires a value.",
                                    token
                                ));
                            });

                            let list = matches
                                .values
                                .entry(arg.name.clone())
                                .or_insert(ArgValue::List(Vec::new()));

                            if let ArgValue::List(vec) = list {
                                vec.push(val);
                            }
                        }
                        Action::Positional => unreachable!(),
                    },
                    None => self.error_and_exit(&format!("Unknown argument: {}", token)),
                }
            } else {
                if let Some(pos_arg) = positionals.next() {
                    matches
                        .values
                        .insert(pos_arg.name.clone(), ArgValue::String(token));
                } else {
                    self.error_and_exit(&format!("Unexpected positional argument: {}", token));
                }
            }
        }

        if matches.values.len() == 0 {
            self.print_help();
            process::exit(0)
        }

        for arg in &self.args {
            if arg.is_required && !matches.values.contains_key(&arg.name) {
                self.error_and_exit(&format!("Missing required argument: {}", arg.name));
            }
        }

        matches
    }

    fn print_help(&self) {
        if !self.description.is_empty() {
            println!("{}\n", self.description);
        }

        println!("{}", "USAGE:".green());
        println!("    {} [OPTIONS] [ARGS]\n", self.program_name.blue());

        println!("{}", "OPTIONS:".green());
        for arg in self.args.iter().filter(|a| a.action != Action::Positional) {
            let short = arg.short.as_deref().unwrap_or("  ");
            let long = arg.long.as_deref().unwrap_or("");
            let val_hint = if arg.action == Action::StoreValue || arg.action == Action::Append {
                " <VAL>"
            } else {
                ""
            };

            println!(
                "    {}, {:<15} {}",
                short.blue(),
                format!("{}{}", long.blue(), val_hint),
                arg.help
            );
        }

        let has_positionals = self.args.iter().any(|a| a.action == Action::Positional);
        if has_positionals {
            println!("\n{}", "ARGS:".green());
            for arg in self.args.iter().filter(|a| a.action == Action::Positional) {
                let req = if arg.is_required { "(required)" } else { "" };
                println!(
                    "    {:<19} {} {}",
                    format!("<{}>", arg.name).blue(),
                    arg.help,
                    req
                );
            }
        }
    }

    fn error_and_exit(&self, msg: &str) -> ! {
        eprintln!("error: {}", msg);
        eprintln!("\nFor more information try --help");
        process::exit(1);
    }
}
