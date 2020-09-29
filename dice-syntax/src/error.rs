use super::lexer::Token;
use crate::{Span, SpannedError};

#[derive(thiserror::Error, Debug)]
pub enum SyntaxError {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),

    #[error("Function {0} has too many arguments (max 255).")]
    FnTooManyArguments(String, Span),

    #[error("Anonymous function has too many arguments (max 255).")]
    AnonymousFnTooManyArguments(Span),
}

impl SpannedError for SyntaxError {
    fn span(&self) -> Span {
        match self {
            SyntaxError::UnexpectedToken(token) => token.span(),
            SyntaxError::FnTooManyArguments(_, span) => *span,
            SyntaxError::AnonymousFnTooManyArguments(span) => *span,
        }
    }
}