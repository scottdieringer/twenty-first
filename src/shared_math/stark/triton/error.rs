use std::error::Error;
use std::fmt::{Display, Formatter};
use InstructionError::*;

#[derive(Debug, Clone)]
pub enum InstructionError {
    InstructionPointerOverflow,
    OpStackTooShallow,
    JumpStackTooShallow,
}

impl Display for InstructionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionPointerOverflow => {
                write!(f, "Instruction pointer points outside of program")
            }

            OpStackTooShallow => {
                write!(f, "Instruction addresses too deeply into the stack")
            }

            JumpStackTooShallow => {
                write!(f, "Jump stack does not contain return address")
            }
        }
    }
}

impl Error for InstructionError {}

pub fn vm_err<T>(runtime_error: InstructionError) -> Result<T, Box<dyn Error>> {
    Err(vm_fail(runtime_error))
}

pub fn vm_fail(runtime_error: InstructionError) -> Box<dyn Error> {
    Box::new(runtime_error)
}
