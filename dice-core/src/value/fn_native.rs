use crate::{error::Error, runtime::Runtime, value::Value};
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

pub type NativeFn = Box<dyn Fn(&mut dyn Runtime, &[Value]) -> Result<Value, Error>>;

#[derive(Clone)]
pub struct FnNative {
    inner: Rc<NativeFn>,
}

impl FnNative {
    pub fn new(native_fn: NativeFn) -> Self {
        Self {
            inner: Rc::new(native_fn),
        }
    }

    #[inline]
    pub fn call(&self, runtime: &mut dyn Runtime, args: &[Value]) -> Result<Value, Error> {
        (*self.inner)(runtime, args)
    }
}

impl Display for FnNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native_fn")
    }
}

impl Debug for FnNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native_fn")
    }
}
