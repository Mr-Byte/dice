use crate::{
    compiler::Compiler,
    scope_stack::{ScopeKind, State},
    visitor::{decl_op::OpKind, FnKind, NodeVisitor},
};
use dice_core::{
    protocol::{
        class::{NEW, SELF},
        ProtocolSymbol,
    },
    value::Symbol,
};
use dice_error::compiler_error::CompilerError;
use dice_syntax::{ClassDecl, FnDecl, OpDecl, SyntaxNode};

impl NodeVisitor<&ClassDecl> for Compiler {
    fn visit(&mut self, node: &ClassDecl) -> Result<(), CompilerError> {
        self.context()?.scope_stack().push_scope(ScopeKind::Block, None);

        let slot = {
            let class_name: Symbol = (&*node.name).into();
            let local = self.context()?.scope_stack().local(class_name).ok_or_else(|| {
                CompilerError::InternalCompilerError(String::from("Class not already declared in scope."))
            })?;

            // NOTE: Check if a class of the given name has already been initialized.
            if let State::Class { ref mut is_initialized } = &mut local.state {
                if *is_initialized {
                    return Err(CompilerError::ItemAlreadyDeclared(node.name.to_owned()));
                }

                *is_initialized = true;
            }

            local.slot as u8
        };

        if let Some(base) = node.base {
            emit_bytecode! {
                self.assembler()?, node.span => [
                    {self.visit(base)?};
                    INHERIT_CLASS &node.name;
                    STORE_LOCAL slot;
                ]
            }
        } else {
            emit_bytecode! {
                self.assembler()?, node.span => [
                    CREATE_CLASS &node.name;
                    STORE_LOCAL slot;
                ]
            }
        }

        for associated_item in node.associated_items.iter().copied() {
            let node = self.syntax_tree.get(associated_item);

            match node {
                SyntaxNode::FnDecl(fn_decl) => {
                    let fn_decl = fn_decl.clone();
                    self.visit_fn(slot, fn_decl)?;
                }
                SyntaxNode::OpDecl(op_decl) => {
                    let op_decl = op_decl.clone();
                    self.visit_op(slot, op_decl)?;
                }
                _ => unreachable!("Unexpected node kind encountered."),
            }
        }

        self.context()?.scope_stack().pop_scope()?;

        Ok(())
    }
}

impl Compiler {
    fn visit_fn(&mut self, slot: u8, fn_decl: FnDecl) -> Result<(), CompilerError> {
        let self_param = fn_decl.args.first().filter(|arg| arg.name == *SELF.get());
        let kind = if let Some(self_param) = self_param {
            // NOTE: If the self parameter has a type annotation, return an error.
            if self_param.type_.is_some() {
                return Err(CompilerError::SelfParameterHasType(self_param.span));
            }

            if fn_decl.name == *NEW.get() {
                FnKind::Constructor
            } else {
                FnKind::Method
            }
        } else {
            if fn_decl.name == *NEW.get() {
                return Err(CompilerError::NewMustHaveSelfReceiver(fn_decl.span));
            }

            FnKind::StaticMethod
        };

        self.visit((&fn_decl, kind))?;

        emit_bytecode! {
            self.assembler()?, fn_decl.span => [
                if matches!(kind, FnKind::Constructor | FnKind::Method) => [
                    STORE_METHOD &*fn_decl.name;
                    LOAD_LOCAL slot;
                ] else [
                    STORE_FIELD &*fn_decl.name;
                    POP;
                    LOAD_LOCAL slot;
                ]
            ]
        }

        Ok(())
    }

    fn visit_op(&mut self, slot: u8, op_decl: OpDecl) -> Result<(), CompilerError> {
        let self_param = op_decl.args.first().filter(|arg| arg.name == *SELF.get());

        if let Some(self_param) = self_param {
            if self_param.type_.is_some() {
                return Err(CompilerError::SelfParameterHasType(self_param.span));
            }
        } else {
            return Err(CompilerError::OperatorMethodHasNoSelf(op_decl.span));
        }

        self.visit((&op_decl, OpKind::Method))?;

        emit_bytecode! {
            self.assembler()?, op_decl.span => [
                STORE_METHOD Self::op_name(&op_decl);
                LOAD_LOCAL slot;
            ]
        }

        Ok(())
    }
}
