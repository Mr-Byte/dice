use super::{error::ErrorKind, ParseResult, Parser};
use crate::syntax::{lexer::TokenKind, BinaryOperator, ParserError, RangeOperator, SyntaxTree, UnaryOperator};

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> ParseResult {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_coalesce()?;

        if self.next_token.is_kind(TokenKind::Assign) {
            if let SyntaxTree::Literal(crate::syntax::Literal::Identifier(variable), _) = expression {
                self.next();
                let rhs = self.parse_coalesce()?;
                let span_end = self.current_token.span.clone();
                expression = SyntaxTree::VariableAssignment(variable.clone(), Box::new(rhs), span_start.clone() + span_end);
            } else {
                return Err(ParserError::new(ErrorKind::InvlaidAssignmentTarget, Some(expression.span().clone())));
            }
        }

        Ok(expression)
    }

    fn parse_coalesce(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_range()?;

        while self.next_token.is_kind(TokenKind::Coalesce) {
            self.next();
            let operator = self.current_token.clone();
            let rhs = self.parse_range()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Binary(
                BinaryOperator::Coalesce(operator.span.clone()),
                Box::new(expression),
                Box::new(rhs),
                span_start.clone() + span_end,
            );
        }

        Ok(expression)
    }

    fn parse_range(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_lazy_or()?;

        while self.next_token.is_any_kind(&[TokenKind::InclusiveRange, TokenKind::ExclusiveRange]) {
            self.next();
            let operator = self.current_token.clone();
            let operator = match operator.kind {
                TokenKind::InclusiveRange => RangeOperator::Inclusive(operator.span.clone()),
                TokenKind::ExclusiveRange => RangeOperator::Exclusive(operator.span.clone()),
                _ => unreachable!(),
            };

            let rhs = self.parse_lazy_or()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Range(operator, Box::new(expression), Box::new(rhs), span_start.clone() + span_end);
        }

        Ok(expression)
    }

    fn parse_lazy_or(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_lazy_and()?;

        while self.next_token.is_kind(TokenKind::LazyOr) {
            self.next();
            let operator = self.current_token.clone();
            let rhs = self.parse_lazy_and()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Binary(
                BinaryOperator::LogicalOr(operator.span.clone()),
                Box::new(expression),
                Box::new(rhs),
                span_start.clone() + span_end,
            );
        }

        Ok(expression)
    }

    fn parse_lazy_and(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_comparison()?;

        while self.next_token.is_kind(TokenKind::LazyAnd) {
            self.next();
            let operator = self.current_token.clone();
            let rhs = self.parse_comparison()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Binary(
                BinaryOperator::LogicalAnd(operator.span.clone()),
                Box::new(expression),
                Box::new(rhs),
                span_start.clone() + span_end,
            );
        }

        Ok(expression)
    }

    fn parse_comparison(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_additive()?;

        while self.next_token.is_any_kind(&[
            TokenKind::Equal,
            TokenKind::NotEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            self.next();
            let operator = self.current_token.clone();
            let operator = match operator.kind {
                TokenKind::Equal => BinaryOperator::Equals(operator.span.clone()),
                TokenKind::NotEqual => BinaryOperator::NotEquals(operator.span.clone()),
                TokenKind::Greater => BinaryOperator::GreaterThan(operator.span.clone()),
                TokenKind::GreaterEqual => BinaryOperator::GreaterThanOrEquals(operator.span.clone()),
                TokenKind::Less => BinaryOperator::LessThan(operator.span.clone()),
                TokenKind::LessEqual => BinaryOperator::LessThanOrEquals(operator.span.clone()),
                _ => unreachable!(),
            };

            let rhs = self.parse_additive()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Binary(operator, Box::new(expression), Box::new(rhs), span_start.clone() + span_end);
        }

        Ok(expression)
    }

    fn parse_additive(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_multiplicative()?;

        while self.next_token.is_any_kind(&[TokenKind::Plus, TokenKind::Minus]) {
            self.next();
            let operator = self.current_token.clone();
            let operator = match operator.kind {
                TokenKind::Plus => BinaryOperator::Add(operator.span.clone()),
                TokenKind::Minus => BinaryOperator::Subtract(operator.span.clone()),
                _ => unreachable!(),
            };

            let rhs = self.parse_multiplicative()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Binary(operator, Box::new(expression), Box::new(rhs), span_start.clone() + span_end);
        }

        Ok(expression)
    }

    fn parse_multiplicative(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_dice_roll()?;

        while self.next_token.is_any_kind(&[TokenKind::Star, TokenKind::Slash, TokenKind::Remainder]) {
            self.next();
            let operator = self.current_token.clone();
            let operator = match operator.kind {
                TokenKind::Star => BinaryOperator::Multiply(operator.span.clone()),
                TokenKind::Slash => BinaryOperator::Divide(operator.span.clone()),
                TokenKind::Remainder => BinaryOperator::Remainder(operator.span.clone()),
                _ => unreachable!(),
            };

            let rhs = self.parse_dice_roll()?;
            let span_end = self.current_token.span.clone();
            expression = SyntaxTree::Binary(operator, Box::new(expression), Box::new(rhs), span_start.clone() + span_end);
        }

        Ok(expression)
    }

    fn parse_dice_roll(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        let mut expression = self.parse_unary()?;

        while self.next_token.is_any_kind(&[TokenKind::DiceRoll]) {
            self.next();
            let operator = self.current_token.clone();
            let operator = match operator.kind {
                TokenKind::DiceRoll => BinaryOperator::DiceRoll(operator.span.clone()),
                _ => unreachable!(),
            };

            let rhs = self.parse_unary()?;
            let span_end = self.current_token.span.clone();

            expression = SyntaxTree::Binary(operator, Box::new(expression), Box::new(rhs), span_start.clone() + span_end);
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> ParseResult {
        let span_start = self.current_token.span.clone();
        if self.next_token.is_any_kind(&[TokenKind::Not, TokenKind::Minus, TokenKind::DiceRoll]) {
            self.next();
            let operator = self.current_token.clone();
            let operator = match operator.kind {
                TokenKind::Not => UnaryOperator::Not(operator.span),
                TokenKind::Minus => UnaryOperator::Negate(operator.span),
                TokenKind::DiceRoll => UnaryOperator::DiceRoll(operator.span),
                _ => unreachable!(),
            };
            let expression = self.parse_unary()?;
            let span_end = self.current_token.span.clone();

            Ok(SyntaxTree::Unary(operator, Box::new(expression), span_start + span_end))
        } else {
            self.parse_accessor()
        }
    }
}
