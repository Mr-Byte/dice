use crate::{compiler::Compiler, syntax::LitInt, CompilerError, Value};

use super::NodeVisitor;

impl NodeVisitor<&LitInt> for Compiler {
    fn visit(&mut self, LitInt(value, span): &LitInt) -> Result<(), CompilerError> {
        match value {
            0 => self.assembler.push_i0(span.clone()),
            1 => self.assembler.push_i1(span.clone()),
            _ => self.assembler.push_const(Value::Int(*value), span.clone()),
        }

        Ok(())
    }
}