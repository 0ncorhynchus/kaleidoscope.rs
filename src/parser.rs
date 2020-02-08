use crate::lexer::*;

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
