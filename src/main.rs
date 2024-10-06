mod codegen;
mod lexer;
mod parser;

mod colors;
mod entry;
mod utils;

fn main() {
    let mut args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        panic!("Not enough arguments");
    }

    entry::compile(args.remove(1));
}
