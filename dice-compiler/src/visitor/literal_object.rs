use super::NodeVisitor;
use crate::compiler::Compiler;
use dice_error::compiler_error::CompilerError;
use dice_syntax::LitObject;

impl NodeVisitor<&LitObject> for Compiler {
    fn visit(&mut self, LitObject { items, span }: &LitObject) -> Result<(), CompilerError> {
        self.context()?.assembler().create_object(*span);

        for (field, value) in items {
            self.context()?.assembler().dup(0, *span);
            self.visit(*value)?;
            self.context()?.assembler().store_field(field.clone(), *span)?;
            self.context()?.assembler().pop(*span);
        }

        Ok(())
    }
}
