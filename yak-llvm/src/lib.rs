use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::FloatPredicate;

use yak_ast::Ast;
use yak_lexer::Lexer;

struct Compiler {}

impl Compiler {
    fn from_ast() {}
    fn from_source() {}

    fn compile_const() {}
    fn compile_expr() {}
    fn compile_fn() {}
    fn compile_let() {}
    fn compile_if() {}
    fn compile() {}
}
