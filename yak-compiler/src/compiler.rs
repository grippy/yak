// use super::{builder::Builder, context::Context, module::Module};

use crate::hir::{FunctionDef, Hir, ModuleDef};
use anyhow::{bail, Context, Error, Result};
use inkwell::builder::Builder;
use inkwell::context::Context as InkwellContext;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::AddressSpace;
// use inkwell::values::{BasicMetadataValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::values::FunctionValue;
use log::{error, info};
use std::fs;
use std::process::Command;
use yak_core::types::name::Name;

const FUNC_ENTRY: &str = "enter";

pub struct CompilerOpts {
    // The package we're building
    pub pkg_id: String,
    pub pkg_local_path: String,
    pub output_dir: String,
}

pub struct Compiler<'a, 'ctx> {
    pub opts: CompilerOpts,
    pub module_files: Vec<String>,
    pub hir: Hir,
    pub context: &'ctx InkwellContext,
    pub builder: &'a Builder<'ctx>,
    // pub fpm: &'a PassManager<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(
        opts: CompilerOpts,
        hir: Hir,
        context: &'ctx InkwellContext,
        builder: &'a Builder<'ctx>,
    ) -> Self {
        Compiler {
            opts,
            module_files: vec![],
            hir,
            context,
            builder,
        }
    }

    pub fn build(opts: CompilerOpts, hir: Hir) -> Result<()> {
        let context = &InkwellContext::create();
        let builder = &context.create_builder();
        let mut compiler = Compiler::new(opts, hir, context, builder);
        compiler.compile()?;
        compiler.link()
    }

    fn link(&self) -> Result<()> {
        let clang_bin = "clang";
        let mut cmd = Command::new(&clang_bin);
        // add module files
        self.module_files.iter().for_each(|mod_file| {
            cmd.arg(mod_file);
        });
        // verbose
        cmd.arg("-v");

        // add output filename
        let output_path = format!("{}/bin", &self.opts.output_dir);
        let output_file = format!("{}/{}", &output_path, &self.opts.pkg_id);
        let _ = fs::create_dir_all(&output_path)
            .with_context(|| format!("failed to create bin path: {}", &output_path))?;
        cmd.arg("-o");
        cmd.arg(&output_file);

        match cmd.output() {
            Ok(output) => {
                let status = output.status;
                if status.success() {
                    info!("Done building {}", &output_file);
                } else {
                    bail!(
                        "clang error: {}",
                        std::str::from_utf8(&output.stderr).unwrap()
                    );
                }
            }
            Err(err) => bail!(err.to_string()),
        }
        Ok(())
    }

    fn write_module(&self, module: &mut Module, module_files: &mut Vec<String>) -> Result<()> {
        // let i64_type = self.context.i64_type();
        // self.builder
        //     .build_return(Some(&i64_type.const_int(0, false)));

        // verify module and print it to file
        match module.verify() {
            Ok(_) => {
                let name = module.get_name().to_str()?;
                // let path = format!("/tmp/{}", &name);
                let path = format!("{}/module", &self.opts.output_dir);
                let _ = fs::create_dir_all(&path)
                    .with_context(|| format!("failed to create file path: {}", &path))?;
                let mod_file = format!("{}/{}.ll", &path, &name);
                let result = module.print_to_file(&mod_file[..]);
                match result {
                    Ok(_) => {
                        module_files.push(mod_file.clone());
                        info!("write module: {}", mod_file);
                        info!("======LL=======");
                        let _ = module.print_to_stderr();
                        info!("===============");
                    }
                    Err(err) => {
                        error!("Error writing module file {}", &mod_file);
                        bail!(err.to_string());
                    }
                }
            }
            Err(err) => {
                error!("LLVM module verify issue...");
                module.print_to_stderr();
                bail!(err.to_string());
            }
        }

        Ok(())
    }

    pub fn compile(&mut self) -> Result<()> {
        // iterate hir.modules
        let mut module_files: Vec<String> = vec![];
        for module_def in self.hir.modules.iter() {
            let module_name = module_def.module_id.name();
            info!("compile module {}", module_name);
            let module = &mut self.context.create_module(module_name.as_str());
            self.compile_constants(module, module_def)?;
            self.compile_functions(module, module_def)?;
            // should come after functions
            self.compile_main(module, module_def)?;
            self.compile_structs(module, module_def)?;
            self.write_module(module, &mut module_files)?;
        }
        // copy module files
        self.module_files = module_files;

        Ok(())
    }

    // Creates a @main function which calls the "pkg:main" definition
    fn compile_main(&self, module: &mut Module<'ctx>, module_def: &ModuleDef) -> Result<()> {
        if !module_def.pkg_root {
            return Ok(());
        }
        let main = module_def
            .function_defs
            .iter()
            .find(|func_def| func_def.function_id.is_main);

        if let Some(func_def) = main {
            info!("compile fn @main");

            let func_name = func_def.function_id.name();
            let func_name_str = func_name.as_str();

            // define return/input type signatures
            let i64_type = self.context.i64_type();
            let func_type =
                i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
            info!("func_type: {:?}", &func_type);

            // create main function
            let func_value = module.add_function("main", func_type, None);
            let basic_block = self.context.append_basic_block(func_value, FUNC_ENTRY);

            // lookup :main func value
            if let Some(func_main) = module.get_function(func_name_str) {
                // TODO:
                // 1. copy input args as func_main input args
                // 2. return func_main call as func return
                // let argsv = vec![];
                // match self
                //     .builder
                //     .build_call(func_main, argsv.as_slice(), "tmp")
                //     .try_as_basic_value()
                //     .left()
                // {
                //     Some(value) => Ok(value.into_float_value()),
                //     None => Err("Invalid call produced."),
                // };
            } else {
                bail!("Missing pkg :main function module function");
            }
            self.builder.position_at_end(basic_block);
            self.builder
                .build_return(Some(&i64_type.const_int(0, false)));
        } else {
            bail!(
                "Expected fn :main in package {}",
                module_def.module_id.name()
            )
        }
        Ok(())
    }

    fn compile_constants(&self, _module: &mut Module, module_def: &ModuleDef) -> Result<()> {
        module_def.constant_defs.iter().for_each(|const_def| {
            info!("compile const {}", const_def.constant_id.name());
        });
        Ok(())
    }

    // Compile all module functions
    fn compile_functions(&self, module: &mut Module<'ctx>, module_def: &ModuleDef) -> Result<()> {
        let mut func_defs_results = vec![];
        module_def.function_defs.iter().try_fold(
            &mut func_defs_results,
            |acc, func_def| -> Result<&mut Vec<()>> {
                let result = self.compile_function(module, module_def, func_def)?;
                acc.push(result);
                Ok(acc)
            },
        )?;
        Ok(())
    }

    // Compile a single function
    fn compile_function(
        &self,
        module: &mut Module<'ctx>,
        _module_def: &ModuleDef,
        func_def: &FunctionDef,
    ) -> Result<()> {
        info!("compile fn {}", func_def.function_id.name());
        let func_name = func_def.function_id.name();
        let func_name_str = func_name.as_str();

        // define return/input type signatures
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();

        // This creates the type signature in the form of
        // `return_type (arg_type, ...)`
        // let func_type =
        //     i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let func_type =
            void_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        info!("func_type: {:?}", &func_type);

        let func_value = module.add_function(func_name_str, func_type, None);
        let basic_block = self.context.append_basic_block(func_value, FUNC_ENTRY);

        self.builder.position_at_end(basic_block);
        self.builder.build_return(None);
        // self.builder.build_unreachable();
        // self.builder
        //     .build_return(Some(&i64_type.const_int(0, false)));

        Ok(())
    }

    fn compile_structs(&self, _module: &mut Module, module_def: &ModuleDef) -> Result<()> {
        module_def.struct_defs.iter().for_each(|struct_def| {
            info!("compile struct {}", struct_def.type_id.name());
        });

        Ok(())
    }
}
