mod codegen;
mod evaluator;
mod parser;

use crate::helper::DnyError;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Instruction {
    Char(char),
    Match,
    Jump(usize),
    Split(usize, usize),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<`_>) -> fmt::Result {
        match self {
            Instruction::Char(c)                => write!(f, "char {}", c),
            Instruction::Match                  => write!(f, "match"),
            Instruction::Jump(addr)             => write!(f, "Jump {:>04}", addr),
            Instruction::Split(addr1, addr2)    => write!(f, "split {:>04}, {:>04}", addr1, addr2),
        }
    }
}

pub fn print(expr: &str) -> Result<(), DnyError> {
    println!("expr: {expr}");
    let ast = parser::parser(expr)?;
    println!("AST: {:?}", ast);

    println!();
    println!("code:");
    let code = codegen::get_code(&ast)?;
    for (n, c) in code.iter().enumerate() {
        println!("{:>04}: {c}", n);
    }

    Ok(())
}

pub fn do_matching(expr: &str, line: &str, is_depth: bool) -> Result<bool, DnyError> {
    let ast = parser::parser(expr)?;
    let code = codegen::get_code(&ast)?;
    let line = line.chars().collect::<Vec<char>>();
    Ok(evaluator::eval(&code, &line, is_depth)?)
}