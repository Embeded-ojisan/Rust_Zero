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


impl Generator {
    fn get_code(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        self.gen_expr(ast)?;
        self.inc_pc()?;
        self.insts.push(Instruction::Match);
        Ok(())
    }

    fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        match ast {
            AST::Char(c)            =>      self.gen_char(*c)?,
            AST::Or(e1, e2)         =>      self.gen_or(e1, e2)?,
            AST::Plus(e1)           =>      self.gen_code(e)?,
            AST::Star(e1)           =>      {
                match &**e1 {
                    AST::Star(_)    =>  self.gen_expr(&e1)?,
                    AST::Seq(e2) if e2.len() == 1 => {
                        self.gen_expr(e3)?
                    } else {
                        self.gen_star(e1)?
                    }
                }
                e => self.gen_star(&e)?,
            }
        }
    }
}