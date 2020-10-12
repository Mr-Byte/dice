use crate::value::Value;
use gc::{Finalize, Gc, GcCell, GcCellRef, GcCellRefMut, Trace};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Trace, Finalize)]
pub struct List(Gc<GcCell<Vec<Value>>>);

impl List {
    pub fn elements(&self) -> GcCellRef<'_, Vec<Value>> {
        self.0.borrow()
    }

    pub fn elements_mut(&self) -> GcCellRefMut<'_, Vec<Value>> {
        self.0.borrow_mut()
    }
}

impl Display for List {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self
            .0
            .borrow()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        write!(fmt, "[{}]", items)
    }
}

impl From<Vec<Value>> for List {
    fn from(value: Vec<Value>) -> Self {
        Self(Gc::new(GcCell::new(value)))
    }
}
