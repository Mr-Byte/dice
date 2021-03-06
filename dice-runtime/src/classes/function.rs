use crate::module::ModuleLoader;
use dice_core::{
    error::Error,
    protocol::class::NEW,
    runtime::Runtime,
    value::{NativeFn, Value, ValueKind},
};

impl<L> crate::Runtime<L>
where
    L: ModuleLoader,
{
    pub(super) fn register_function(&mut self) {
        let class = self.any_class.derive("Function");

        class.set_method(&NEW, Box::new(construct_function) as NativeFn);

        self.set_value_class(ValueKind::Function, class);
    }
}

fn construct_function(_runtime: &mut dyn Runtime, args: &[Value]) -> Result<Value, Error> {
    match args {
        [_, param, ..] => match param {
            value @ Value::FnNative(_) => Ok(value.clone()),
            value @ Value::FnBound(_) => Ok(value.clone()),
            value @ Value::FnScript(_) => Ok(value.clone()),
            value @ Value::FnClosure(_) => Ok(value.clone()),
            _ => Ok(Value::Null),
        },
        _ => Ok(Value::Null),
    }
}
