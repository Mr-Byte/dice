use super::{BlockKind, NodeVisitor};
use crate::{compiler::Compiler, error::CompilerError, scope_stack::ScopeKind};
use dice_syntax::{SyntaxNode, WhileLoop};

impl NodeVisitor<&WhileLoop> for Compiler {
    fn visit(&mut self, WhileLoop { condition, body, span }: &WhileLoop) -> Result<(), CompilerError> {
        if let Some(SyntaxNode::Block(block)) = self.syntax_tree.get(*body) {
            let block = block.clone();
            let loop_start = self.context()?.assembler().current_position();

            self.context()?
                .scope_stack()
                .push_scope(ScopeKind::Loop, Some(loop_start as usize));
            self.visit(*condition)?;

            let loop_end = self.context()?.assembler().jump_if_false(*span);

            self.visit((&block, BlockKind::<&str>::Loop))?;
            self.context()?.assembler().jump_back(loop_start, *span);
            self.context()?.assembler().patch_jump(loop_end);

            let scope_close = self.context()?.scope_stack().pop_scope()?;

            for location in scope_close.exit_points.iter() {
                self.context()?.assembler().patch_jump(*location as u64);
            }

            self.context()?.assembler().push_unit(*span);
        } else {
            return Err(CompilerError::InternalCompilerError(String::from(
                "While loop bodies should only ever contain blocks.",
            )));
        }

        Ok(())
    }
}