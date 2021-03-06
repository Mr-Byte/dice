use super::NodeVisitor;
use crate::compiler::Compiler;
use dice_core::error::Error;
use dice_syntax::LitObject;

impl NodeVisitor<&LitObject> for Compiler {
    fn visit(&mut self, LitObject { items, span }: &LitObject) -> Result<(), Error> {
        self.assembler()?.create_object(*span);

        for (field, value) in items {
            self.assembler()?.dup(0, *span);
            self.visit(*value)?;
            self.assembler()?.store_field(field.clone(), *span)?;
            self.assembler()?.pop(*span);
        }

        Ok(())
    }
}
