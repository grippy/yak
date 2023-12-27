#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::support::load_library_permanently;
use inkwell::types::{IntType, PointerType};
use inkwell::values::{
    AnyValue, ArrayValue, AsValueRef, BasicMetadataValueEnum, BasicValueEnum, CallSiteValue,
    PointerValue,
};
use inkwell::{AddressSpace, OptimizationLevel};

use std::env;
use std::process::Command;

const SOURCE_FILE_NAME: &str = "helloworld.ll";
const EXEC_FILE_NAME: &str = "helloworld";
const FUNC_MAIN: &str = "main";
const FUNC_ENTRY: &str = "enter";
const FUNC_PRINTF: &str = "printf";
const FUNC_SPRINTF: &str = "sprintf";

// get env variable w/ default
fn get_env(key: &'static str, default: &'static str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
    ) -> Self {
        Compiler {
            context,
            builder,
            module,
        }
    }

    pub fn compile(&self) {
        // create fn "main" and fill it in
        // so it calls printf with the hello_str
        let stdlib_path = "/usr/local/rustup/toolchains/1.74-aarch64-unknown-linux-gnu/lib/libstd-173ad5c1e159cc01.so";
        load_library_permanently(stdlib_path);

        let i64_type = self.context.i64_type();
        let func_type =
            i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let func_main = self.module.add_function(FUNC_MAIN, func_type, None);
        let basic_block = self.context.append_basic_block(func_main, FUNC_ENTRY);
        self.builder.position_at_end(basic_block);

        // generate printf and sprintf functions
        self.generate_printf_func();
        self.generate_sprintf_func();

        // define `print_int` from the yak_std library
        let print_int_value = i64_type.const_int(42, false);
        let print_int_type = self
            .context
            .void_type()
            .fn_type(&[self.context.i64_type().into()], false);
        let print_int = self.module.add_function("print_int", print_int_type, None);
        self.builder
            .build_call(print_int, &[print_int_value.into()], "tmp")
            .try_as_basic_value();

        // set hello string from env or use default
        let mut hello_str = get_env("HELLO_STR", "hello, world!");
        if !hello_str.ends_with("\n") {
            hello_str = format!("{}\n", hello_str);
        }
        // generate fmt str
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let buffer = self.builder.build_array_alloca(
            self.context.i8_type(),
            self.context.i64_type().const_int(100, false),
            "buffer",
        );
        let buffer_ptr = self
            .builder
            .build_pointer_cast(buffer, i8_ptr_type, "buffer_ptr");

        let fmt_str_ptr = self
            .builder
            .build_global_string_ptr("DEBUG: %s\0", "fmt_string");

        // Convert hello_str in to global string ptr
        let hello_str_ptr = self
            .builder
            .build_global_string_ptr(hello_str.as_str(), "hello_str");

        // call sprintf
        let sprintf_args = &[
            buffer_ptr.into(),
            fmt_str_ptr.as_pointer_value().into(),
            hello_str_ptr.as_pointer_value().into(),
        ];
        let results = self.generate_sprintf_call(sprintf_args);
        // println!("{:?}", results);

        // convert buffer to point cast
        let buffer_ptr =
            self.builder
                .build_pointer_cast(buffer, i8_ptr_type, "buffer_pointer_cast");

        // call printf function w/ buffer_ptr
        let i64_type = self.generate_printf_call(&[buffer_ptr.into()]);

        self.builder
            .build_return(Some(&i64_type.const_int(0, false)));

        // verify module and print it to file
        match self.module.verify() {
            Ok(_) => {
                let _result = self.module.print_to_file(SOURCE_FILE_NAME);
                println!("Generated module: {}", SOURCE_FILE_NAME);
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }

        self.build_executable();

        // self.execute()
    }

    fn build_executable(&self) {
        let clang_bin = "clang";
        let output_result = Command::new(&clang_bin)
            .arg(SOURCE_FILE_NAME)
            .arg("/Users/gmelton/work/yak/target/debug/libyak_std.a")
            .arg("-v")
            .arg("-o")
            .arg(EXEC_FILE_NAME)
            .output();
        match output_result {
            Ok(output) => {
                let status = output.status;
                if status.success() {
                    println!("Generated executable: {}", EXEC_FILE_NAME);
                } else {
                    println!(
                        "Linker error: {}",
                        std::str::from_utf8(&output.stderr).unwrap()
                    );
                }
            }
            Err(err) => println!("Failure of linking: {}", err),
        }
    }

    // generate printf
    fn generate_printf_func(&self) {
        let i32_type = self.context.i32_type();
        let str_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[str_type.into()], true);
        self.module.add_function(FUNC_PRINTF, printf_type, None);
    }

    fn generate_printf_call(&self, args: &[BasicMetadataValueEnum]) -> IntType {
        let func = self.module.get_function(FUNC_PRINTF).unwrap();
        self.builder.build_call(func, args, "tmpcall");
        self.context.i64_type()
    }

    // generate sprintf func
    fn generate_sprintf_func(&self) {
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let sprintf_type = i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], true);
        self.module.add_function(FUNC_SPRINTF, sprintf_type, None);
    }

    // generate sprintf call
    fn generate_sprintf_call(&self, args: &[BasicMetadataValueEnum<'ctx>]) -> BasicValueEnum<'ctx> {
        let func = self.module.get_function(FUNC_SPRINTF).unwrap();
        let basic_value = self
            .builder
            .build_call(func, args, "tmpcall")
            .try_as_basic_value()
            .left()
            .unwrap();

        basic_value
    }

    // creates a "global" string const and returns a ptr value to it
    fn generate_global_string(&self, value: &str, name: &str) -> PointerValue<'ctx> {
        let ty = self.context.i8_type().array_type(value.len() as u32);
        let gv = self
            .module
            .add_global(ty, Some(AddressSpace::default()), name);
        gv.set_linkage(Linkage::Internal);
        gv.set_initializer(&self.context.const_string(value.as_ref(), false));
        let ptr_value = self.builder.build_pointer_cast(
            gv.as_pointer_value(),
            self.context.i8_type().ptr_type(AddressSpace::default()),
            name,
        );

        ptr_value
    }

    fn execute(&self) {
        let ee = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        // get "main" function
        let maybe_fn = unsafe { ee.get_function::<unsafe extern "C" fn() -> f64>(FUNC_MAIN) };
        let compiled_fn = match maybe_fn {
            Ok(f) => f,
            Err(err) => {
                panic!("{:?}", err);
            }
        };

        // call "main"
        unsafe {
            compiled_fn.call();
        }
    }
}

pub fn create_compiler() {
    let context = Context::create();
    let module = context.create_module(SOURCE_FILE_NAME);
    let builder = context.create_builder();
    let compiler = Compiler::new(&context, &builder, &module);
    compiler.compile();
}

fn main() {
    create_compiler();
}
