#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
extern crate alloc;

mod memory;
mod stack;
mod valids;
mod opcode;
mod trap;
mod eval;

pub use crate::memory::Memory;
pub use crate::stack::Stack;
pub use crate::valids::Valids;
pub use crate::opcode::{Opcode, ExternalOpcode};
pub use crate::trap::{Trap, ExitReason};

use core::ops::Range;
use alloc::rc::Rc;
use crate::eval::{eval, Control};

/// Core execution layer for EVM.
pub struct Core {
    /// Program code.
    code: Rc<Vec<u8>>,
    /// Program counter.
    position: Result<usize, ExitReason>,
    /// Return value range.
    return_range: Range<usize>,
    /// Code validity maps.
    valids: Valids,
    /// Memory.
    memory: Memory,
    /// Stack.
    stack: Stack,
}

impl Core {
    pub fn step(&mut self) -> Result<(), Trap> {
        let position = self.position?;

        match self.code.get(position).map(|v| Opcode::parse(*v)) {
            Some(Ok(opcode)) => {
                match eval(opcode, position, self) {
                    Control::Continue(p) => {
                        self.position = Ok(position + p);
                        Ok(())
                    },
                    Control::Exit(e) => {
                        self.position = Err(e);
                        Err(Trap::Exit(e))
                    },
                    Control::Jump(p) => {
                        if self.valids.is_valid(p) {
                            self.position = Ok(p);
                            Ok(())
                        } else {
                            self.position = Err(ExitReason::InvalidJump);
                            Err(Trap::Exit(ExitReason::InvalidJump))
                        }
                    },
                }
            },
            Some(Err(external)) => {
                self.position = Ok(position + 1);
                Err(Trap::External(external))
            },
            None => {
                self.position = Err(ExitReason::CodeEnded);
                Err(Trap::Exit(ExitReason::CodeEnded))
            },
        }
    }
}