pub mod any;

mod array;
mod bool;
mod class;
mod float;
mod function;
mod int;
mod module;
mod string;
mod unit;

use crate::{module::ModuleLoader, Runtime};

impl<L> Runtime<L>
where
    L: ModuleLoader,
{
    pub(super) fn register_known_types(&mut self) {
        self.register_array();
        self.register_bool();
        self.register_class();
        self.register_float();
        self.register_function();
        self.register_int();
        self.register_string();
        self.register_unit();
    }
}
