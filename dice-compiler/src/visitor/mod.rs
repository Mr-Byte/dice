mod decl_fn;
mod decl_var;
mod expr_assignment;
mod expr_binary_op;
mod expr_block;
mod expr_break;
mod expr_continue;
mod expr_field_access;
mod expr_fn_call;
mod expr_if;
mod expr_return;
mod expr_unary_op;
mod expr_while;
mod literal_anonymous_fn;
mod literal_bool;
mod literal_float;
mod literal_int;
mod literal_list;
mod literal_none;
mod literal_object;
mod literal_string;
mod literal_unit;
mod literal_variable;
mod syntax_node;

use crate::error::CompilerError;
pub use expr_block::BlockKind;

pub(super) trait NodeVisitor<T> {
    fn visit(&mut self, node: T) -> Result<(), CompilerError>;
}
