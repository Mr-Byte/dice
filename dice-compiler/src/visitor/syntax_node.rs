use super::{expr_block::BlockKind, NodeVisitor};
use crate::{compiler::Compiler, error::CompilerError};
use dice_syntax::{SyntaxNode, SyntaxNodeId};

impl NodeVisitor<SyntaxNodeId> for Compiler {
    fn visit(&mut self, node: SyntaxNodeId) -> Result<(), CompilerError> {
        let node = self
            .syntax_tree
            .get(node)
            .cloned()
            .expect("Node should never be empty.");

        match &node {
            SyntaxNode::LitIdent(literal) => self.visit(literal)?,
            SyntaxNode::LitUnit(literal) => self.visit(literal)?,
            SyntaxNode::LitNone(literal) => self.visit(literal)?,
            SyntaxNode::LitBool(literal) => self.visit(literal)?,
            SyntaxNode::LitInt(literal) => self.visit(literal)?,
            SyntaxNode::LitFloat(literal) => self.visit(literal)?,
            SyntaxNode::LitString(literal) => self.visit(literal)?,
            SyntaxNode::LitAnonymousFn(literal) => self.visit(literal)?,
            SyntaxNode::LitObject(literal) => self.visit(literal)?, //self.visit(literal)?,
            SyntaxNode::LitList(literal) => self.visit(literal)?,
            SyntaxNode::SafeAccess(_) => todo!(),
            SyntaxNode::FieldAccess(field_access) => self.visit(field_access)?,
            SyntaxNode::Index(_) => todo!(),
            SyntaxNode::Assignment(assignment) => self.visit(assignment)?,
            SyntaxNode::Unary(unary) => self.visit(unary)?,
            SyntaxNode::Binary(binary) => self.visit(binary)?,
            SyntaxNode::VarDecl(variable) => self.visit(variable)?,
            SyntaxNode::FnDecl(func) => self.visit(func)?,
            SyntaxNode::OpDecl(func) => self.visit(func)?,
            SyntaxNode::IfExpression(conditional) => self.visit(conditional)?,
            SyntaxNode::WhileLoop(while_loop) => self.visit(while_loop)?,
            SyntaxNode::ForLoop(_) => todo!(),
            SyntaxNode::Break(break_node) => self.visit(break_node)?,
            SyntaxNode::Continue(continue_node) => self.visit(continue_node)?,
            SyntaxNode::Block(block) => self.visit((block, BlockKind::<&str>::Block))?,
            SyntaxNode::FunctionCall(fn_call) => self.visit(fn_call)?,
            SyntaxNode::Return(return_expr) => self.visit(return_expr)?,
        }

        Ok(())
    }
}