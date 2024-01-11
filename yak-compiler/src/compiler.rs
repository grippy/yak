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
use yak_core::types::name::Name;

const FUNC_ENTRY: &str = "enter";

pub struct Compiler<'a, 'ctx> {
    pub hir: Hir,
    pub context: &'ctx InkwellContext,
    pub builder: &'a Builder<'ctx>,
    // pub fpm: &'a PassManager<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(hir: Hir, context: &'ctx InkwellContext, builder: &'a Builder<'ctx>) -> Self {
        Compiler {
            hir,
            context,
            builder,
        }
    }

    pub fn build(hir: Hir) -> Result<()> {
        let context = &InkwellContext::create();
        let builder = &context.create_builder();
        let compiler = Compiler::new(hir, context, builder);
        compiler.compile()
    }

    pub fn compile(&self) -> Result<()> {
        // iterate hir.modules
        for module_def in self.hir.modules.iter() {
            let module_name = module_def.module_id.name();
            info!("compile module {}", module_name);
            let module = &mut self.context.create_module(module_name.as_str());
            self.compile_constants(module, module_def)?;
            self.compile_functions(module, module_def)?;
            self.compile_structs(module, module_def)?;
            self.write_module(module)?;
        }

        Ok(())
    }

    fn write_module(&self, module: &mut Module) -> Result<()> {
        let i64_type = self.context.i64_type();
        self.builder
            .build_return(Some(&i64_type.const_int(0, false)));

        // verify module and print it to file
        match module.verify() {
            Ok(_) => {
                let name = module.get_name().to_str()?;
                let path = format!("/tmp/{}", &name);
                let _ = fs::create_dir_all(&path)
                    .with_context(|| format!("failed to create file path: {}", &path))?;
                let src_file = format!("{}/{}.ll", &path, &name);
                let result = module.print_to_file(&src_file[..]);
                match result {
                    Ok(_) => {
                        info!("write module: {}", src_file);
                        info!("======LL=======");
                        let _ = module.print_to_stderr();
                        info!("===============");
                    }
                    Err(err) => {
                        bail!(err.to_string());
                    }
                }
            }
            Err(err) => {
                error!("LLVM module verify issue...");
                bail!(err.to_string());
            }
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

        // define input function type
        let i64_type = self.context.i64_type();
        let func_type =
            i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);

        let func_value = module.add_function(func_name_str, func_type, None);
        let basic_block = self.context.append_basic_block(func_value, FUNC_ENTRY);
        self.builder.position_at_end(basic_block);

        Ok(())
    }

    fn compile_structs(&self, _module: &mut Module, module_def: &ModuleDef) -> Result<()> {
        module_def.struct_defs.iter().for_each(|struct_def| {
            info!("compile struct {}", struct_def.type_id.name());
        });

        Ok(())
    }
}
