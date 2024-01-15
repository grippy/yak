use anyhow::{bail, Context, Error, Result};
use yak_ast::{Ast, ConstStmt, FuncInputArgTypeStmt, FuncInputTypeStmt, FuncStmt, StructStmt};
use yak_core::types::constant::ConstantId;
use yak_core::types::field::FieldId;
use yak_core::types::function::{FunctionArgId, FunctionId};
use yak_core::types::module::ModuleId;
use yak_core::types::types::TypeId;
use yak_core::utils::clean_quotes;

trait Lower<Stmt> {
    fn lower(stmt: &Stmt, opts: Opts) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Opts {
    pkg_id: Option<String>,
    // required for FunctionDef
    struct_name: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModuleDef {
    pub pkg_root: bool,
    pub module_id: ModuleId,
    pub struct_defs: Vec<StructDef>,
    pub function_defs: Vec<FunctionDef>,
    pub constant_defs: Vec<ConstantDef>,
}

impl ModuleDef {
    pub fn fn_main(&self) -> Option<&FunctionDef> {
        self.function_defs
            .iter()
            .find(|func_def| func_def.function_id.is_main)
    }
}

struct Block {}
struct If {}
struct Condition {}
struct ExprValue {}
struct IntValue {}
struct FloatValue {}
struct StringValue {}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantDef {
    pub constant_id: ConstantId,
    // type_id: &TypeId,
}

impl Lower<ConstStmt> for ConstantDef {
    fn lower(stmt: &ConstStmt, opts: Opts) -> Result<Self> {
        if opts.pkg_id.is_none() {
            bail!("expected pkg_id value for ConstantDef")
        }
        let pkg_id = opts.pkg_id.unwrap();
        let const_name = stmt.assign.var_type.var_name.clone();
        let def = ConstantDef {
            constant_id: ConstantId::new(pkg_id, const_name),
        };
        Ok(def)
    }
}

pub struct LetDef {}
pub struct EnumDef {}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub function_id: FunctionId,
    pub args: Vec<FunctionArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArg {
    pub arg_id: FunctionArgId,
    pub type_id: TypeId,
}

impl Lower<FuncStmt> for FunctionDef {
    fn lower(stmt: &FuncStmt, opts: Opts) -> Result<Self> {
        if opts.pkg_id.is_none() {
            bail!("expected pkg_id value for FunctionDef")
        }
        let pkg_id = opts.pkg_id.unwrap();
        let func_name = stmt.func_name.clone();
        // function def
        let mut def = FunctionDef {
            function_id: FunctionId::new(pkg_id.clone(), opts.struct_name, func_name),
            args: vec![],
        };
        // function args
        if let Some(input_type) = &stmt.func_type.input_type {
            input_type.args.iter().enumerate().try_fold(
                &mut def.args,
                |acc, (input_num, input_arg)| -> Result<&mut Vec<FunctionArg>> {
                    let arg_name = input_arg.arg_name.clone();
                    let arg_num = input_num;
                    let arg_type = input_arg.arg_type.type_name.clone();
                    let func_arg = FunctionArg {
                        arg_id: FunctionArgId::new(arg_name, arg_num),
                        type_id: TypeId::new(pkg_id.clone(), arg_type),
                    };
                    acc.push(func_arg);
                    Ok(acc)
                },
            )?;
        }
        Ok(def)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub type_id: TypeId,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub field_id: FieldId,
    pub type_id: TypeId,
}

impl Lower<StructStmt> for StructDef {
    fn lower(stmt: &StructStmt, opts: Opts) -> Result<Self> {
        if opts.pkg_id.is_none() {
            bail!("expected pkg_id value for StructStmt")
        }
        // struct def
        let pkg_id = opts.pkg_id.unwrap();
        let struct_name = stmt.struct_type.type_name.clone();
        let mut def = StructDef {
            type_id: TypeId::new(pkg_id.clone(), struct_name),
            fields: vec![],
        };
        // struct fields
        stmt.fields.iter().enumerate().try_fold(
            &mut def.fields,
            |acc, (field_num, field)| -> Result<&mut Vec<StructField>> {
                let field_name = field.field_name.clone();
                let field_type = field.field_type.type_name.clone();
                let struct_field = StructField {
                    field_id: FieldId::new(field_name, field_num),
                    type_id: TypeId::new(pkg_id.clone(), field_type),
                };
                acc.push(struct_field);
                Ok(acc)
            },
        )?;

        Ok(def)
    }
}

pub struct TraitDef {}

pub struct ImplTraitDef {}

// High-level IR
// Converts ast -> IR
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Hir {
    pub modules: Vec<ModuleDef>,
}

impl Hir {
    pub fn from_ast(&mut self, pkg_root: bool, as_pkg_id: Option<String>, ast: &Ast) -> Result<()> {
        // get package stmt
        let pkg = &ast.parsed.package;
        // create module
        let pkg_id = clean_quotes(pkg.package_id.clone());
        let as_pkg_name = if let Some(as_pkg_id) = as_pkg_id {
            Some(as_pkg_id.clone())
        } else {
            Some(pkg_id.clone())
        };
        let mut module = ModuleDef {
            pkg_root: pkg_root,
            module_id: ModuleId::new(pkg_id.clone(), as_pkg_name.clone()),
            ..Default::default()
        };
        // struct defs
        ast.parsed.structs.iter().try_fold(
            &mut module.struct_defs,
            |acc, stmt| -> Result<&mut Vec<StructDef>> {
                let opts = Opts {
                    pkg_id: as_pkg_name.clone(),
                    ..Default::default()
                };
                let def = StructDef::lower(stmt, opts)?;
                acc.push(def);
                Ok(acc)
            },
        )?;
        // function defs
        ast.parsed.funcs.iter().try_fold(
            &mut module.function_defs,
            |acc, stmt| -> Result<&mut Vec<FunctionDef>> {
                // top-level functions don't have a struct name
                let opts = Opts {
                    pkg_id: as_pkg_name.clone(),
                    ..Default::default()
                };
                let def = FunctionDef::lower(stmt, opts)?;
                acc.push(def);
                Ok(acc)
            },
        )?;
        // constant defs
        ast.parsed.consts.iter().try_fold(
            &mut module.constant_defs,
            |acc, stmt| -> Result<&mut Vec<ConstantDef>> {
                let opts = Opts {
                    pkg_id: as_pkg_name.clone(),
                    ..Default::default()
                };
                let const_def = ConstantDef::lower(stmt, opts)?;
                acc.push(const_def);
                Ok(acc)
            },
        )?;
        // add module to hir
        self.modules.push(module);

        Ok(())
    }

    pub fn merge_modules(&mut self, hir: &Hir) {
        self.modules.append(&mut hir.modules.clone());
    }
}
