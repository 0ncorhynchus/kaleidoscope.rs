use std::num::ParseFloatError;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    EOF,
    Def,
    Extern,
    Identifier(String), // IdentifierStr
    Number(f64),        // NumVal
}
#[derive(Debug, PartialEq)]
pub enum LexerError {
    InvalidNumber(ParseFloatError),
    UnknownInitial(char),
}

impl From<ParseFloatError> for LexerError {
    fn from(err: ParseFloatError) -> Self {
        Self::InvalidNumber(err)
    }
}

pub struct Lexer<I> {
    iter: I,
    last_char: Option<char>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        let mut iter = iter;
        let last_char = iter.next();
        Self { iter, last_char }
    }

    fn consume_char(&mut self) {
        self.last_char = self.iter.next();
    }

    fn get_char(&mut self) -> Option<char> {
        let c = self.last_char;
        self.consume_char();
        c
    }

    fn get_token(&mut self) -> Result<Token, LexerError> {
        if let Some(c) = self.last_char {
            if c.is_ascii_whitespace() {
                self.skip_chars(char::is_ascii_whitespace);
            }
        }

        if let Some(c) = self.get_char() {
            if c.is_ascii_alphabetic() {
                let ident = self.get_chars(c, char::is_ascii_alphanumeric);

                return Ok(match ident.as_str() {
                    "def" => Token::Def,
                    "extern" => Token::Extern,
                    _ => Token::Identifier(ident),
                });
            }

            if c.is_ascii_digit() || c == '.' {
                let num = self.get_chars(c, |c| c.is_ascii_digit() || c == &'.');

                return Ok(Token::Number(num.parse()?));
            }

            if c == '#' {
                self.skip_chars(|c| c != &'\n' && c != &'\r');

                if self.last_char.is_some() {
                    return self.get_token();
                } else {
                    return Ok(Token::EOF);
                }
            }

            Err(LexerError::UnknownInitial(c))
        } else {
            Ok(Token::EOF)
        }
    }

    fn skip_chars<P: Fn(&char) -> bool>(&mut self, predicate: P) {
        while let Some(c) = self.last_char {
            if !predicate(&c) {
                return;
            }
            self.consume_char();
        }
    }

    fn get_chars<P: Fn(&char) -> bool>(&mut self, initial: char, predicate: P) -> String {
        let mut chars = initial.to_string();
        while let Some(c) = self.last_char {
            if !predicate(&c) {
                break;
            }
            chars.push(c);
            self.consume_char();
        }
        chars
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.get_token() {
            Ok(token) => {
                if token == Token::EOF {
                    None
                } else {
                    Some(Ok(token))
                }
            }
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "3.141592 def fib x";
        let mut lexer = Lexer::new(input.chars());
        assert_eq!(lexer.get_token(), Ok(Token::Number(3.141592)));
        assert_eq!(lexer.get_token(), Ok(Token::Def));
        assert_eq!(lexer.get_token(), Ok(Token::Identifier("fib".to_string())));
        assert_eq!(lexer.get_token(), Ok(Token::Identifier("x".to_string())));
        assert_eq!(lexer.get_token(), Ok(Token::EOF));
    }
}
