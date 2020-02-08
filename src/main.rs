#[derive(Clone, Debug, PartialEq)]
enum Token {
    EOF,
    Def,
    Extern,
    Identifier(String), // IdentifierStr
    Number(f64),        // NumVal
}

fn main() {
    println!("Hello, world!");
}
