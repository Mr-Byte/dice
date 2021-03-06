use super::NodeVisitor;
use crate::compiler::Compiler;
use dice_core::error::Error;
use dice_syntax::IfExpression;

impl NodeVisitor<&IfExpression> for Compiler {
    fn visit(
        &mut self,
        IfExpression {
            condition,
            primary,
            secondary,
            span,
        }: &IfExpression,
    ) -> Result<(), Error> {
        self.visit(*condition)?;
        let if_jump = self.assembler()?.jump_if_false(*span);
        self.visit(*primary)?;

        let else_jump = self.assembler()?.jump(*span);

        self.assembler()?.patch_jump(if_jump);

        if let Some(secondary) = secondary {
            self.visit(*secondary)?;
        } else {
            self.assembler()?.push_unit(*span);
        }

        self.assembler()?.patch_jump(else_jump);

        Ok(())
    }
}
