use super::NodeVisitor;
use crate::compiler::Compiler;
use dice_core::protocol::operator::{DICE_ROLL, RANGE_EXCLUSIVE, RANGE_INCLUSIVE};
use dice_core::protocol::ProtocolSymbol;
use dice_error::{compiler_error::CompilerError, span::Span};
use dice_syntax::{Binary, BinaryOperator, SyntaxNodeId};

impl NodeVisitor<&Binary> for Compiler {
    fn visit(
        &mut self,
        Binary {
            operator,
            lhs_expression,
            rhs_expression,
            span,
        }: &Binary,
    ) -> Result<(), CompilerError> {
        match operator {
            BinaryOperator::LogicalAnd => self.logical_and(*lhs_expression, *rhs_expression, *span)?,
            BinaryOperator::LogicalOr => self.logical_or(*lhs_expression, *rhs_expression, *span)?,
            BinaryOperator::Pipeline => self.pipeline(*lhs_expression, *rhs_expression, *span)?,
            BinaryOperator::DiceRoll => self.dice_roll(*lhs_expression, *rhs_expression, *span)?,
            BinaryOperator::RangeInclusive => self.range_inclusive(*lhs_expression, *rhs_expression, *span)?,
            BinaryOperator::RangeExclusive => self.range_exclusive(*lhs_expression, *rhs_expression, *span)?,
            BinaryOperator::Coalesce => self.coalesce(*lhs_expression, *rhs_expression, *span)?,
            operator => self.binary(*operator, *lhs_expression, *rhs_expression, *span)?,
        }

        Ok(())
    }
}

impl Compiler {
    fn logical_and(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        let short_circuit_jump;

        emit_bytecode! {
            self.assembler()?, span => [
                { self.visit(lhs_expression)? };
                DUP 0;
                ASSERT_BOOL;
                JUMP_IF_FALSE -> short_circuit_jump;
                POP;
                { self.visit(rhs_expression)? };
                ASSERT_BOOL;
            ]
        }

        self.compiler_stack
            .top_mut()?
            .assembler()
            .patch_jump(short_circuit_jump);

        Ok(())
    }
}

impl Compiler {
    fn logical_or(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        let short_circuit_jump;

        emit_bytecode! {
            self.assembler()?, span => [
                { self.visit(lhs_expression)? };
                DUP 0;
                ASSERT_BOOL;
                JUMP_IF_TRUE -> short_circuit_jump;
                POP;
                { self.visit(rhs_expression)? };
                ASSERT_BOOL;
            ]
        }

        self.compiler_stack
            .top_mut()?
            .assembler()
            .patch_jump(short_circuit_jump);

        Ok(())
    }

    fn pipeline(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        self.visit(rhs_expression)?;
        self.visit(lhs_expression)?;
        self.context()?.assembler().call(1, span);

        Ok(())
    }

    fn dice_roll(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        self.context()?.assembler().load_global(DICE_ROLL.get(), span)?;
        self.visit(lhs_expression)?;
        self.visit(rhs_expression)?;
        self.context()?.assembler().call(2, span);

        Ok(())
    }

    fn range_inclusive(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        self.context()?.assembler().load_global(RANGE_INCLUSIVE.get(), span)?;
        self.visit(lhs_expression)?;
        self.visit(rhs_expression)?;
        self.context()?.assembler().call(2, span);

        Ok(())
    }

    fn range_exclusive(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        self.context()?.assembler().load_global(RANGE_EXCLUSIVE.get(), span)?;
        self.visit(lhs_expression)?;
        self.visit(rhs_expression)?;
        self.context()?.assembler().call(2, span);

        Ok(())
    }

    fn coalesce(
        &mut self,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        self.visit(lhs_expression)?;
        let coalesce_jump;
        emit_bytecode! {
            self.context()?.assembler(), span => [
                DUP 0;
                PUSH_NULL;
                EQ;
                JUMP_IF_FALSE -> coalesce_jump;
                POP;
            ]
        }

        self.visit(rhs_expression)?;
        self.context()?.assembler().patch_jump(coalesce_jump);

        Ok(())
    }

    fn binary(
        &mut self,
        operator: BinaryOperator,
        lhs_expression: SyntaxNodeId,
        rhs_expression: SyntaxNodeId,
        span: Span,
    ) -> Result<(), CompilerError> {
        self.visit(lhs_expression)?;
        self.visit(rhs_expression)?;

        match operator {
            BinaryOperator::Multiply => self.context()?.assembler().mul(span),
            BinaryOperator::Divide => self.context()?.assembler().div(span),
            BinaryOperator::Remainder => self.context()?.assembler().rem(span),
            BinaryOperator::Add => self.context()?.assembler().add(span),
            BinaryOperator::Subtract => self.context()?.assembler().sub(span),
            BinaryOperator::GreaterThan => self.context()?.assembler().gt(span),
            BinaryOperator::LessThan => self.context()?.assembler().lt(span),
            BinaryOperator::GreaterThanEquals => self.context()?.assembler().gte(span),
            BinaryOperator::LessThanEquals => self.context()?.assembler().lte(span),
            BinaryOperator::Equals => self.context()?.assembler().eq(span),
            BinaryOperator::NotEquals => self.context()?.assembler().neq(span),
            BinaryOperator::Is => self.context()?.assembler().is(span),
            _ => unreachable!(),
        }

        Ok(())
    }
}
