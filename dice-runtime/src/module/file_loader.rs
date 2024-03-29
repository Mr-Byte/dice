use dice_compiler::compiler::Compiler;
use dice_core::{
    error::{
        codes::INVALID_SCRIPT_LOCATION,
        context::{Context, ContextKind, MODULE_LOAD_ERROR},
        Error,
    },
    source::{Source, SourceKind},
    tags,
};

use crate::module::{Module, ModuleLoader};
use crate::runtime::RuntimeContext;
use crate::value::Symbol;

#[derive(Default)]
pub struct FileModuleLoader;

impl ModuleLoader for FileModuleLoader {
    fn load_module(&mut self, ctx: &RuntimeContext, name: Symbol) -> Result<Module, Error> {
        let name_str = ctx.interner.resolve(name).expect("symbol not found");

        (|| {
            let path = dunce::canonicalize(name_str)?;
            let working_dir = dunce::canonicalize(std::env::current_dir()?)?;

            // TODO: Have a way to set the modules root as a part of the runtime.
            if !path.starts_with(&working_dir) {
                return Err(Error::new(INVALID_SCRIPT_LOCATION).with_tags(tags! {
                    directory => working_dir.to_string_lossy().to_string()
                }));
            }

            let source = std::fs::read_to_string(&path)?;
            let source = Source::with_path(source, path.to_string_lossy(), SourceKind::Module);
            let module = Compiler::compile_source(source)?;
            let module = Module::new(name.clone(), module);

            Ok(module)
        })()
        .map_err(move |error: Error| {
            error.push_context(Context::new(MODULE_LOAD_ERROR, ContextKind::Note).with_tags(tags! {
                module => name_str
            }))
        })
    }
}
