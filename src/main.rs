#[derive(Clone, Debug, PartialEq)]
enum Token {
    EOF,
    Def,
    Extern,
    Identifier(String), // IdentifierStr
    Number(f64),        // NumVal
}

struct Lexer<I> {
    iter: I,
    last_char: Option<char>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    fn new(iter: I) -> Self {
        let mut iter = iter;
        let last_char = iter.next();
        Self { iter, last_char }
    }

    fn get_char(&mut self) -> Option<char> {
        let c = self.last_char;
        self.last_char = self.iter.next();
        c
    }
}

fn main() {
    println!("Hello, world!");
}
