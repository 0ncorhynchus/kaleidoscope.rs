mod ir;
mod lexer;
mod parser;

use crate::ir::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut generator = IRGenerator::new();
    loop {
        print!("parser> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        if buffer.trim().is_empty() {
            continue;
        }
        if buffer.trim() == "quit" {
            generator.dump_module();
            break;
        }

        let lexer = Lexer::new(buffer.chars());
        let tokens = lexer.collect::<Result<Vec<_>, _>>();
        let tokens = match tokens {
            Ok(tokens) => tokens,
            Err(err) => {
                eprintln!("\x1b[1;31merror\x1b[m: {:?}", err);
                continue;
            }
        };

        let mut parser = Parser::new(tokens.into_iter());
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(err) => {
                eprintln!("\x1b[1;31merror\x1b[m: {}", err);
                continue;
            }
        };
        // println!("{:?}", ast);

        match generator.gen(&ast) {
            Ok(ir) => {
                ir.dump();
                println!();
            }
            Err(err) => {
                eprintln!("\x1b[1;31merror\x1b[m: {:?}", err);
            }
        }
    }
    Ok(())
}
