use super::NodeCompiler;
use crate::{compiler::Compiler, syntax::IfExpression, CompilerError};

impl NodeCompiler<&IfExpression> for Compiler {
    fn compile_node(
        &mut self,
        IfExpression(condition, primary, secondary, span): &IfExpression,
    ) -> Result<(), CompilerError> {
        // Both the primary and secondary blocks get their own scopes.
        // Only emit a jump over the secondary block if one exists.
        // Enforce that blocks without a secondary condition end in a discard expression.
        // Use the fancy new scoping mechanisms to help with patching branches.
        // If an if statement is at the top of a block and is not followed by a discard,
        // enforce that all branches must end in a discard.

        self.compile_node(*condition)?;
        let if_jump = self.assembler.jump_if_false(span.clone());
        self.compile_node(*primary)?;

        let else_jump = self.assembler.jump(span.clone());

        self.assembler.patch_jump(if_jump);

        if let Some(secondary) = secondary {
            self.compile_node(*secondary)?;
        }

        self.assembler.patch_jump(else_jump);

        Ok(())
    }
}