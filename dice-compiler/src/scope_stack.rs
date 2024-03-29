use dice_core::error::{codes::INTERNAL_COMPILER_ERROR, Error};

#[derive(Clone)]
pub struct ScopeVariable {
    pub name: String,
    pub slot: usize,
    pub is_captured: bool,
    pub state: State,
}

impl ScopeVariable {
    pub fn is_mutable(&self) -> bool {
        matches!(self.state, State::Local { is_mutable, .. } if is_mutable)
    }

    pub fn is_initialized(&self) -> bool {
        matches!(
            self.state,
            State::Local { is_initialized, .. }  | State::Function { is_initialized }  | State::Class { is_initialized }
            if is_initialized
        )
    }
}

#[derive(Clone)]
pub enum State {
    Local { is_mutable: bool, is_initialized: bool },
    Function { is_initialized: bool },
    Class { is_initialized: bool },
}

impl State {
    pub fn initialized(is_mutable: bool) -> Self {
        State::Local {
            is_initialized: true,
            is_mutable,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ScopeKind {
    Block,
    Loop,
}

#[derive(Clone, Default)]
pub struct CallContext {
    pub depth: usize,
    pub exit_points: Vec<usize>,
}

#[derive(Clone)]
pub struct ScopeContext {
    pub depth: usize,
    pub kind: ScopeKind,
    pub entry_point: Option<usize>,
    pub exit_points: Vec<usize>,
    pub variables: Vec<ScopeVariable>,
    pub call_context: CallContext,
    slot_count: usize,
}

impl Default for ScopeContext {
    fn default() -> Self {
        Self {
            depth: 0,
            kind: ScopeKind::Block,
            entry_point: None,
            exit_points: Vec::new(),
            variables: Vec::new(),
            call_context: Default::default(),
            slot_count: 0,
        }
    }
}

pub struct ScopeStack {
    stack: Vec<ScopeContext>,
    pub slot_count: usize,
}

impl ScopeStack {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            stack: vec![ScopeContext {
                kind,
                ..Default::default()
            }],
            slot_count: 0,
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind, entry_point: Option<usize>) {
        self.stack.push(ScopeContext {
            kind,
            depth: self.stack.len(),
            entry_point,
            ..Default::default()
        });
    }

    pub fn pop_scope(&mut self) -> Result<ScopeContext, Error> {
        self.stack.pop().ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))
    }

    pub fn in_context_of(&self, kind: ScopeKind) -> bool {
        self.first_of_kind(kind).is_some()
    }

    pub fn add_local(&mut self, name: impl Into<String>, state: State) -> Result<usize, Error> {
        self.add_local_impl(name.into(), state)
    }

    fn add_local_impl(&mut self, name: String, state: State) -> Result<usize, Error> {
        self.top_mut()?.slot_count += 1;

        let slot_count = self.stack.iter().rev().map(|scope| scope.slot_count).sum();
        let slot = slot_count - 1;
        let local = ScopeVariable {
            name,
            slot,
            is_captured: false,
            state,
        };

        self.top_mut()?.variables.push(local);

        if slot_count > self.slot_count {
            self.slot_count = slot_count;
        }

        Ok(slot)
    }

    pub fn local(&mut self, name: impl Into<String>) -> Option<&mut ScopeVariable> {
        let name = name.into();
        self.stack
            .iter_mut()
            .rev()
            .flat_map(|scope| scope.variables.iter_mut().rev())
            .find(|var| var.name == name)
    }

    pub fn top_mut(&mut self) -> Result<&mut ScopeContext, Error> {
        self.stack.last_mut().ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))
    }

    /// Push the bytecode location of an exit point to the inner most loop's scope, to later be patched.
    pub fn add_loop_exit_point(&mut self, exit_point: usize) -> Result<(), Error> {
        let scope = self
            .first_of_kind_mut(ScopeKind::Loop)
            .ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))?;

        scope.exit_points.push(exit_point);

        Ok(())
    }

    /// Get the entry point of the first scope to match the specified kind.
    pub fn entry_point(&mut self, kind: ScopeKind) -> Result<usize, Error> {
        let scope = self
            .first_of_kind(kind)
            .ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))?;

        scope
            .entry_point
            .clone()
            .ok_or_else(|| Error::new(INTERNAL_COMPILER_ERROR))
    }

    fn first_of_kind(&self, kind: ScopeKind) -> Option<&ScopeContext> {
        self.stack.iter().rev().find(|scope| scope.kind == kind)
    }

    fn first_of_kind_mut(&mut self, kind: ScopeKind) -> Option<&mut ScopeContext> {
        self.stack.iter_mut().rev().find(|scope| scope.kind == kind)
    }
}
