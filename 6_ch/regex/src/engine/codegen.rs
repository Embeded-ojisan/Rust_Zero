use super::{parse::AST, Instruction};
use crate::helper::safe_add;
use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut::Formatter<`_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}
 
impl Error for CodeGenError {}

#[derive(Default, Debug)]
struct Generator {
    pc: usize,
    insts: Vec<Instruction>, 
}

pub fn get_code(ast: &AST) -> Result<Vec<Instruction>, CodeGenError> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}