use super::NodeVisitor;
use crate::compiler::Compiler;
use dice_core::error::Error;
use dice_syntax::Index;

impl NodeVisitor<&Index> for Compiler {
    fn visit(&mut self, node: &Index) -> Result<(), Error> {
        self.visit(node.expression)?;

        // NOTE: Take the current call context and temporarily store it on the stack, replacing it with a new one, so that
        // any call chains associated with evaluating the index short-circuit only in the index. Once the index is
        // compiled, the original call context is restored, so further chained calls will shirt-circuit correctly.
        let original_call_context = std::mem::take(&mut self.context()?.scope_stack().top_mut()?.call_context);
        *self.context()?.temporary_count() += 1;
        self.visit(node.index_expression)?;
        *self.context()?.temporary_count() -= 1;
        self.context()?.scope_stack().top_mut()?.call_context = original_call_context;
        self.assembler()?.load_index(node.span);

        Ok(())
    }
}
