use dice_core::{
    error::{codes::NEW_MUST_CALL_SUPER_FROM_SUBCLASS, Error},
    protocol::class::SELF,
    span::Span,
};
use dice_syntax::{Block, FnArg, SyntaxNode};

use crate::{
    compiler::Compiler,
    compiler_stack::CompilerKind,
    scope_stack::{ScopeKind, State},
    visitor::ClassKind,
};

use super::NodeVisitor;

impl NodeVisitor<&Block> for Compiler {
    fn visit(&mut self, block: &Block) -> Result<(), Error> {
        for expression in block.expressions.iter() {
            self.visit(*expression)?;
            self.assembler()?.pop(block.span);
        }

        match block.trailing_expression {
            Some(trailing_expression) => {
                self.visit(trailing_expression)?;
            }
            None => self.assembler()?.push_unit(block.span),
        }

        Ok(())
    }
}

pub enum BlockKind {
    Block,
    Loop,
}

impl NodeVisitor<(&Block, BlockKind)> for Compiler {
    fn visit(&mut self, (block, kind): (&Block, BlockKind)) -> Result<(), Error> {
        self.context()?.scope_stack().push_scope(ScopeKind::Block, None);
        self.scan_item_decls(block)?;
        self.visit(block)?;

        // NOTE: If in context of a loop, pop the last value off the stack.
        if let BlockKind::Loop = kind {
            self.assembler()?.pop(block.span);
        }

        self.visit_close_upvalues(block)?;
        self.context()?.scope_stack().pop_scope()?;

        Ok(())
    }
}

pub enum FunctionBlockKind<'args> {
    Function(&'args [FnArg]),
    Method(&'args [FnArg]),
    Constructor(&'args [FnArg], ClassKind),
}

impl<'args> FunctionBlockKind<'args> {
    fn args(&self) -> &'args [FnArg] {
        match self {
            FunctionBlockKind::Function(args)
            | FunctionBlockKind::Method(args)
            | FunctionBlockKind::Constructor(args, ..) => *args,
        }
    }
}

impl<'args> NodeVisitor<(&Block, FunctionBlockKind<'args>)> for Compiler {
    fn visit(&mut self, (block, kind): (&Block, FunctionBlockKind<'args>)) -> Result<(), Error> {
        if let FunctionBlockKind::Constructor(_, ClassKind::Derived) = kind {
            self.assert_super_call(block)?;
        }

        self.context()?.scope_stack().push_scope(ScopeKind::Block, None);
        self.visit_args(&kind, kind.args())?;
        self.scan_item_decls(block)?;
        self.visit(block)?;
        self.visit_close_upvalues(block)?;

        if let FunctionBlockKind::Function(_) | FunctionBlockKind::Method(_) = kind {
            self.visit_return(block.span)?
        } else if let FunctionBlockKind::Constructor(..) = kind {
            // NOTE: If in context of a constructor, pop the last value, load self, return.
            let local_slot = self
                .context()?
                .scope_stack()
                .local(SELF)
                .expect("Methods should always have a self.")
                .slot as u8;

            emit_bytecode! {
                self.assembler()?, block.span => [
                    POP;
                    LOAD_LOCAL local_slot;
                    RET;
                ]
            }
        }

        self.context()?.scope_stack().pop_scope()?;

        Ok(())
    }
}

impl Compiler {
    fn assert_super_call(&self, block: &Block) -> Result<(), Error> {
        if let Some(expr) = block.expressions.iter().chain(block.trailing_expression.iter()).next() {
            if let SyntaxNode::SuperCall(_) = self.syntax_tree.get(*expr) {
                return Ok(());
            }
        }

        // TODO: Get the span of the function and first line.
        Err(Error::new(NEW_MUST_CALL_SUPER_FROM_SUBCLASS).with_span(block.span))
    }

    fn visit_args(&mut self, kind: &FunctionBlockKind, args: &[FnArg]) -> Result<(), Error> {
        // NOTE: The calling convention uses the first parameter as self in methods, but for functions it's inaccessible.
        if let FunctionBlockKind::Function(_) = kind {
            self.context()?.scope_stack().add_local(
                "",
                State::Local {
                    is_mutable: false,
                    is_initialized: true,
                },
            )?;
        }

        for arg in args {
            let slot = self.context()?.scope_stack().add_local(
                arg.name.clone(),
                State::Local {
                    is_mutable: false,
                    is_initialized: true,
                },
            )? as u8;

            if let Some(type_) = &arg.type_ {
                emit_bytecode! {
                    self.assembler()?, arg.span => [
                        {self.visit(&type_.name)?};
                        if type_.is_nullable => [
                            ASSERT_TYPE_OR_NULL_FOR_LOCAL slot;
                        ] else [
                            ASSERT_TYPE_FOR_LOCAL slot;
                        ]
                    ]
                }
            }
        }

        Ok(())
    }

    fn visit_close_upvalues(&mut self, block: &Block) -> Result<(), Error> {
        let scope = self.context()?.scope_stack().top_mut()?;

        for variable in scope.variables.clone() {
            if variable.is_captured {
                self.context()?
                    .assembler()
                    .close_upvalue(variable.slot as u8, block.span);
            }
        }

        Ok(())
    }

    pub(super) fn visit_return(&mut self, span: Span) -> Result<(), Error> {
        if let CompilerKind::Function {
            return_type: Some(return_type),
        }
        | CompilerKind::Method {
            return_type: Some(return_type),
        } = self.context()?.kind()
        {
            emit_bytecode! {
                self.assembler()?, span => [
                    {self.visit(&return_type.name)?};
                    if return_type.is_nullable => [
                        ASSERT_TYPE_OR_NULL_AND_RETURN;
                    ] else [
                        ASSERT_TYPE_AND_RETURN;
                    ]
                ]
            }
        } else {
            self.assembler()?.ret(span)
        }

        Ok(())
    }
}
