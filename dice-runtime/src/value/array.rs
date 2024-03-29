use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

use gc_arena::{Collect, Gc, lock::RefLock};

use crate::{
    runtime::RuntimeContext,
    value::{Object, Value},
};

#[derive(Clone, PartialEq, Collect)]
#[collect(no_drop)]
pub struct Array<'gc> {
    inner: Gc<'gc, ArrayInner<'gc>>,
}

impl<'gc> Array<'gc> {
    pub fn elements(&self) -> Ref<'gc, [Value]> {
        Ref::map(self.inner.array.borrow(), |array| array.as_slice())
    }

    pub fn elements_mut(&self, ctx: &RuntimeContext<'gc>) -> RefMut<'gc, [Value]> {
        RefMut::map(self.inner.array.borrow_mut(ctx.mutation), |array| array.as_mut_slice())
    }

    pub fn push(&self, ctx: &RuntimeContext<'gc>, value: Value<'gc>) {
        self.inner.array.borrow_mut(ctx.mutation).push(value)
    }

    pub fn pop(&self, ctx: &RuntimeContext<'gc>) -> Option<Value<'gc>> {
        self.inner.array.borrow_mut(ctx.mutation).pop()
    }

    pub fn from_vec(ctx: &RuntimeContext<'gc>, value: Vec<Value<'gc>>) -> Self {
        Self {
            inner: Gc::new(
                ctx.mutation,
                ArrayInner {
                    array: Gc::new(ctx.mutation, RefLock::new(value)),
                    object: Object::new(ctx, None),
                },
            ),
        }
    }
}

// impl Display for Array<'_> {
//     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let items = self
//             .inner
//             .array
//             .borrow()
//             .iter()
//             .map(|value| value.to_string())
//             .collect::<Vec<_>>()
//             .join(", ");

//         write!(fmt, "[{}]", items)
//     }
// }

impl<'gc> Deref for Array<'gc> {
    type Target = Object<'gc>;

    fn deref(&self) -> &Self::Target {
        &self.inner.object
    }
}

#[derive(Clone, PartialEq, Collect)]
#[collect(no_drop)]
struct ArrayInner<'gc> {
    array: Gc<'gc, RefLock<Vec<Value<'gc>>>>,
    object: Object<'gc>,
}
