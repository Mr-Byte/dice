use crate::{runtime::core::Symbol, SyntaxError};

#[derive(thiserror::Error, Debug)]
pub enum CompilerError {
    #[error(transparent)]
    SyntaxError(#[from] SyntaxError),

    #[error("The item {0} has already been declared in this scope.")]
    ItemAlreadyDeclared(Symbol),
    #[error("Encountered undeclared variable {0}.")]
    UndeclaredVariable(Symbol),
    #[error("Cannot assign to immutable variable {0}.")]
    ImmutableVariable(Symbol),
    #[error("Variable {0} has not been initialized.")]
    UnitiailizedVariable(Symbol),
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,

    #[error("The break keyword can only be used inside loops.")]
    InvalidBreak,
    #[error("The continue keyword can only be used inside loops.")]
    InvalidContinue,
    #[error("Loops cannot end with an expression. Try adding ; to the last statement.")]
    InvalidLoopEnding,
    #[error("The return keyword can only be used inside functions.")]
    InvalidReturn,

    #[error("Compilation unit has too many constants.")]
    TooManyConstants,
    #[error("Compilation unit has too many upvalues.")]
    TooManyUpvalues,

    #[error("Internal Compiler Error: {0}")]
    InternalCompilerError(String),
}
