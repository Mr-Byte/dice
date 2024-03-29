use crate::module::ModuleLoader;
use dice_core::{protocol::object::MODULE_CLASS, value::Class};

impl<'gc, L> crate::Runtime<L>
where
    L: ModuleLoader,
{
    pub fn new_module_class(base: &Class) -> Class {
        let class = base.derive(&MODULE_CLASS);

        class
    }
}
