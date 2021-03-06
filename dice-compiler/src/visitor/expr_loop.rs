use super::{BlockKind, NodeVisitor};
use crate::{compiler::Compiler, scope_stack::ScopeKind};
use dice_core::error::{codes::INTERNAL_COMPILER_ERROR, Error};
use dice_syntax::{Loop, SyntaxNode};

impl NodeVisitor<&Loop> for Compiler {
    fn visit(&mut self, Loop { body, span }: &Loop) -> Result<(), Error> {
        if let SyntaxNode::Block(block) = self.syntax_tree.get(*body) {
            let block = block.clone();
            let loop_start = self.assembler()?.current_position();

            self.context()?
                .scope_stack()
                .push_scope(ScopeKind::Loop, Some(loop_start as usize));

            self.visit((&block, BlockKind::Loop))?;
            self.assembler()?.jump_back(loop_start, *span);

            let scope_close = self.context()?.scope_stack().pop_scope()?;

            for location in scope_close.exit_points.iter() {
                self.assembler()?.patch_jump(*location as u64);
            }

            self.assembler()?.push_unit(*span);
        } else {
            return Err(Error::new(INTERNAL_COMPILER_ERROR));
        }

        Ok(())
    }
}
