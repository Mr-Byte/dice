use dice_core::{
    error::{codes::INVALID_ERROR_PROPAGATE_USAGE, Error},
    protocol::error::{IS_OK, RESULT},
};
use dice_syntax::ErrorPropagate;

use crate::{compiler::Compiler, compiler_stack::CompilerKind};

use super::NodeVisitor;

impl NodeVisitor<&ErrorPropagate> for Compiler {
    fn visit(&mut self, ErrorPropagate { expression, span }: &ErrorPropagate) -> Result<(), Error> {
        if !matches!(
            self.context()?.kind(),
            CompilerKind::Function { .. } | CompilerKind::Method { .. }
        ) {
            return Err(Error::new(INVALID_ERROR_PROPAGATE_USAGE).with_span(*span));
        }

        self.visit(*expression)?;
        let error_propagate_jump;
        let temporary_count = *self.context()?.temporary_count();

        emit_bytecode! {
            self.assembler()?, *span => [
                DUP 0;
                LOAD_FIELD IS_OK;
                JUMP_IF_TRUE -> error_propagate_jump;
                for _ in 0..temporary_count => [
                    SWAP;
                    POP;
                ]
                {self.visit_return(*span)?};
                PATCH_JUMP <- error_propagate_jump;
                LOAD_FIELD RESULT;
            ]
        }

        Ok(())
    }
}
