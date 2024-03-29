use std::fmt::Display;

use dice_bytecode::Bytecode;
use dice_core::{
    error::{codes::INTERNAL_COMPILER_ERROR, Error},
    source::Source,
};
use dice_syntax::TypeAnnotation;

use super::{
    assembler::Assembler,
    scope_stack::{ScopeKind, ScopeStack},
    upvalue::UpvalueDescriptor,
};

#[derive(Debug, Clone)]
pub enum CompilerKind {
    Script,
    Module,
    Function { return_type: Option<TypeAnnotation> },
    Method { return_type: Option<TypeAnnotation> },
    Constructor,
}

impl Display for CompilerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerKind::Script => write!(f, "script"),
            CompilerKind::Module => write!(f, "module"),
            CompilerKind::Function { .. } => write!(f, "function"),
            CompilerKind::Method { .. } => write!(f, "function"),
            CompilerKind::Constructor => write!(f, "function"),
        }
    }
}

pub struct CompilerContext {
    kind: CompilerKind,
    assembler: Assembler,
    upvalues: Vec<UpvalueDescriptor>,
    scope_stack: ScopeStack,
    temporary_count: usize,
}

impl CompilerContext {
    pub fn new(kind: CompilerKind) -> Self {
        Self {
            assembler: Assembler::new(),
            scope_stack: ScopeStack::new(ScopeKind::Block),
            upvalues: Vec::new(),
            temporary_count: 0,
            kind,
        }
    }

    pub fn assembler(&mut self) -> &mut Assembler {
        &mut self.assembler
    }

    pub fn scope_stack(&mut self) -> &mut ScopeStack {
        &mut self.scope_stack
    }

    pub fn upvalues(&mut self) -> &mut Vec<UpvalueDescriptor> {
        &mut self.upvalues
    }

    pub fn add_upvalue(&mut self, descriptor: UpvalueDescriptor) -> usize {
        let index = match self.upvalues.iter().position(|upvalue| *upvalue == descriptor) {
            Some(position) => position,
            None => {
                self.upvalues.push(descriptor);
                self.upvalues.len() - 1
            }
        };

        index
    }

    pub fn kind(&self) -> CompilerKind {
        self.kind.clone()
    }

    pub fn temporary_count(&mut self) -> &mut usize {
        &mut self.temporary_count
    }

    pub fn finish(mut self, source: Source) -> Bytecode {
        let slot_count = self.scope_stack.slot_count;
        let upvalue_count = self.upvalues().len();
        self.assembler.generate(slot_count, upvalue_count, source)
    }
}

pub struct CompilerStack {
    stack: Vec<CompilerContext>,
}

impl CompilerStack {
    pub fn new(kind: CompilerKind) -> Self {
        Self {
            stack: vec![CompilerContext::new(kind)],
        }
    }

    pub fn push(&mut self, kind: CompilerKind) {
        self.stack.push(CompilerContext::new(kind));
    }

    pub fn pop(&mut self) -> Result<CompilerContext, Error> {
        self.stack.pop().ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))
    }

    pub fn top_mut(&mut self) -> Result<&mut CompilerContext, Error> {
        self.stack.last_mut().ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))
    }

    pub fn offset(&mut self, offset: usize) -> Option<&mut CompilerContext> {
        if offset >= self.stack.len() {
            return None;
        }

        let index = self.stack.len() - offset - 1;
        self.stack.get_mut(index)
    }

    pub fn resolve_upvalue(&mut self, name: String, depth: usize) -> Option<usize> {
        let parent_local = self.offset(depth + 1)?.scope_stack().local(name.clone());
        let descriptor = match parent_local {
            Some(parent_local) => {
                parent_local.is_captured = true;

                UpvalueDescriptor::ParentLocal {
                    slot: parent_local.slot,
                    is_mutable: parent_local.is_mutable(),
                }
            }
            None => {
                let outer_index = self.resolve_upvalue(name, depth + 1)?;
                let parent = self.offset(depth + 1)?;
                let is_mutable = match parent.upvalues()[outer_index] {
                    UpvalueDescriptor::ParentLocal { is_mutable, .. } | UpvalueDescriptor::Outer { is_mutable, .. } => {
                        is_mutable
                    }
                };

                UpvalueDescriptor::Outer {
                    upvalue_index: outer_index,
                    is_mutable,
                }
            }
        };

        let current = self.offset(depth)?;
        Some(current.add_upvalue(descriptor))
    }
}
