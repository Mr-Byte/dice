use super::NodeVisitor;
use crate::{compiler::Compiler, error::CompilerError};
use dice_syntax::LitList;

impl NodeVisitor<&LitList> for Compiler {
    fn visit(&mut self, LitList { items: value, span }: &LitList) -> Result<(), CompilerError> {
        for item in value {
            self.visit(*item)?;
        }

        self.context()?.assembler().create_list(value.len() as u8, *span);

        Ok(())
    }
}