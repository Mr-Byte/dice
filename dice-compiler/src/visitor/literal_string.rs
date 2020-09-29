use super::NodeVisitor;
use crate::{compiler::Compiler, error::CompilerError};
use dice_core::value::Value;
use dice_syntax::LitString;

impl NodeVisitor<&LitString> for Compiler {
    fn visit(&mut self, LitString { value, span }: &LitString) -> Result<(), CompilerError> {
        self.context()?
            .assembler()
            .push_const(Value::String(value.clone().into()), *span)?;

        Ok(())
    }
}