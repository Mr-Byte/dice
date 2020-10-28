use super::NodeVisitor;
use crate::{compiler::Compiler, scope_stack::ScopeVariable};
use dice_core::value::Symbol;
use dice_error::{compiler_error::CompilerError, span::Span};
use dice_syntax::{Assignment, AssignmentOperator, FieldAccess, Index, SyntaxNode, SyntaxNodeId};

impl NodeVisitor<&Assignment> for Compiler {
    fn visit(&mut self, assignment: &Assignment) -> Result<(), CompilerError> {
        let lhs = self.syntax_tree.get(assignment.lhs_expression);

        match lhs {
            SyntaxNode::LitIdent(lit_ident) => {
                let target: Symbol = (&*lit_ident.name).into();
                self.assign_ident(target, assignment)
            }
            SyntaxNode::FieldAccess(field_access) => {
                let field_access = field_access.clone();
                self.assign_field(field_access, assignment)
            }
            SyntaxNode::Index(index) => {
                let index = index.clone();
                self.assign_index(index, assignment)
            }
            _ => Err(CompilerError::InvalidAssignmentTarget),
        }
    }
}

// TODO: Convert this to the emit_bytecode! macro.
impl Compiler {
    fn assign_index(&mut self, target: Index, assignment: &Assignment) -> Result<(), CompilerError> {
        self.visit(target.expression)?;
        self.visit(target.index_expression)?;

        match assignment.operator {
            AssignmentOperator::Assignment => {
                self.visit(assignment.rhs_expression)?;
                self.assembler()?.store_index(assignment.span);
                self.assembler()?.pop(assignment.span);
                self.assembler()?.push_unit(assignment.span);
            }
            operator => {
                self.assembler()?.dup(1, assignment.span);
                self.assembler()?.dup(1, assignment.span);
                self.assembler()?.load_index(assignment.span);
                self.visit(assignment.rhs_expression)?;
                self.visit_operator(operator, assignment.span)?;
                self.assembler()?.store_index(assignment.span);
                self.assembler()?.pop(assignment.span);
                self.assembler()?.push_unit(assignment.span);
            }
        }

        Ok(())
    }

    fn assign_field(&mut self, target: FieldAccess, assignment: &Assignment) -> Result<(), CompilerError> {
        self.visit(target.expression)?;

        match assignment.operator {
            AssignmentOperator::Assignment => {
                self.visit(assignment.rhs_expression)?;
                self.assembler()?.store_field(target.field, target.span)?;
                self.assembler()?.pop(target.span);
                self.assembler()?.push_unit(target.span);
            }
            operator => {
                self.assembler()?.dup(0, target.span);
                self.assembler()?.load_field(&*target.field, target.span)?;
                self.visit(assignment.rhs_expression)?;
                self.visit_operator(operator, target.span)?;
                self.assembler()?.store_field(target.field, target.span)?;
                self.assembler()?.pop(target.span);
                self.assembler()?.push_unit(target.span);
            }
        }

        Ok(())
    }

    fn assign_ident(&mut self, target: Symbol, assignment: &Assignment) -> Result<(), CompilerError> {
        {
            if let Some(local) = self.context()?.scope_stack().local(target.clone()) {
                let local = local.clone();
                self.assign_local(
                    target,
                    assignment.operator,
                    assignment.rhs_expression,
                    assignment.span,
                    local,
                )
            } else if let Some(upvalue) = self.compiler_stack.resolve_upvalue(target.clone(), 0) {
                self.assign_upvalue(
                    target,
                    assignment.operator,
                    assignment.rhs_expression,
                    assignment.span,
                    upvalue,
                )
            } else {
                Err(CompilerError::UndeclaredVariable((&*target).to_owned()))
            }
        }
    }

    fn assign_upvalue(
        &mut self,
        target: Symbol,
        operator: AssignmentOperator,
        rhs_expression: SyntaxNodeId,
        span: Span,
        upvalue: usize,
    ) -> Result<(), CompilerError> {
        if !self.context()?.upvalues()[upvalue].is_mutable() {
            return Err(CompilerError::ImmutableVariable((&*target).to_owned()));
        }
        match operator {
            AssignmentOperator::Assignment => {
                self.visit(rhs_expression)?;
                self.assembler()?.store_upvalue(upvalue as u8, span);
                self.assembler()?.pop(span);
                self.assembler()?.push_unit(span);
            }
            operator => {
                self.assembler()?.load_upvalue(upvalue as u8, span);
                self.visit(rhs_expression)?;
                self.visit_operator(operator, span)?;
                self.assembler()?.store_upvalue(upvalue as u8, span);
                self.assembler()?.pop(span);
                self.assembler()?.push_unit(span);
            }
        }

        Ok(())
    }

    fn assign_local(
        &mut self,
        target: Symbol,
        operator: AssignmentOperator,
        rhs_expression: SyntaxNodeId,
        span: Span,
        local: ScopeVariable,
    ) -> Result<(), CompilerError> {
        let slot = local.slot as u8;

        if !local.is_mutable() {
            return Err(CompilerError::ImmutableVariable((&*target).to_owned()));
        }

        match operator {
            AssignmentOperator::Assignment => {
                self.visit(rhs_expression)?;
                self.assembler()?.store_local(slot, span);
                self.assembler()?.pop(span);
                self.assembler()?.push_unit(span);
            }
            operator => {
                self.assembler()?.load_local(slot, span);
                self.visit(rhs_expression)?;
                self.visit_operator(operator, span)?;
                self.assembler()?.store_local(slot, span);
                self.assembler()?.pop(span);
                self.assembler()?.push_unit(span);
            }
        }

        Ok(())
    }

    fn visit_operator(&mut self, operator: AssignmentOperator, span: Span) -> Result<(), CompilerError> {
        match operator {
            AssignmentOperator::MulAssignment => self.assembler()?.mul(span),
            AssignmentOperator::DivAssignment => self.assembler()?.div(span),
            AssignmentOperator::AddAssignment => self.assembler()?.add(span),
            AssignmentOperator::SubAssignment => self.assembler()?.sub(span),
            _ => unreachable!(),
        }

        Ok(())
    }
}
