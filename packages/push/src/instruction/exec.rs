use super::{instruction_error::PushInstructionError, Instruction, NumOpens};
use crate::{
    error::{Error, InstructionResult},
    instruction::PushInstruction,
    push_vm::{
        program::PushProgram,
        stack::{StackDiscard, StackError},
        HasStack,
    },
};

/*
 * exec_if requires a boolean and two (additional) values on the exec stack.
 * If the bool is true, we remove the second of the two exec stack values,
 * and if it's false, we remove the first.
 */

/*
 * exec_while requires a boolean and one additional value on the exec stack.
 * If the bool is true, then you push a copy of the "body" onto the exec,
 * followed by another copy of exec_while.
 */

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExecInstruction {
    Noop,
    Dup,
    IfElse,
    When,
    Unless,
}

impl From<ExecInstruction> for PushInstruction {
    fn from(instr: ExecInstruction) -> Self {
        Self::Exec(instr)
    }
}

impl NumOpens for ExecInstruction {
    fn num_opens(&self) -> usize {
        match self {
            Self::Noop => 0,
            Self::Dup | Self::When | Self::Unless => 1,
            Self::IfElse => 2,
        }
    }
}
impl<S> Instruction<S> for ExecInstruction
where
    S: Clone + HasStack<PushProgram> + HasStack<bool>,
{
    type Error = PushInstructionError;

    fn perform(&self, state: S) -> InstructionResult<S, Self::Error> {
        match self {
            Self::Noop => Ok(state),
            Self::IfElse | Self::When | Self::Unless => {
                match self {
                    Self::When => {
                        let condition = state.stack::<bool>().top();
                        let block = state.stack::<PushProgram>().top();
                        match (condition, block) {
                            // If there is a boolean that is true and a block, discard the boolean
                            // and leave the block so that it may be
                            // performed next.
                            (Ok(true), Ok(_)) => Ok(state).with_stack_discard::<bool>(1),
                            // If there is a boolean that is false and a block, discard both the
                            // boolean and and the block since we don't
                            // want to perform that block.
                            (Ok(false), Ok(_)) => Ok(state)
                                .with_stack_discard::<bool>(1)
                                .with_stack_discard::<PushProgram>(1),
                            // If there is no boolean but there is a block, discard the block since
                            // we only want to perform it if there is a
                            // boolean that is true.
                            (Err(StackError::Underflow { .. }), Ok(_)) => {
                                Ok(state).with_stack_discard::<PushProgram>(1)
                            }
                            // If there is no block, then we just skip this instruction regardless
                            // of whether there's a boolean or not.
                            (
                                Ok(_) | Err(StackError::Underflow { .. }),
                                Err(StackError::Underflow { .. }),
                            ) => Ok(state),
                            (Err(e), _) | (_, Err(e)) => Err(Error::recoverable(state, e)),
                        }
                    }
                    Self::Unless => todo!(),
                    Self::IfElse => todo!(),
                    _ => {
                        unreachable!("We failed to handle an Exec instruction: {self:?}")
                    }
                }
            }
            Self::Dup => todo!(), // Could overflow exec
        }
    }
}

#[cfg(test)]
mod exec_instruction_tests {
    use super::ExecInstruction;
    use crate::{instruction::Instruction, push_vm::push_state::PushState};

    #[test]
    fn noop_is_correct() {
        let state = PushState::builder()
            .with_max_stack_size(1000)
            .with_no_program()
            .build();
        let result_state = ExecInstruction::Noop.perform(state).unwrap();
        panic!("Not sure how to check that the result state is the same as the input state");
    }
}
