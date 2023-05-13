use super::Instruction;
use crate::helper::safe_add;
use std::{
    collection::VecDeque,
    error::Error,
    InvalidPC,
    InvalidContext,
}

impl Display for EvelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for EvelError {}

pub fn evel(inst: &[Instruction], line: &[char], is_depth: bool) -> result<(), EvelError> {
    if is_depth {
        eval_width(inst, line)
    } else {
        evel_width(inst, line)
    }
}

fn evel_width(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvelError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvelError::InvalidPC);
        }
    };

    match next {
        Instruction::Char(c) => {
            if c == sp_c {
                safe_add(&mut pc, &1, || EvelError::PCOverFlow)?;
                safe_add(&mut pc, &1, || EvelError::SPOverFlow)?;
            } else {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }
}