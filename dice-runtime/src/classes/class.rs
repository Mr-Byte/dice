use crate::module::ModuleLoader;
use dice_core::value::{Class, Value};
use dice_core::{runtime::Runtime, value::ValueKind};
use dice_error::runtime_error::RuntimeError;
use std::rc::Rc;

pub fn register(runtime: &mut crate::Runtime<impl ModuleLoader>, base: Class) {
    let mut class = runtime.new_class("Class", Some(base)).unwrap();
    runtime.known_types.insert(ValueKind::Class, class.class());

    class.register_native_method("name", Rc::new(name));
}

fn name(_: &mut dyn Runtime, args: &[Value]) -> Result<Value, RuntimeError> {
    match args {
        [Value::Class(class), ..] => Ok(Value::with_string(class.name())),
        _ => Ok(Value::Null),
    }
}
