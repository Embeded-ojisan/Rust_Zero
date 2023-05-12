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
                        if let Some(e3 @ AST::Star(_)) = e2.get(0) {
                            self.gen_expr(e3)?
                        } else {
                            self.gen_star(e1)?
                        }
                    }
                    e => self.gen_star(&e)?,
                }
            }
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,    
        }
        Ok(())
    }

    fn gen_char(&mut self, c: char) -> Result<(), CodeGenError> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn get_star(&mut self, e: &AST) -> Result<(), CodeGenError> {
        let l1 = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        self.gen_expr(e)?;

        self.inc_pc()?;
        self.insts.push(Instruction::Jump(l1));

        if let Some(Instruction::Split(_, l3)) = self.insts.get_mut(l1) {
            *l3 = self.pc;
            Ok(())
        } else {
            Err(CodeGenError::FailStar)
        }
    }

    fn get_seq(&mut self, exprs: &[AST]) => Result<(), CodeGenError> {
        for e in exprs {
            self.gen_expr(e)?;
        }

        Ok(())
    }

    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
    }
}