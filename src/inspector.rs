use std::{
    borrow::{Borrow, BorrowMut},
    default,
};

use indicatif::ProgressBar;
use revm::{
    interpreter::{CallInputs, CallOutcome, Interpreter, OpCode},
    Database, EvmContext, Inspector,
};

use crate::stack_fmt::VecU256;

pub struct CustomTracer<'a> {
    print: bool,
    depth: usize,
    op_length_max: usize,
    progress_bar: Option<&'a ProgressBar>,
    result: Option<&'a mut CustomTracerResult>,
}

#[derive(Default)]
pub struct CustomTracerResult {
    pub outcome: Option<CallOutcome>,
}

impl<'a> CustomTracer<'a> {
    pub fn default() -> CustomTracer<'a> {
        CustomTracer {
            print: true,
            depth: 0,
            op_length_max: 0,
            progress_bar: None,
            result: None,
        }
    }

    pub fn new(
        progress_bar: &'a ProgressBar,
        result: &'a mut CustomTracerResult,
    ) -> CustomTracer<'a> {
        CustomTracer {
            print: false,
            depth: 0,
            op_length_max: 0,
            progress_bar: Some(progress_bar),
            result: Some(result),
        }
    }
}

impl<'a, DB: Database> Inspector<DB> for CustomTracer<'a> {
    fn step(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
        if self.print {
            let opcode_num = interp.current_opcode();
            let some_opcode = OpCode::new(opcode_num);

            let opcode_str = some_opcode.map(|op| op.as_str()).unwrap_or("UNKNOWN");
            if opcode_str.len() > self.op_length_max {
                self.op_length_max = opcode_str.len();
            }

            // some_opcode.map(|op| op.gas)

            print!(
                "{opcode_num:0>2x} {opcode_str:pad_length$}",
                pad_length = self.op_length_max
            );
        }
    }

    fn step_end(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
        if self.print {
            let mut stack = VecU256(vec![]);
            for i in 0..8 {
                if let Ok(value) = interp.stack.peek(i) {
                    stack.0.push(value);
                } else {
                    break;
                }
            }
            println!(
                "\tStack: {stack}\t(gas_used: {:?})",
                interp.gas.limit() - interp.gas.remaining()
            );
        }
    }

    #[inline]
    fn call(
        &mut self,
        context: &mut EvmContext<DB>,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        let _ = context;
        let _ = inputs;
        self.depth += 1;
        None
    }

    #[inline]
    fn call_end(
        &mut self,
        context: &mut EvmContext<DB>,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        let _ = context;
        let _ = inputs;
        self.depth -= 1;
        if self.depth == 0 {
            // println!("Tx ended with {:?}", outcome.gas());
            let mut value = self.result.as_mut().unwrap();
            value.outcome = Some(outcome.clone());
        }
        outcome
    }
}
