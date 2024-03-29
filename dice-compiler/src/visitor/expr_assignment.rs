use dice_core::{
    error::{
        codes::{CANNOT_REASSIGN_IMMUTABLE_VARIABLE, INVALID_ASSIGNMENT_TARGET, VARIABLE_NOT_DECLARED},
        Error,
    },
    span::Span,
    tags,
};
use dice_syntax::{Assignment, AssignmentOperator, FieldAccess, Index, SyntaxNode, SyntaxNodeId};

use crate::{compiler::Compiler, scope_stack::ScopeVariable};

use super::NodeVisitor;

impl NodeVisitor<&Assignment> for Compiler {
    fn visit(&mut self, assignment: &Assignment) -> Result<(), Error> {
        let lhs = self.syntax_tree.get(assignment.lhs_expression);

        match lhs {
            SyntaxNode::LitIdent(lit_ident) => {
                let target: String = (&*lit_ident.identifier).into();
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
            _ => Err(Error::new(INVALID_ASSIGNMENT_TARGET).with_span(lhs.span())),
        }
    }
}

impl Compiler {
    fn assign_index(&mut self, target: Index, assignment: &Assignment) -> Result<(), Error> {
        self.visit(target.expression)?;
        self.visit(target.index_expression)?;

        match assignment.operator {
            AssignmentOperator::Assignment => {
                emit_bytecode! {
                    self.assembler()?, assignment.span => [
                        {self.visit(assignment.rhs_expression)?};
                        ASSIGN_INDEX;
                    ]
                }
            }
            operator => {
                emit_bytecode! {
                    self.assembler()?, assignment.span => [
                        DUP 1;
                        DUP 1;
                        LOAD_INDEX;
                        {self.visit(assignment.rhs_expression)?};
                        {self.visit_operator(operator, assignment.span)?};
                        ASSIGN_INDEX;
                    ]
                }
            }
        }

        Ok(())
    }

    fn assign_field(&mut self, target: FieldAccess, assignment: &Assignment) -> Result<(), Error> {
        self.visit(target.expression)?;

        match assignment.operator {
            AssignmentOperator::Assignment => {
                emit_bytecode! {
                    self.assembler()?, target.span => [
                        {self.visit(assignment.rhs_expression)?};
                        STORE_FIELD target.field;
                    ]
                }
            }
            operator => {
                emit_bytecode! {
                    self.assembler()?, target.span => [
                        DUP 0;
                        LOAD_FIELD &*target.field;
                        {self.visit(assignment.rhs_expression)?};
                        {self.visit_operator(operator, target.span)?};
                        ASSIGN_FIELD target.field;
                    ]
                }
            }
        }

        Ok(())
    }

    fn assign_ident(&mut self, target: String, assignment: &Assignment) -> Result<(), Error> {
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
                Err(Error::new(VARIABLE_NOT_DECLARED)
                    .with_span(assignment.span)
                    .with_tags(tags! {
                        name => target
                    }))
            }
        }
    }

    fn assign_upvalue(
        &mut self,
        target: String,
        operator: AssignmentOperator,
        rhs_expression: SyntaxNodeId,
        span: Span,
        upvalue: usize,
    ) -> Result<(), Error> {
        if !self.context()?.upvalues()[upvalue].is_mutable() {
            return Err(Error::new(CANNOT_REASSIGN_IMMUTABLE_VARIABLE)
                .with_span(span)
                .with_tags(tags! {
                    name => target
                }));
        }
        match operator {
            AssignmentOperator::Assignment => {
                emit_bytecode! {
                    self.assembler()?, span => [
                        {self.visit(rhs_expression)?};
                        ASSIGN_UPVALUE upvalue as u8;
                    ]
                }
            }
            operator => {
                emit_bytecode! {
                    self.assembler()?, span => [
                        LOAD_UPVALUE upvalue as u8;
                        {self.visit(rhs_expression)?};
                        {self.visit_operator(operator, span)?};
                        ASSIGN_UPVALUE upvalue as u8;
                    ]
                }
            }
        }

        Ok(())
    }

    fn assign_local(
        &mut self,
        target: String,
        operator: AssignmentOperator,
        rhs_expression: SyntaxNodeId,
        span: Span,
        local: ScopeVariable,
    ) -> Result<(), Error> {
        let slot = local.slot as u8;

        if !local.is_mutable() {
            return Err(Error::new(CANNOT_REASSIGN_IMMUTABLE_VARIABLE)
                .with_span(span)
                .with_tags(tags! {
                    name => target
                }));
        }

        match operator {
            AssignmentOperator::Assignment => {
                emit_bytecode! {
                    self.assembler()?, span => [
                        {self.visit(rhs_expression)?};
                        ASSIGN_LOCAL slot;
                    ]
                }
            }
            operator => {
                emit_bytecode! {
                    self.assembler()?, span => [
                        LOAD_LOCAL slot;
                        {self.visit(rhs_expression)?};
                        {self.visit_operator(operator, span)?};
                        ASSIGN_LOCAL slot;
                    ]
                }
            }
        }

        Ok(())
    }

    fn visit_operator(&mut self, operator: AssignmentOperator, span: Span) -> Result<(), Error> {
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
