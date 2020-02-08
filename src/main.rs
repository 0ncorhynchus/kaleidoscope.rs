mod lexer;

use crate::lexer::Lexer;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    loop {
        print!("lexer> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        if buffer.trim().is_empty() {
            continue;
        }
        if buffer.trim() == "quit" {
            break;
        }

        let lexer = Lexer::new(buffer.chars());
        let tokens = lexer.collect::<Result<Vec<_>, _>>();
        match tokens {
            Ok(tokens) => {
                println!("{:?}", tokens);
            }
            Err(err) => {
                    println!("Error: {:?}", err);
            }
        }
    }
    Ok(())
}
