use anyhow::{bail, Context, Error, Result};
use yak_ast::{Ast, ConstStmt, FuncStmt, StructStmt};
use yak_core::types::constant::ConstantId;
use yak_core::types::function::FunctionId;
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
    pub module_id: ModuleId,
    pub struct_defs: Vec<StructDef>,
    pub function_defs: Vec<FunctionDef>,
    pub constant_defs: Vec<ConstantDef>,
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

struct LetDef {}
struct EnumDef {}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub function_id: FunctionId,
}

impl Lower<FuncStmt> for FunctionDef {
    fn lower(stmt: &FuncStmt, opts: Opts) -> Result<Self> {
        if opts.pkg_id.is_none() {
            bail!("expected pkg_id value for FunctionDef")
        }
        let pkg_id = opts.pkg_id.unwrap();
        let func_name = stmt.func_name.clone();
        let def = FunctionDef {
            function_id: FunctionId::new(pkg_id, opts.struct_name, func_name),
        };
        Ok(def)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub type_id: TypeId,
}

impl Lower<StructStmt> for StructDef {
    fn lower(stmt: &StructStmt, opts: Opts) -> Result<Self> {
        if opts.pkg_id.is_none() {
            bail!("expected pkg_id value for StructStmt")
        }
        let pkg_id = opts.pkg_id.unwrap();
        let struct_name = stmt.struct_type.type_name.clone();
        let def = StructDef {
            type_id: TypeId::new(pkg_id, struct_name),
        };
        Ok(def)
    }
}

struct TraitDef {}
struct ImplTraitDef {}

// High-level IR
// Converts ast -> IR
#[derive(Debug, Default)]
pub struct Hir {
    pub modules: Vec<ModuleDef>,
}

impl Hir {
    pub fn from_ast(ast: &Ast) -> Result<Hir> {
        // get package stmt
        let pkg = &ast.parsed.package;
        // create module
        let pkg_id = clean_quotes(pkg.package_id.clone());
        let mut module = ModuleDef {
            module_id: ModuleId::new(pkg_id.clone()),
            ..Default::default()
        };
        // struct defs
        ast.parsed.structs.iter().try_fold(
            &mut module.struct_defs,
            |acc, stmt| -> Result<&mut Vec<StructDef>> {
                let opts = Opts {
                    pkg_id: Some(pkg_id.clone()),
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
                    pkg_id: Some(pkg_id.clone()),
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
                    pkg_id: Some(pkg_id.clone()),
                    ..Default::default()
                };
                let const_def = ConstantDef::lower(stmt, opts)?;
                acc.push(const_def);
                Ok(acc)
            },
        )?;
        // add module to hir
        let mut hir = Hir::default();
        hir.modules.push(module);

        Ok(hir)
    }
}
