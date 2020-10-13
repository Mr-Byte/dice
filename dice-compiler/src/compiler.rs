use crate::assembler::Assembler;
use crate::{
    compiler_stack::{CompilerContext, CompilerKind, CompilerStack},
    scope_stack::State,
    visitor::NodeVisitor,
};
use dice_core::source::{Source, SourceKind};
use dice_core::{bytecode::Bytecode, constants::EXPORT};
use dice_error::{compiler_error::CompilerError, span::Span};
use dice_syntax::{Parser, SyntaxTree};

#[allow(dead_code)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum CompilationKind {
    Script,
    Module,
}

pub struct Compiler {
    pub(crate) source: Source,
    pub(crate) syntax_tree: SyntaxTree,
    pub(crate) compiler_stack: CompilerStack,
}

impl Compiler {
    fn new(source: Source, syntax_tree: SyntaxTree, kind: CompilationKind) -> Self {
        let compiler_kind = match kind {
            CompilationKind::Script => CompilerKind::Script,
            CompilationKind::Module => CompilerKind::Module,
        };
        let compiler_stack = CompilerStack::new(compiler_kind);

        Self {
            source,
            syntax_tree,
            compiler_stack,
        }
    }

    // TODO: Should this be made an instance function and more logic pushed into the new function?
    pub fn compile(source: Source) -> Result<Bytecode, CompilerError> {
        let syntax_tree = Parser::new(source.source()).parse()?;
        let kind = match source.kind() {
            SourceKind::Module => CompilationKind::Module,
            SourceKind::Script => CompilationKind::Script,
        };
        let mut compiler = Self::new(source, syntax_tree, kind);

        if kind == CompilationKind::Module {
            compiler
                .context()?
                .scope_stack()
                .add_local(EXPORT, State::initialized(false))?;
        }

        compiler.visit(compiler.syntax_tree.root())?;

        if kind == CompilationKind::Module {
            let exports_slot = compiler
                .context()?
                .scope_stack()
                .local(EXPORT)
                .expect("#export should always be defined for modules.")
                .slot as u8;

            emit_bytecode! {
                compiler.context()?.assembler(), Span::new(0..0) => [
                    POP;
                    LOAD_LOCAL exports_slot;
                ]
            };
        }

        let compiler_context = compiler.compiler_stack.pop()?;

        Ok(compiler_context.finish())
    }

    pub(super) fn context(&mut self) -> Result<&mut CompilerContext, CompilerError> {
        self.compiler_stack.top_mut()
    }

    pub(super) fn assembler(&mut self) -> Result<&mut Assembler, CompilerError> {
        Ok(self.compiler_stack.top_mut()?.assembler())
    }
}
