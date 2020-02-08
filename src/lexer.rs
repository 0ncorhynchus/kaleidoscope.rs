use std::num::ParseFloatError;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    EOF,
    Def,
    Extern,
    Identifier(String), // IdentifierStr
    Number(f64),        // NumVal
}

#[derive(Debug, PartialEq)]
enum LexerError {
    InvalidNumber(ParseFloatError),
    UnknownInitial(char),
}

impl From<ParseFloatError> for LexerError {
    fn from(err: ParseFloatError) -> Self {
        Self::InvalidNumber(err)
    }
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

    fn get_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespaces();

        if let Some(c) = self.get_char() {
            if c.is_ascii_alphabetic() {
                let mut ident = c.to_string();
                while let Some(c) = self.last_char {
                    if !c.is_ascii_alphanumeric() {
                        break;
                    }
                    ident.push(c);
                    self.get_char();
                }

                return Ok(match ident.as_str() {
                    "def" => Token::Def,
                    "extern" => Token::Extern,
                    _ => Token::Identifier(ident),
                });
            }

            if c.is_ascii_digit() || c == '.' {
                let mut num = c.to_string();
                while let Some(c) = self.last_char {
                    if !c.is_ascii_digit() && c != '.' {
                        break;
                    }
                    num.push(c);
                    self.get_char();
                }

                return Ok(Token::Number(num.parse()?));
            }

            if c == '#' {
                while let Some(c) = self.last_char {
                    if c == '\n' || c == '\r' {
                        return self.get_token();
                    }
                    self.get_char();
                }

                return Ok(Token::EOF);
            }

            Err(LexerError::UnknownInitial(c))
        } else {
            Ok(Token::EOF)
        }
    }

    fn skip_whitespaces(&mut self) {
        if let Some(c) = self.last_char {
            if !c.is_ascii_whitespace() {
                return;
            }
        } else {
            return;
        }

        while let Some(c) = self.last_char {
            if !c.is_ascii_whitespace() {
                break;
            }
            self.get_char();
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
