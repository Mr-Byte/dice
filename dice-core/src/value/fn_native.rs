use crate::{runtime::Runtime, value::Value};
use dice_error::runtime_error::RuntimeError;
use std::fmt::{Debug, Display};
use std::rc::Rc;

pub type NativeFn = Rc<dyn Fn(&mut dyn Runtime, &[Value]) -> Result<Value, RuntimeError>>;

#[derive(Clone, gc::Trace, gc::Finalize)]
pub struct FnNative(#[unsafe_ignore_trace] NativeFn);

impl FnNative {
    pub fn new(native_fn: NativeFn) -> Self {
        Self(native_fn)
    }

    #[inline]
    pub fn call(&self, runtime: &mut dyn Runtime, args: &[Value]) -> Result<Value, RuntimeError> {
        self.0(runtime, args)
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
