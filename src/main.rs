mod cli;
mod generate;
mod lexer;
mod parse;
mod semantic;
mod types;

use crate::cli::Cli;
use crate::parse::Parser;
use crate::semantic::check_semantics;
use clap::Parser as ClapParser;

fn main() {
    let options = Cli::parse();

    let contents = std::fs::read_to_string(options.input).unwrap();

    let mut parser = Parser::new(&contents);

    let spec = match parser.parse() {
        Ok(spec) => spec,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let metadata = match check_semantics(&spec) {
        Ok(metadata) => metadata,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let cpp_res = generate::cpp::generate_cli(&spec, &metadata);
    std::fs::write(options.output, cpp_res).unwrap();
}
