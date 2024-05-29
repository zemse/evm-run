use revm::{
    interpreter::{Interpreter, OpCode},
    Database, EvmContext, Inspector,
};

use crate::stack_fmt::VecU256;

#[derive(Default)]
pub struct CustomTracer {
    op_length_max: usize,
}

impl<DB: Database> Inspector<DB> for CustomTracer {
    fn step(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
        let opcode_num = interp.current_opcode();
        let opcode_str = OpCode::new(opcode_num)
            .map(|op| op.as_str())
            .unwrap_or("UNKNOWN");

        if opcode_str.len() > self.op_length_max {
            self.op_length_max = opcode_str.len();
        }
        print!(
            "{opcode_num:0>2x} {opcode_str:pad_length$}",
            pad_length = self.op_length_max
        );
    }

    fn step_end(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
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
