use super::NodeVisitor;
use crate::{
    compiler::Compiler,
    error::CompilerError,
    scope_stack::{ScopeKind, State},
};
use dice_syntax::Block;

pub enum BlockKind<'args, T: AsRef<str>> {
    Block,
    Loop,
    Function(&'args [T]),
}

impl<'args, T: AsRef<str>> NodeVisitor<(&Block, BlockKind<'args, T>)> for Compiler {
    fn visit(&mut self, (block, kind): (&Block, BlockKind<'args, T>)) -> Result<(), CompilerError> {
        self.context()?.scope_stack().push_scope(ScopeKind::Block, None);

        if let BlockKind::Function(args) = kind {
            for arg in args {
                self.context()?.scope_stack().add_local(
                    arg.as_ref().to_owned(),
                    State::Local {
                        is_mutable: false,
                        is_initialized: true,
                    },
                )?;
            }
        }

        self.scan_item_decls(block)?;

        for expression in block.expressions.iter() {
            self.visit(*expression)?;
            self.context()?.assembler().pop(block.span);
            self.patch_expression_exit_points()?;
        }

        match block.trailing_expression {
            Some(trailing_expression) => {
                self.visit(trailing_expression)?;
                self.patch_expression_exit_points()?;
            }
            None => self.context()?.assembler().push_unit(block.span),
        }

        if let BlockKind::Loop = kind {
            self.context()?.assembler().pop(block.span);
        }

        let scope = self.context()?.scope_stack().pop_scope()?;

        for variable in scope.variables {
            if variable.is_captured {
                self.context()?
                    .assembler()
                    .close_upvalue(variable.slot as u8, block.span);
            }
        }

        // NOTE: If in context of a function, implicitly return the top item on the stack.
        // If the previous instruction was a return, this will never execute.
        if let BlockKind::Function(_) = kind {
            self.context()?.assembler().ret(block.span)
        }

        Ok(())
    }
}

impl Compiler {
    fn patch_expression_exit_points(&mut self) -> Result<(), CompilerError> {
        let exit_points: Vec<usize> = self
            .context()?
            .scope_stack()
            .top_mut()?
            .expression_exit_points
            .drain(..)
            .collect();

        for exit_point in exit_points {
            self.context()?.assembler().patch_jump(exit_point as u64);
        }

        Ok(())
    }
}
