use crate::lexer::*;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum ExprAST {
    Number(f64),
    Variable(String),
    BinaryOp {
        op: Operator,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Call {
        callee: String,
        args: Vec<Self>,
    },
    Prototype(Prototype),
    Function {
        proto: Prototype,
        body: Box<Self>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Prototype {
    name: String,
    args: Vec<String>,
}

type ParserError = &'static str;
type Result<T> = std::result::Result<T, ParserError>;

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    iter: Peekable<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<ExprAST> {
        let ast = match self.iter.peek() {
            Some(Token::Def) => {
                self.iter.next();
                self.parse_defeinition()?
            }
            Some(Token::Extern) => {
                self.iter.next();
                self.parse_extern()?
            }
            Some(_) => self.parse_expression()?,
            None => {
                return Err("Unimplemented");
            }
        };

        match self.iter.peek() {
            Some(Token::SemiColon) => {
                self.iter.next();
            }
            Some(_) => {
                let mut remainds = Vec::new();
                for token in &mut self.iter {
                    remainds.push(token);
                }
                eprintln!("\x1b[1;33mwarning\x1b[m: Invalid syntax: {:?}", remainds);
            }
            None => {
                eprintln!("\x1b[1;33mwarning\x1b[m: Expected semicolon");
            }
        }
        Ok(ast)
    }

    fn parse_defeinition(&mut self) -> Result<ExprAST> {
        let proto = self.parse_prototype()?;
        let body = self.parse_expression()?;
        Ok(ExprAST::Function {
            proto,
            body: Box::new(body),
        })
    }

    fn parse_extern(&mut self) -> Result<ExprAST> {
        Ok(ExprAST::Prototype(self.parse_prototype()?))
    }

    fn parse_prototype(&mut self) -> Result<Prototype> {
        if let Some(Token::Identifier(name)) = self.iter.next() {
            if self.iter.next() != Some(Token::OpenParenthesis) {
                return Err("Expected '(' in prototype");
            }
            let mut args = Vec::new();
            while let Some(Token::Identifier(arg)) = self.iter.peek() {
                args.push(arg.clone());
                self.iter.next();
            }
            if self.iter.next() != Some(Token::CloseParenthesis) {
                return Err("Expected ')' in prototype");
            }
            Ok(Prototype { name, args })
        } else {
            Err("Expected function name in prototype")
        }
    }

    fn parse_expression(&mut self) -> Result<ExprAST> {
        let lhs = self.parse_primary()?;
        self.parse_op_and_rhs(0, lhs)
    }

    fn parse_primary(&mut self) -> Result<ExprAST> {
        match self.iter.next() {
            Some(Token::Number(value)) => Ok(ExprAST::Number(value)),
            Some(Token::Identifier(name)) => {
                if self.iter.peek() != Some(&Token::OpenParenthesis) {
                    Ok(ExprAST::Variable(name))
                } else {
                    self.iter.next();
                    let mut args = Vec::new();
                    if self.iter.peek() != Some(&Token::CloseParenthesis) {
                        loop {
                            args.push(self.parse_expression()?);
                            match self.iter.peek() {
                                Some(Token::CloseParenthesis) => {
                                    break;
                                }
                                Some(Token::Comma) => {
                                    self.iter.next();
                                }
                                _ => {
                                    return Err("Expected ')' or ',' in argument list");
                                }
                            }
                        }
                    }
                    self.iter.next(); // consume ')'
                    Ok(ExprAST::Call { callee: name, args })
                }
            }
            Some(Token::OpenParenthesis) => self.parse_parenthesis(),
            _ => Err("Expected expression"),
        }
    }

    fn parse_parenthesis(&mut self) -> Result<ExprAST> {
        let ast = self.parse_expression()?;
        if self.iter.next() == Some(Token::CloseParenthesis) {
            Ok(ast)
        } else {
            Err("Expected ')'")
        }
    }

    fn parse_op_and_rhs(&mut self, expr_prec: u8, lhs: ExprAST) -> Result<ExprAST> {
        let mut lhs = lhs;
        loop {
            if let Some(Token::Operator(op)) = self.iter.peek() {
                let op = *op;
                let token_prec = self.get_prec(op);
                if token_prec < expr_prec {
                    return Ok(lhs);
                }

                self.iter.next();

                let mut rhs = self.parse_primary()?;
                if let Some(Token::Operator(next_op)) = self.iter.peek() {
                    let next_op = *next_op;
                    if token_prec < self.get_prec(next_op) {
                        rhs = self.parse_op_and_rhs(token_prec + 1, rhs)?;
                    }
                }

                lhs = ExprAST::BinaryOp {
                    op: op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
            } else {
                return Ok(lhs);
            }
        }
    }

    fn get_prec(&self, op: Operator) -> u8 {
        match op {
            Operator::LessThan => 10,
            Operator::Plus => 20,
            Operator::Minus => 20,
            Operator::Times => 40,
        }
    }
}
