mod cli;

mod codegen;
mod lexer;
mod parser;

mod colors;
mod entry;
mod utils;

use crate::cli::{Arg, ArgParser};

fn main() {
    let mut parser = ArgParser::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"));

    parser.add_arg(
        Arg::new("input")
            .action(cli::Action::Positional)
            .help("The .uma source file")
            .required(true),
    );

    parser.add_arg(
        Arg::new("output")
            .short("-o")
            .long("--output")
            .action(cli::Action::StoreValue)
            .help("The output executable name"),
    );

    let matches = parser.parse();

    entry::compile(
        matches.get_string("input").unwrap(),
        matches.get_string("output"),
    );
}
