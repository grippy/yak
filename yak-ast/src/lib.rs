#![allow(dead_code)]
#![allow(private_interfaces)]

mod expr;

mod test;

use anyhow::{bail, Context, Error, Result};
use expr::expr::ExprParser;
use expr::pratt::PrattParser;
use std::fs;
use std::path::PathBuf;
use yak_core::models::yak_package::{
    Symbol, YakDependency, YakExport, YakFile, YakImport, YakPackage, YakSymbol,
};
use yak_core::models::yak_version::YakVersion;
use yak_lexer::token::TokenType as Ty;
use yak_lexer::{Lexer, Token};

#[cfg(not(test))]
use log::{debug, error, info, warn};
#[cfg(test)]
use std::{println as debug, println as info, println as error, println as warn};

// Strip quotes from start/end of AST strings
fn clean_quotes(mut s: String) -> String {
    if s.starts_with("\"") {
        s = s.strip_prefix("\"").unwrap().to_string();
    }
    if s.ends_with("\"") {
        s = s.strip_suffix("\"").unwrap().to_string();
    }
    s
}

#[derive(Debug, Clone, Default)]
struct Balance {
    // [
    bracket_l: usize,
    // ]
    bracket_r: usize,
    // {
    brace_l: usize,
    // }
    brace_r: usize,
    // {
    paren_l: usize,
    // }
    paren_r: usize,
}

impl Balance {
    fn balanced_brackets(&self) -> bool {
        self.bracket_l == self.bracket_r
    }

    fn balanced_braces(&self) -> bool {
        self.brace_l == self.brace_r
    }

    fn balanced_parens(&self) -> bool {
        self.paren_l == self.paren_r
    }
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub package: PackageStmt,
    pub consts: Vec<ConstStmt>,
    pub enums: Vec<EnumStmt>,
    pub errors: Vec<Error>,
    pub funcs: Vec<FuncStmt>,
    pub impl_traits: Vec<ImplTraitStmt>,
    pub lets: Vec<LetStmt>,
    pub primitives: Vec<PrimitiveStmt>,
    pub structs: Vec<StructStmt>,
    pub traits: Vec<TraitStmt>,
    pub tests: Vec<TestStmt>,
}

#[derive(Debug)]
pub struct Ast {
    file: Option<PathBuf>,
    stack: Vec<Token>,
    pub parsed: Parsed,
}

impl Ast {
    pub fn from_files(files: Vec<PathBuf>) -> Result<Vec<Self>> {
        let mut out: Vec<Ast> = vec![];
        for file in files {
            out.push(Ast::from_file(file)?);
        }
        Ok(out)
    }
    pub fn from_file(file: PathBuf) -> Result<Self> {
        let src = fs::read_to_string(&file)
            .with_context(|| format!("unable to read file: {}", &file.display()))?;
        let mut lexer = Lexer::from_source(&src);
        lexer.parse();
        Ok(Ast {
            file: Some(file),
            stack: lexer.tokens_as_stack(),
            parsed: Parsed::default(),
        })
    }
    pub fn from_source(source: &str) -> Self {
        let mut lexer = Lexer::from_source(source);
        lexer.parse();
        Ast {
            file: None,
            stack: lexer.tokens_as_stack(),
            parsed: Parsed::default(),
        }
    }
    pub fn parse_package(&mut self) -> Result<()> {
        match PackageStmt::parse(&mut self.stack) {
            Ok(stmt) => self.parsed.package = stmt,
            Err(err) => self.parsed.errors.push(err),
        };
        if self.parsed.errors.len() > 0 {
            for err in self.parsed.errors.iter() {
                error!("parser error: {}", &err);
            }
            bail!("fix these errors");
        }
        Ok(())
    }
    pub fn parse_file(&mut self, file: PathBuf) -> Result<()> {
        let src = fs::read_to_string(&file)
            .with_context(|| format!("unable to read file: {}", &file.display()))?;
        let mut lexer = Lexer::from_source(&src);
        lexer.parse();
        self.file = Some(file);
        self.stack = lexer.tokens_as_stack();
        self.parse()?;
        Ok(())
    }
    pub fn parse(&mut self) -> Result<()> {
        // parse top-level statements
        while let Some(token) = self.stack.pop() {
            // debug!("ast.parse {:?}", token.ty);
            match token.ty {
                Ty::Sp | Ty::NL => {}
                Ty::Comment(_) => {}
                Ty::Indent(indent) => {
                    if indent > 0 {
                        bail!("syntax error: top-level indentation not allowed");
                    }
                    match self.top_level_stmts() {
                        Some(err) => self.parsed.errors.push(err),
                        _ => {}
                    }
                }
                _ => {
                    bail!("failed to parse token: {:?}", token);
                }
            }
        }
        if self.parsed.errors.len() > 0 {
            for err in self.parsed.errors.iter() {
                error!("parser error: {}", &err);
            }
            bail!("fix these errors");
        }
        Ok(())
    }
    fn top_level_stmts(&mut self) -> Option<Error> {
        // current token is indent=0
        while let Some(token) = self.stack.pop() {
            // matching any of these on the top-level
            // should take everything until the end of the statement
            match token.ty {
                Ty::NL => {
                    // skip
                }
                Ty::Comment(_) => {
                    // skip
                }
                Ty::KwConst
                | Ty::KwEnum
                | Ty::KwFn
                | Ty::KwImpl
                | Ty::KwLet
                | Ty::KwPrimitive
                | Ty::KwStruct
                | Ty::KwTest
                | Ty::KwTestCase
                | Ty::KwTrait
                | Ty::KwType => {
                    let mut stack = self.take_toplevel_stmt();
                    match token.ty {
                        Ty::KwConst => {
                            let stmt = ConstStmt::parse(&mut stack);
                            match stmt {
                                Ok(const_stmt) => {
                                    debug!("const_stmt {:#?}", const_stmt);
                                    self.parsed.consts.push(const_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwEnum => {
                            let stmt = EnumStmt::parse(&mut stack);
                            match stmt {
                                Ok(enum_stmt) => {
                                    debug!("enum_stmt {:#?}", enum_stmt);
                                    self.parsed.enums.push(enum_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwFn => {
                            let stmt = FuncStmt::parse(&mut stack);
                            match stmt {
                                Ok(func_stmt) => {
                                    debug!("func_stmt {:#?}", func_stmt);
                                    self.parsed.funcs.push(func_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        // todo
                        Ty::KwImpl => {
                            todo!("parse impl")
                        }
                        Ty::KwLet => {
                            let stmt = LetStmt::parse(&mut stack);
                            match stmt {
                                Ok(let_stmt) => {
                                    debug!("let_stmt {:#?}", let_stmt);
                                    self.parsed.lets.push(let_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwPrimitive => {
                            let stmt = PrimitiveStmt::parse(&mut stack);
                            match stmt {
                                Ok(primitive_stmt) => {
                                    debug!("primitive_stmt {:#?}", primitive_stmt);
                                    self.parsed.primitives.push(primitive_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwStruct => {
                            let stmt = StructStmt::parse(&mut stack);
                            match stmt {
                                Ok(struct_stmt) => {
                                    debug!("struct_stmt {:#?}", struct_stmt);
                                    self.parsed.structs.push(struct_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        // todo
                        Ty::KwTest => {
                            todo!("parse test")
                        }
                        // todo
                        Ty::KwTestCase => {
                            todo!("parse testcase")
                        }
                        // todo
                        Ty::KwTrait => {
                            todo!("parse ^Trait")
                        }
                        // todo
                        Ty::KwType => {
                            todo!("parse type")
                        }
                        _ => {}
                    }
                }
                _ => {
                    // not a top-level token
                    self.stack.push(token);
                    break;
                }
            }
        }
        None
    }
    fn take_toplevel_stmt(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(token) = self.stack.pop() {
            if token.ty == Ty::Indent(0) {
                self.stack.push(token);
                break;
            } else if token.ty == Ty::Sp {
                // eat spaces
                continue;
            }
            tokens.push(token);
        }
        tokens.reverse();
        tokens
    }
}

//
// **** Utility functions
//

// Take until match
fn take_all_until_match_any(tokens: &mut Vec<Token>, matches: Vec<Ty>) -> Vec<Token> {
    let mut taken: Vec<Token> = vec![];
    while let Some(tok) = tokens.pop() {
        if matches.contains(&tok.ty) {
            tokens.push(tok);
            break;
        }
        taken.push(tok);
    }
    taken.reverse();
    taken
}

fn take_all_include_pattern(tokens: &mut Vec<Token>, pattern: Vec<Ty>) -> Result<Vec<Token>> {
    if pattern.len() == 0 {
        bail!("expected pattern length");
    }

    let mut pattern_iter = pattern.clone().into_iter();
    // debug!("checking for pattern: {:?}", &pattern_iter);

    let pattern_ty = pattern_iter.next().unwrap();
    let mut taken: Vec<Token> = vec![];

    while let Some(tok) = tokens.pop() {
        // debug!("> tok: {:?} pattern: {:?}", &tok.ty, &pattern_ty);
        if &tok.ty == &pattern_ty {
            taken.push(tok);
            while let Some(pattern_next_ty) = pattern_iter.next() {
                if tokens.len() > 0 {
                    let next = tokens.pop().unwrap();
                    // debug!(
                    //     ">> next: {:?} pattern_next: {:?}",
                    //     &next.ty, &pattern_next_ty
                    // );
                    if &next.ty == &pattern_next_ty {
                        taken.push(next);
                    } else {
                        // incomplete: reset pattern
                        pattern_iter = pattern.clone().into_iter();
                        // account for the top-level already has a placeholder
                        // for the first token stored in pattern_ty
                        pattern_iter.next();
                        tokens.push(next);
                        break;
                    }
                }
            }
            if pattern_iter.len() == 0 {
                break;
            }
        } else {
            // this didn't match so take
            taken.push(tok);
        }
    }

    taken.reverse();
    Ok(taken)
}

fn remove_newline_indent(tokens: &mut Vec<Token>) -> Result<Vec<Token>, Error> {
    let mut cleaned: Vec<Token> = vec![];
    while let Some(tok) = tokens.pop() {
        match tok.ty {
            Ty::Indent(_) | Ty::NL => {}
            _ => {
                cleaned.push(tok);
            }
        }
    }
    cleaned.reverse();
    Ok(cleaned)
}

fn remove_newline_indent_space(tokens: &mut Vec<Token>) -> Result<Vec<Token>, Error> {
    let mut cleaned: Vec<Token> = vec![];
    while let Some(tok) = tokens.pop() {
        match tok.ty {
            Ty::Indent(_) | Ty::NL | Ty::Sp => {}
            _ => {
                cleaned.push(tok);
            }
        }
    }
    cleaned.reverse();
    Ok(cleaned)
}

// Convert TokenType::Op* to Op

fn ty_into_op(ty: Ty) -> Result<Op, Error> {
    let op = match ty {
        Ty::OpEqEq => Op::Boolean(BooleanOp::EqEq),
        Ty::OpNotEq => Op::Boolean(BooleanOp::NotEq),
        Ty::OpGte => Op::Boolean(BooleanOp::Gte),
        Ty::OpGt => Op::Boolean(BooleanOp::Gt),
        Ty::OpLte => Op::Boolean(BooleanOp::Lte),
        Ty::OpLt => Op::Boolean(BooleanOp::Lt),
        Ty::OpAdd => Op::Arith(ArithOp::Add),
        Ty::OpSub => Op::Arith(ArithOp::Sub),
        Ty::OpMul => Op::Arith(ArithOp::Mul),
        Ty::OpDiv => Op::Arith(ArithOp::Div),
        Ty::OpMod => Op::Arith(ArithOp::Mod),
        Ty::OpAssignEq => Op::Assign(AssignOp::Eq),
        Ty::OpAssignAdd => Op::Assign(AssignOp::Add),
        Ty::OpAssignSub => Op::Assign(AssignOp::Sub),
        Ty::OpAssignDiv => Op::Assign(AssignOp::Div),
        Ty::OpAssignMul => Op::Assign(AssignOp::Mul),
        Ty::OpLogicalAnd => Op::Logical(LogicalOp::And),
        Ty::OpLogicalOr => Op::Logical(LogicalOp::Or),
        Ty::OpBitwiseAnd => Op::Bitwise(BitwiseOp::And),
        Ty::OpBitwiseOr => Op::Bitwise(BitwiseOp::Or),
        _ => bail!("Expected operator (assign, boolean, arithmetic, logical, or bitwise",),
    };
    return Ok(op);
}

// `into_type_stmt_generics` parses an inner generic type statement
// `T1[T2...]`
fn into_type_stmt_generics(stack: &mut Vec<Token>) -> Result<Vec<TypeStmt>, Error> {
    debug!("into_type_stmt_generics {:?}", stack);

    // TypeStmt
    let mut generics: Vec<TypeStmt> = vec![];

    // we should have an IdType here
    // var x: A[B[C[D]]E]
    while let Some(tok) = stack.pop() {
        let mut type_stmt = TypeStmt::default();
        match tok.ty {
            Ty::IdType(id) => {
                type_stmt.type_name = id;
            }
            _ => {
                if Ty::primitives().contains(&tok.ty) || Ty::builtins().contains(&tok.ty) {
                    // convert the type to String
                    type_stmt.type_name = tok.ty.into();
                } else {
                    debug!("failed on: {:?}", tok);
                    bail!("type statement inner generic expected a type identity");
                }
            }
        }

        if let Some(tok) = stack.pop() {
            if tok.ty == Ty::PunctBracketL {
                // last token should be right bracket
                // strip and parse inner
                let mut balance = Balance::default();
                balance.bracket_l += 1;
                if let Some(tok) = stack.first() {
                    match tok.ty {
                        Ty::PunctBracketR => {
                            stack.remove(0);
                            balance.bracket_r += 1;
                        }
                        _ => {}
                    }
                }
                if balance.balanced_brackets() {
                    type_stmt.generics = Some(Box::new(into_type_stmt_generics(stack)?));
                } else {
                    bail!(
                        "unable to parse type statement inner generic: expected balanced brackets",
                    );
                }
            } else {
                stack.push(tok)
            }
        }

        generics.push(type_stmt);
    }

    Ok(generics)
}

//
// **** Traits
//

// Parse trait
trait Parse {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error>
    where
        Self: Sized;

    fn validate(&self) -> Result<(), Error>;
}

// ParseSelf trait
trait ParseSelf {
    fn parse(self, stack: &mut Vec<Token>) -> Result<Self, Error>
    where
        Self: Sized;

    fn validate(&self) -> Result<(), Error>;
}

//
//
//
// Ast Statements
//
//
//

//
// Package Statements
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageStmt {
    pub package_id: String,
    pub description: String,
    pub version: String,
    pub files: Vec<PackageFileStmt>,
    pub dependencies: Vec<PackageDependencyStmt>,
    pub imports: Vec<PackageImportStmt>,
    pub exports: PackageExportStmt,
}

impl PackageStmt {
    pub fn into_yak_package(
        self,
        pkg_root: bool,
        pkg_local_path: String,
        pkg_remote_path: Option<String>,
    ) -> Result<YakPackage> {
        let mut pkg = YakPackage::default();
        pkg.pkg_root = pkg_root;
        pkg.pkg_id = clean_quotes(self.package_id);
        pkg.pkg_description = clean_quotes(self.description);
        pkg.pkg_version = YakVersion {
            version: clean_quotes(self.version),
        };
        pkg.pkg_local_path = pkg_local_path;
        pkg.pkg_remote_path = pkg_remote_path;
        // convert files...
        pkg.pkg_files = self.files.into_iter().map(|file| file.into()).collect();
        // convert dependencies
        pkg.pkg_dependencies = self
            .dependencies
            .into_iter()
            .map(|dep| dep.into())
            .collect();
        // convert imports
        pkg.pkg_imports = self.imports.into_iter().map(|imp| imp.into()).collect();
        // convert exports
        pkg.pkg_exports = self.exports.into();
        Ok(pkg)
    }
}

impl Parse for PackageStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        info!("PackageStmt {:?}", stack);
        // remove newlines and indentation
        let mut pkg_stmt = PackageStmt::default();
        while let Some(token) = stack.pop() {
            match token.ty {
                Ty::NL => {
                    // skip
                }
                Ty::Comment(_) => {
                    // skip
                }
                Ty::Indent(0) => {
                    // skip
                }
                Ty::KwPackage => {
                    while let Some(next) = stack.pop() {
                        match next.ty {
                            Ty::IdPackage(id) | Ty::IdVar(id) | Ty::LitString(id) => {
                                pkg_stmt.package_id = id;
                                break;
                            }
                            Ty::Sp => {}
                            _ => {
                                stack.push(next);
                                bail!("failed to parse package id. Expected IdPackage, IdVar or LitString")
                            }
                        }
                    }
                }
                Ty::KwDescription => {
                    while let Some(next) = stack.pop() {
                        match next.ty {
                            Ty::LitString(desc) => {
                                pkg_stmt.description = desc;
                                break;
                            }
                            Ty::Sp => {}
                            _ => {
                                stack.push(next);
                                bail!("failed to parse package description. Expected LitString")
                            }
                        }
                    }
                }
                Ty::KwVersion => {
                    while let Some(next) = stack.pop() {
                        match next.ty {
                            Ty::LitString(version) => {
                                pkg_stmt.version = version;
                                break;
                            }
                            Ty::Sp => {}
                            _ => {
                                stack.push(next);
                                bail!("failed to parse package version. Expected LitString")
                            }
                        }
                    }
                }
                Ty::KwDependencies => {
                    debug!("deps stack: {:?}\n", stack);
                    // can't reuse deps here because of how anyhow results work
                    let mut deps = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut deps = remove_newline_indent_space(&mut deps)?;
                    debug!("deps: {:?}\n\n", deps);

                    // files is List of string
                    if let Some(next) = deps.pop() {
                        match next.ty {
                            Ty::PunctBraceL => {}
                            _ => {
                                bail!("failed to parse package dependencies. Expected PunctBraceL")
                            }
                        }
                    }
                    // iterate deps
                    let mut index = 0usize;
                    let mut dep_stmt = PackageDependencyStmt::default();
                    while let Some(next) = deps.pop() {
                        match next.ty {
                            Ty::IdVar(package_id) | Ty::IdPackage(package_id) => {
                                if index != 0 {
                                    bail!("failed to parse package dependencies. Expected pattern IdVar or IdPackage followed by LitString")
                                }
                                index += 1;
                                dep_stmt.package_id = package_id;
                            }
                            Ty::LitString(path) => {
                                if index != 1 {
                                    bail!("failed to parse package dependencies. Expected pattern IdVar or IdPackage followed by LitString")
                                }
                                // reset
                                index = 0;
                                dep_stmt.path = path;
                                pkg_stmt.dependencies.push(dep_stmt.clone())
                            }
                            Ty::PunctBraceR => {
                                break;
                            }
                            _ => {
                                bail!("failed to parse package files. Expected LitString or PunctBraceR")
                            }
                        }
                    }
                }
                Ty::KwExport => {
                    // debug!("export stack: {:?}\n", stack);
                    let mut exports = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut exports = remove_newline_indent_space(&mut exports)?;
                    // debug!("exports: {:?}\n\n", exports);

                    // files is List of string
                    if let Some(next) = exports.pop() {
                        match next.ty {
                            Ty::PunctBraceL => {}
                            _ => {
                                bail!("failed to parse package export. Expected PunctBraceL")
                            }
                        }
                    }

                    // iterate list
                    while let Some(next) = exports.pop() {
                        match next.ty {
                            // const
                            Ty::IdVar(sym) => {
                                let mut sym_stmt = PackageSymbolStmt::default();
                                sym_stmt.symbol = PackageSymbol::Var(sym);
                                pkg_stmt.exports.symbols.push(sym_stmt);
                            }
                            // :func
                            Ty::IdFunc(sym) => {
                                let mut sym_stmt = PackageSymbolStmt::default();
                                sym_stmt.symbol = PackageSymbol::Func(sym);
                                pkg_stmt.exports.symbols.push(sym_stmt);
                            }
                            // Type
                            Ty::IdType(sym) => {
                                let mut sym_stmt = PackageSymbolStmt::default();
                                sym_stmt.symbol = PackageSymbol::Type(sym);
                                pkg_stmt.exports.symbols.push(sym_stmt);
                            }
                            // Trait
                            Ty::IdTrait(sym) => {
                                let mut sym_stmt = PackageSymbolStmt::default();
                                sym_stmt.symbol = PackageSymbol::Trait(sym);
                                pkg_stmt.exports.symbols.push(sym_stmt);
                            }
                            Ty::PunctBraceR => {
                                break;
                            }
                            _ => {
                                if Ty::primitives().contains(&next.ty) {
                                    let mut sym_stmt = PackageSymbolStmt::default();
                                    let sym: String = next.ty.into();
                                    sym_stmt.symbol = PackageSymbol::Primitive(sym);
                                    pkg_stmt.exports.symbols.push(sym_stmt);
                                    continue;
                                } else if Ty::builtins().contains(&next.ty) {
                                }

                                bail!("failed to parse package export. Expected PunctBraceR. Found {:?}", next)
                            }
                        }
                    }
                }
                Ty::KwImport => {
                    debug!("import stack: {:?}\n", stack);
                    let mut imports = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut imports = remove_newline_indent_space(&mut imports)?;
                    debug!("imports: {:?}\n\n", imports);

                    // files is List of string
                    if let Some(next) = imports.pop() {
                        match next.ty {
                            Ty::PunctBraceL => {}
                            _ => {
                                bail!("failed to parse package import. Expected PunctBraceL")
                            }
                        }
                    }

                    // iterate list
                    while let Some(next) = imports.pop() {
                        match next.ty {
                            // const
                            Ty::IdVar(package_id) | Ty::IdPackage(package_id) => {
                                // sniff next
                                let mut imp_stmt = PackageImportStmt::default();
                                imp_stmt.package_id = package_id;
                                if let Some(next) = imports.pop() {
                                    match next.ty {
                                        Ty::KwAs => {
                                            // next should be a package id
                                            if let Some(next) = imports.pop() {
                                                match next.ty {
                                                    Ty::IdVar(as_package_id)
                                                    | Ty::IdPackage(as_package_id) => {
                                                        imp_stmt.as_package_id =
                                                            Some(as_package_id);
                                                    }
                                                    _ => {
                                                        bail!("failed to parse package import. Expected IdVar or IdPackage after KwAs")
                                                    }
                                                }
                                            }
                                        }
                                        Ty::IdVar(_) | Ty::IdPackage(_) => {
                                            pkg_stmt.imports.push(imp_stmt);
                                            imports.push(next);
                                            continue;
                                        }
                                        Ty::PunctBraceL => {
                                            // handle this directly below
                                            // or on the next iteration
                                            debug!("push next: {:?}", next);
                                            imports.push(next);
                                        }
                                        Ty::PunctBraceR => {
                                            break;
                                        }
                                        _ => {
                                            bail!("failed to parse package import. Unexpected token after IdVar or IdPackage (found {:?})", next)
                                        }
                                    }
                                }

                                // parse symbols
                                if let Some(next) = imports.pop() {
                                    match next.ty {
                                        Ty::PunctBraceL => {
                                            // Behold thy glorious import symbols
                                            // should be similar to export but we can alias these
                                            while let Some(next) = imports.pop() {
                                                match next.ty {
                                                    // const
                                                    Ty::IdVar(sym) => {
                                                        let mut sym_stmt =
                                                            PackageSymbolStmt::default();
                                                        sym_stmt.symbol = PackageSymbol::Var(sym);

                                                        // check for alias.. should match IdVar
                                                        if let Some(next) = imports.pop() {
                                                            match next.ty {
                                                                Ty::KwAs => {
                                                                    if let Some(next) =
                                                                        imports.pop()
                                                                    {
                                                                        match next.ty {
                                                                    Ty::IdVar(sym) => {
                                                                        sym_stmt.as_symbol = Some(PackageSymbol::Var(sym));
                                                                    },
                                                                    _ => bail!("failed to parse package import. Expected KwAs of IdVar type to match")
                                                                }
                                                                    }
                                                                }
                                                                _ => {
                                                                    imports.push(next);
                                                                }
                                                            }
                                                        }
                                                        imp_stmt.symbols.push(sym_stmt);
                                                    }
                                                    // :func
                                                    Ty::IdFunc(sym) => {
                                                        let mut sym_stmt =
                                                            PackageSymbolStmt::default();
                                                        sym_stmt.symbol = PackageSymbol::Func(sym);
                                                        // check for alias.. should match IdFunc
                                                        if let Some(next) = imports.pop() {
                                                            match next.ty {
                                                                Ty::KwAs => {
                                                                    if let Some(next) =
                                                                        imports.pop()
                                                                    {
                                                                        match next.ty {
                                                                    Ty::IdFunc(sym) => {
                                                                        sym_stmt.as_symbol = Some(PackageSymbol::Func(sym));
                                                                    },
                                                                    _ => bail!("failed to parse package import. Expected KwAs of IdFunc type to match")
                                                                }
                                                                    }
                                                                }
                                                                _ => {
                                                                    imports.push(next);
                                                                }
                                                            }
                                                        }
                                                        imp_stmt.symbols.push(sym_stmt);
                                                    }
                                                    // Type
                                                    Ty::IdType(sym) => {
                                                        debug!("parse type sym: {:?}", sym);
                                                        let mut sym_stmt =
                                                            PackageSymbolStmt::default();
                                                        sym_stmt.symbol = PackageSymbol::Type(sym);

                                                        // check for alias.. should match IdType
                                                        if let Some(next) = imports.pop() {
                                                            match next.ty {
                                                                Ty::KwAs => {
                                                                    if let Some(next) =
                                                                        imports.pop()
                                                                    {
                                                                        match next.ty {
                                                                    Ty::IdType(sym) => {
                                                                        sym_stmt.as_symbol = Some(PackageSymbol::Type(sym));
                                                                    },
                                                                    _ => bail!("failed to parse package import. Expected KwAs of IdType type to match")
                                                                }
                                                                    }
                                                                }
                                                                _ => {
                                                                    imports.push(next);
                                                                }
                                                            }
                                                        }
                                                        imp_stmt.symbols.push(sym_stmt);
                                                    }

                                                    Ty::IdTrait(name) => {
                                                        let mut sym_stmt =
                                                            PackageSymbolStmt::default();
                                                        sym_stmt.symbol =
                                                            PackageSymbol::Trait(name);

                                                        // check alias
                                                        if let Some(next) = imports.pop() {
                                                            match next.ty {
                                                                Ty::KwAs => {
                                                                    if let Some(next) =
                                                                        imports.pop()
                                                                    {
                                                                        match next.ty {
                                                                            Ty::IdTrait(name) => {
                                                                                sym_stmt.as_symbol =
                                                                                Some(PackageSymbol::Trait(name));
                                                                                imp_stmt
                                                                                    .symbols
                                                                                    .push(sym_stmt);
                                                                            }
                                                                            _ => {
                                                                                bail!("failed to parse package import. KwAs doesn't match IdTrait")
                                                                            }
                                                                        }
                                                                    } else {
                                                                        bail!("failed to parse package import. Expected KwAs of IdType type to match")
                                                                    }
                                                                }
                                                                _ => {
                                                                    imports.push(next);
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Ty::PunctBraceR => {
                                                        pkg_stmt.imports.push(imp_stmt);
                                                        break;
                                                    }
                                                    _ => {
                                                        bail!("failed to parse package import. Unexpected token {:?} while parsing package symbols", next)
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            // TODO: is this an error or not?
                                            // we don't have any symbols to import
                                            pkg_stmt.imports.push(imp_stmt);
                                            continue;
                                        }
                                    }
                                }
                            }

                            Ty::PunctBraceR => {
                                break;
                            }
                            _ => {
                                bail!("failed to parse package import. Expected PunctBraceR but encountered {:?}", next)
                            }
                        }
                    }
                }
                Ty::KwFiles => {
                    // debug!("files stack: {:?}\n", stack);
                    let mut files = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut files = remove_newline_indent_space(&mut files)?;
                    // debug!("files: {:?}\n\n", files);
                    // files is List of string
                    if let Some(next) = files.pop() {
                        match next.ty {
                            Ty::PunctBraceL => {}
                            _ => {
                                bail!("failed to parse package files. Expected PunctBraceL")
                            }
                        }
                    }
                    // iterate list
                    while let Some(next) = files.pop() {
                        match next.ty {
                            Ty::LitString(file) => {
                                let file_stmt = PackageFileStmt { path: file };
                                pkg_stmt.files.push(file_stmt);
                            }
                            Ty::PunctBraceR => {
                                break;
                            }
                            _ => {
                                bail!("failed to parse package files. Expected LitString or PunctBraceR")
                            }
                        }
                    }
                }
                _ => {
                    bail!("unsupported top-level package field")
                }
            }
        }

        info!("PkgStmt: {:?}", pkg_stmt);

        Ok(pkg_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageDependencyStmt {
    pub package_id: String,
    pub path: String,
}

impl Into<YakDependency> for PackageDependencyStmt {
    fn into(self) -> YakDependency {
        let mut yak_dep = YakDependency::default();
        yak_dep.pkg_id = clean_quotes(self.package_id);
        yak_dep.path = clean_quotes(self.path);
        yak_dep
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageFileStmt {
    pub path: String,
}

impl Into<YakFile> for PackageFileStmt {
    fn into(self) -> YakFile {
        let mut yak_file = YakFile::default();
        yak_file.path = clean_quotes(self.path);
        yak_file
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageImportStmt {
    pub package_id: String,
    pub as_package_id: Option<String>,
    // we need to explicitly import symbols
    // from a package to make them available
    pub symbols: Vec<PackageSymbolStmt>,
}

impl Into<YakImport> for PackageImportStmt {
    fn into(self) -> YakImport {
        let mut yak_import = YakImport::default();
        yak_import.pkg_id = self.package_id;
        yak_import.as_pkg_id = self.as_package_id;
        yak_import.symbols = self.symbols.into_iter().map(|sym| sym.into()).collect();
        yak_import
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageExportStmt {
    symbols: Vec<PackageSymbolStmt>,
}

impl Into<YakExport> for PackageExportStmt {
    fn into(self) -> YakExport {
        let mut yak_export = YakExport::default();
        yak_export.symbols = self.symbols.into_iter().map(|sym| sym.into()).collect();
        yak_export
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSymbol {
    None,
    // Box<PackageSymbol>
    Builtin(String),
    Primitive(String),
    Var(String),
    Func(String),
    Type(String),
    Trait(String),
}

impl Default for PackageSymbol {
    fn default() -> Self {
        Self::None
    }
}

impl Into<Symbol> for PackageSymbol {
    fn into(self) -> Symbol {
        match self {
            PackageSymbol::None => Symbol::None,
            PackageSymbol::Builtin(s) => Symbol::Builtin(clean_quotes(s)),
            PackageSymbol::Primitive(s) => Symbol::Primitive(clean_quotes(s)),
            PackageSymbol::Var(s) => Symbol::Var(clean_quotes(s)),
            PackageSymbol::Func(s) => Symbol::Func(clean_quotes(s)),
            PackageSymbol::Type(s) => Symbol::Type(clean_quotes(s)),
            PackageSymbol::Trait(s) => Symbol::Trait(clean_quotes(s)),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageSymbolStmt {
    pub symbol: PackageSymbol,
    pub as_symbol: Option<PackageSymbol>,
}

impl Into<YakSymbol> for PackageSymbolStmt {
    fn into(self) -> YakSymbol {
        let mut yak_sym = YakSymbol::default();
        yak_sym.symbol = self.symbol.into();
        yak_sym.as_symbol = if self.as_symbol.is_some() {
            Some(self.as_symbol.unwrap().into())
        } else {
            None
        };
        yak_sym
    }
}

//
// Primitive statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrimitiveStmt {
    pub primitive_type: TypeStmt,
}

impl Parse for PrimitiveStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("PrimitiveStmt parse {:?}", stack);
        let mut primitive_stmt = PrimitiveStmt::default();
        primitive_stmt.primitive_type = TypeStmt::parse(&mut stack)?;
        Ok(primitive_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        // this should validate that the op is an assignment eq
        Ok(())
    }
}

//
// Constant statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConstStmt {
    pub assign: AssignStmt,
}

impl Parse for ConstStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("ConstStmt parse {:?}", stack);
        let assign = AssignStmt::parse(stack)?;
        Ok(ConstStmt { assign })
    }
    fn validate(&self) -> Result<(), Error> {
        // this should validate that the op is an assignment eq
        Ok(())
    }
}

//
// Assignment statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssignStmt {
    pub var_type: VarTypeStmt,
    pub op: Op,
    pub expr: ExprStmt,
}

impl Parse for AssignStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("AssignStmt parse {:?}", stack);
        let mut assign = AssignStmt::default();
        // VarTypeStmt
        assign.var_type = VarTypeStmt::parse(&mut stack)?;

        // Operator
        // Convert TokenType::Op* => Op
        if let Some(token) = stack.pop() {
            assign.op = ty_into_op(token.ty)?
        }

        // ExprStmt
        assign.expr = ExprStmt::parse(stack)?;

        return Ok(assign);
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

//
// Variable type statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VarTypeStmt {
    pub var_name: String,
    pub var_type: Option<TypeStmt>,
}

impl Parse for VarTypeStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("VarTypeStmt parse {:?}", stack);
        if let Some(token) = stack.pop() {
            match token.ty {
                Ty::IdVar(id) => {
                    let mut var_type_stmt = VarTypeStmt::default();
                    var_type_stmt.var_name = id;

                    // is next token a ":" or operator?
                    if stack.len() > 0 {
                        let tok = stack.pop().unwrap();
                        if tok.ty == Ty::PunctColon {
                            let mut taken = take_all_until_match_any(stack, Ty::operators());
                            var_type_stmt.var_type = Some(TypeStmt::parse(&mut taken)?);
                        } else {
                            stack.push(tok);
                        }
                    }
                    return Ok(var_type_stmt);
                }
                _ => {}
            }
        }
        bail!("expected to find VarTypeStmt");
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

//
// Type statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeStmt {
    pub type_name: String,
    pub generics: Option<Box<Vec<TypeStmt>>>,
}
impl Parse for TypeStmt {
    // TypeStmt
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("TypeStmt parse {:?}", stack);
        if stack.len() == 0 {
            bail!("type statement has no tokens");
        }
        let mut type_stmt = Self::default();

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdType(id) => {
                    type_stmt.type_name = id;
                }
                _ => {
                    if Ty::primitives().contains(&tok.ty) || Ty::builtins().contains(&tok.ty) {
                        // convert the type to String
                        type_stmt.type_name = tok.ty.into();
                        // a primitive will never have generics...
                        // so return early (this cleans up how we might parse tuple types)
                        return Ok(type_stmt);
                    } else {
                        debug!("failed on: {:?}", tok);
                        bail!("type statement expected a type identity");
                    }
                }
            }
        }

        // test if we should try to parse generics or not
        // (we won't have these if we're parsing value types)
        let has_generics = stack.iter().find_map(|tok| {
            if tok.ty.eq(&Ty::PunctBracketL) {
                Some(true)
            } else {
                None
            }
        });

        debug!("has_generics {:?}", &has_generics);
        if let Some(true) = has_generics {
            // parse inner generic type stmts
            // [ T ]

            // We need to account for StructValue types here (i.e. stop before )
            let mut inner = take_all_until_match_any(stack, vec![Ty::PunctBraceL]);
            debug!("generics {:?}", &inner);

            // check for balanced brackets
            // iterate the list of inner tokens
            let mut balance = Balance::default();

            for tok in &inner {
                match tok.ty {
                    Ty::PunctBracketL => {
                        balance.bracket_l += 1;
                    }
                    Ty::PunctBracketR => {
                        balance.bracket_r += 1;
                    }
                    _ => {}
                }
            }
            if balance.balanced_brackets() {
                // pop last and first
                inner.pop();
                inner.remove(0);
                type_stmt.generics = Some(Box::new(into_type_stmt_generics(&mut inner)?));
            } else {
                bail!("unable to parse type statement inner generic: expected balanced brackets",);
            }
        }

        Ok(type_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

//
// Enum statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnumStmt {
    pub enum_name: String,
    pub variants: Vec<EnumVariantStmt>,
}

impl Parse for EnumStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("EnumStmt parse {:?}", stack);

        if stack.len() == 0 {
            bail!("type statement has no tokens");
        }
        let mut enum_stmt = Self::default();

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdType(id) => {
                    enum_stmt.enum_name = id;
                }
                _ => {
                    bail!("enum statement expected a type identity");
                }
            }
        }

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::NL => {}
                _ => {
                    bail!("enum statement expected a newline after identity");
                }
            }
        }

        // we should always have a NL here...
        loop {
            let mut inner = take_all_until_match_any(stack, vec![Ty::NL]);
            // does this span multiple lines?
            // check if inner is balanced
            let mut balance = Balance::default();
            inner.iter().for_each(|tok| match tok.ty {
                Ty::PunctBraceL => balance.brace_l += 1,
                Ty::PunctBraceR => balance.brace_r += 1,
                _ => {}
            });

            // span brace across multiple lines
            if balance.brace_l > balance.brace_r {
                // take until end of brace right
                let mut combine =
                    take_all_include_pattern(stack, vec![Ty::Indent(2), Ty::PunctBraceR])?;
                combine.append(&mut inner);
                inner = combine;
            }
            if inner.len() == 0 {
                break;
            }
            let variant = EnumVariantStmt::parse(&mut inner)?;
            enum_stmt.variants.push(variant);

            if let Some(tok) = stack.pop() {
                match tok.ty {
                    Ty::NL => {}
                    _ => {
                        bail!("enum statement expected enum variant");
                    }
                }
            }
            // debug!("-----");
        }

        // let variants = vec![];
        Ok(enum_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariantType {
    None,
    Struct(EnumStructStmt),
    Tuple(TupleStmt),
}

impl Default for EnumVariantType {
    fn default() -> Self {
        EnumVariantType::None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnumVariantStmt {
    pub variant_name: String,
    pub variant_type: EnumVariantType,
}

impl Parse for EnumVariantStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        // remove all ident + NL
        let mut cleaned: Vec<Token> = remove_newline_indent(&mut stack)?;
        debug!("EnumVariantStmt parse {:?} (cleaned)", cleaned);
        let mut enum_variant = EnumVariantStmt::default();

        if let Some(tok) = cleaned.pop() {
            match tok.ty {
                Ty::IdType(name) => {
                    enum_variant.variant_name = name;
                }
                _ => {
                    bail!("enum variant statement expected indent");
                }
            }
        } else {
            bail!("enum variant statement expected indent");
        }

        if let Some(tok) = cleaned.pop() {
            match tok.ty {
                Ty::PunctBraceL => {
                    // struct or tuple type
                    // debug!("\n####");
                    // debug!("before {:?}", &cleaned);
                    match cleaned.first() {
                        Some(tok) => match tok.ty {
                            Ty::PunctBraceR => {
                                cleaned.reverse();
                                cleaned.pop();
                                cleaned.reverse();
                            }
                            _ => {
                                bail!("enum variant statement expected closing brace");
                            }
                        },
                        _ => {
                            bail!("enum variant statement expected closing brace");
                        }
                    }
                    // debug!("after {:?}", &cleaned);
                    // debug!("\n####");
                }
                _ => cleaned.push(tok),
            }

            if let Some(next) = cleaned.pop() {
                // do we have a tuple or struct?
                match next.ty {
                    Ty::IdVar(_) => {
                        // parse enum struct type
                        // add back IdVar
                        cleaned.push(next);
                        let enum_struct_stmt = EnumStructStmt::parse(&mut cleaned)?;
                        enum_variant.variant_type = EnumVariantType::Struct(enum_struct_stmt);
                    }
                    Ty::IdType(_) => {
                        cleaned.push(next);
                        enum_variant.variant_type =
                            EnumVariantType::Tuple(TupleStmt::parse(&mut cleaned)?);
                    }
                    _ => {
                        // we might have a primitive type here
                        // so parse tuple type if we do...
                        if Ty::primitives().contains(&next.ty) || Ty::builtins().contains(&next.ty)
                        {
                            cleaned.push(next);
                            enum_variant.variant_type =
                                EnumVariantType::Tuple(TupleStmt::parse(&mut cleaned)?);
                        } else {
                            debug!("failed on: {:?}", next);
                            bail!("enum variant statement expected variable or type or primitive identity");
                        }
                    }
                }
            }
        }

        Ok(enum_variant)
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnumStructStmt {
    pub fields: Vec<StructFieldStmt>,
}

impl Parse for EnumStructStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("EnumStructStmt parse {:?}", stack);
        let mut enum_struct = EnumStructStmt::default();

        let mut field: Vec<Token> = vec![];
        while let Some(next) = stack.pop() {
            match next.ty {
                Ty::IdVar(_) => {
                    if field.len() > 0 {
                        field.reverse();
                        enum_struct.fields.push(StructFieldStmt::parse(&mut field)?);

                        if field.len() > 0 {
                            debug!(
                                "field variable should have no length after parsing StructField"
                            );
                            field.clear();
                        }
                    }
                }
                _ => {}
            }
            field.push(next);
        }

        // flush field if it has a length
        if field.len() > 0 {
            field.reverse();
            enum_struct.fields.push(StructFieldStmt::parse(&mut field)?);
        }

        Ok(enum_struct)
    }

    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnumValueStmt {
    pub enum_name: String,
    pub variant_name: String,
    pub variant_value_type: EnumVariantValueType,
}

impl Parse for EnumValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("EnumValueStmt parse {:?}", stack);
        warn!("EnumValueStmt parsing not currently implemented!!!");
        let enum_val = EnumValueStmt::default();
        Ok(enum_val)
    }
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariantValueType {
    None,
    Struct(Box<StructValueStmt>),
    Tuple(Box<TupleValueStmt>),
}

impl Default for EnumVariantValueType {
    fn default() -> Self {
        EnumVariantValueType::None
    }
}

//
// Struct statements
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StructStmt {
    pub struct_type: TypeStmt,
    pub fields: Vec<StructFieldStmt>,
}

impl Parse for StructStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("StructStmt parse {:?}", &stack);
        let mut struct_stmt = StructStmt::default();

        // parse type statement
        let mut type_stack = take_all_until_match_any(&mut stack, vec![Ty::NL]);
        struct_stmt.struct_type = TypeStmt::parse(&mut type_stack)?;

        // parse struct fields
        let mut cleaned: Vec<Token> = remove_newline_indent(&mut stack)?;
        let mut field: Vec<Token> = vec![];
        while let Some(next) = cleaned.pop() {
            match next.ty {
                Ty::IdVar(_) => {
                    if field.len() > 0 {
                        field.reverse();
                        struct_stmt.fields.push(StructFieldStmt::parse(&mut field)?);
                        if field.len() > 0 {
                            debug!(
                                "field variable should have no length after parsing StructField"
                            );
                            field.clear();
                        }
                    }
                }
                _ => {}
            }
            field.push(next);
        }

        // flush field if it has a length
        if field.len() > 0 {
            field.reverse();
            struct_stmt.fields.push(StructFieldStmt::parse(&mut field)?);
        }

        Ok(struct_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StructFieldStmt {
    pub field_name: String,
    pub field_type: TypeStmt,
}

impl Parse for StructFieldStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("StructFieldStmt parse {:?}", &stack);
        let mut struct_field = StructFieldStmt::default();

        // IdVar
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdVar(name) => {
                    struct_field.field_name = name;
                }
                _ => {
                    bail!("expected IdVar");
                }
            }
        } else {
            bail!("expected IdVar");
        }
        // Colon
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctColon => {}
                _ => {
                    bail!("struct field expected colon after variable identity");
                }
            }
        } else {
            bail!("struct field expected colon after variable identity");
        }
        // TypeStmt
        struct_field.field_type = TypeStmt::parse(stack)?;

        Ok(struct_field)
    }

    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StructValueStmt {
    pub struct_type: TypeStmt,
    pub fields: Vec<StructFieldValueStmt>,
}

impl Parse for StructValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("StructValueStmt parse {:?}", &stack);
        let mut struct_value_stmt = StructValueStmt::default();

        // parse type statement
        struct_value_stmt.struct_type = TypeStmt::parse(stack)?;

        // eat starting and ending braces
        if let Some(next) = stack.pop() {
            match next.ty {
                Ty::PunctBraceL => {
                    // last token should be PunctBraceR
                    match stack.first() {
                        Some(tok) => {
                            if tok.ty == Ty::PunctBraceR {
                                stack.remove(0);
                            }
                        }
                        None => {}
                    }
                }
                _ => stack.push(next),
            }
        }

        let mut field: Vec<Token> = vec![];
        while let Some(next) = stack.pop() {
            match next.ty {
                Ty::IdVar(_) => {
                    // Handle expr statements by checking
                    // if the next token is a colon. This make everything
                    // after the colon and expression
                    let mut next_colon = false;
                    if let Some(next) = stack.pop() {
                        match next.ty {
                            Ty::PunctColon => {
                                next_colon = true;
                            }
                            _ => {}
                        }
                        stack.push(next);
                    }
                    if next_colon && field.len() > 0 {
                        // fields with enum struct values
                        // might end up with a PunctBraceR
                        // and we need to remove them
                        if let Some(last) = field.pop() {
                            match last.ty {
                                // eat PunctBraceR
                                Ty::PunctBraceR => {}
                                _ => field.push(last),
                            }
                        }
                        field.reverse();
                        struct_value_stmt
                            .fields
                            .push(StructFieldValueStmt::parse(&mut field)?);
                        if field.len() > 0 {
                            debug!(
                                "field variable should have no length after parsing StructFieldValue"
                            );
                            field.clear();
                        }
                    }
                }
                _ => {}
            }
            field.push(next);
        }

        // flush field if it has a length
        if field.len() > 0 {
            // fields with enum struct values
            // might end up with a PunctBraceR
            // and we need to remove them
            if let Some(last) = field.pop() {
                match last.ty {
                    Ty::PunctBraceR => {}
                    _ => field.push(last),
                }
            }
            field.reverse();
            struct_value_stmt
                .fields
                .push(StructFieldValueStmt::parse(&mut field)?);
        }

        Ok(struct_value_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StructFieldValueStmt {
    pub field_name: String,
    pub field_value: ExprStmt,
}

impl Parse for StructFieldValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("StructFieldValueStmt parse {:?}", &stack);
        let mut struct_field_value = StructFieldValueStmt::default();

        // IdVar
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdVar(name) => {
                    struct_field_value.field_name = name;
                }
                _ => {
                    bail!("expected IdVar");
                }
            }
        } else {
            bail!("expected IdVar");
        }
        // Colon
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctColon => {}
                _ => {
                    bail!("struct field expected colon after variable identity");
                }
            }
        } else {
            bail!("struct field expected colon after variable identity");
        }

        // ExprStmt
        struct_field_value.field_value = ExprStmt::parse(stack)?;

        Ok(struct_field_value)
    }

    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

//
// Tuple statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TupleStmt {
    pub types: Vec<TypeStmt>,
}
impl Parse for TupleStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("TupleStmt parse {:?}", stack);
        let mut tuple_type = TupleStmt::default();
        loop {
            let type_stmt = TypeStmt::parse(&mut stack)?;
            tuple_type.types.push(type_stmt);
            if stack.len() == 0 {
                break;
            }
        }
        Ok(tuple_type)
    }

    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

//
// Tuple value statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TupleValueStmt {
    fields: Vec<TupleFieldValueStmt>,
}

impl Parse for TupleValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("TupleValueStmt parse {:?}", stack);
        warn!("TupleValueStmt parsing not currently implemented!!!");
        let tup_val = TupleValueStmt::default();
        Ok(tup_val)
    }
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TupleFieldValueStmt {
    pub field_value: ExprStmt,
}
impl Parse for TupleFieldValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("TupleFieldValueStmt {:?}", stack);
        let tup_field_val = TupleFieldValueStmt::default();
        Ok(tup_field_val)
    }
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

//
// Function value statements
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncValueStmt {
    pub func_name: String,
    pub args: Vec<FuncArgValueStmt>,
}

impl Parse for FuncValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncValueStmt {:?}", stack);
        let mut func_val = FuncValueStmt::default();

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdFunc(name) => {
                    func_val.func_name = name;
                }
                _ => {
                    bail!("expected IdFunc to parse FuncValueStmt");
                }
            }
        }

        // eat starting and ending braces
        if let Some(next) = stack.pop() {
            match next.ty {
                Ty::PunctBraceL => {
                    // last token should be PunctBraceR
                    match stack.first() {
                        Some(tok) => {
                            if tok.ty == Ty::PunctBraceR {
                                stack.remove(0);
                            }
                        }
                        None => {}
                    }
                }
                _ => stack.push(next),
            }
        }

        let mut arg: Vec<Token> = vec![];
        while let Some(next) = stack.pop() {
            match next.ty {
                Ty::IdVar(_) => {
                    let mut next_colon = false;
                    if let Some(next) = stack.pop() {
                        match next.ty {
                            Ty::PunctColon => {
                                next_colon = true;
                            }
                            _ => {}
                        }
                        stack.push(next);
                    }

                    if next_colon && arg.len() > 0 {
                        arg.reverse();
                        func_val.args.push(FuncArgValueStmt::parse(&mut arg)?);
                        if arg.len() > 0 {
                            debug!(
                                "arg variable should have no length after parsing FuncArgValueStmt"
                            );
                            arg.clear();
                        }
                    }
                }
                _ => {}
            }
            arg.push(next);
        }

        // flush arg if it has a length
        if arg.len() > 0 {
            arg.reverse();
            func_val.args.push(FuncArgValueStmt::parse(&mut arg)?);
        }

        Ok(func_val)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncArgValueStmt {
    pub arg_name: String,
    pub arg_value: ExprStmt,
}

impl Parse for FuncArgValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncArgValueStmt {:?}", stack);
        let mut func_arg_val = FuncArgValueStmt::default();

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdVar(name) => {
                    func_arg_val.arg_name = name;
                }
                _ => {
                    bail!("FuncArgValueStmt expected IdVar");
                }
            }
        } else {
            bail!("FuncArgValueStmt expected IdVar");
        }
        // Colon
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctColon => {}
                _ => {
                    bail!("FuncArgValueStmt expected colon after IdVar");
                }
            }
        } else {
            bail!("FuncArgValueStmt expected colon after IdVar");
        }

        // parse this as an ExprStmt
        func_arg_val.arg_value = ExprStmt::parse(stack)?;

        Ok(func_arg_val)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

//
// Let statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LetStmt {
    pub assign: AssignStmt,
}

impl Parse for LetStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("LetStmt parse {:?}", stack);
        let assign = AssignStmt::parse(stack)?;
        Ok(LetStmt { assign })
    }

    fn validate(&self) -> Result<(), Error> {
        // this should validate that the op is an assignment eq
        Ok(())
    }
}

//
// Variable statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VarStmt {
    pub var_name: String,
    pub assign: AssignStmt,
}

//
// Expression statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
}

impl Parse for ExprStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("ExprStmt parse {:?}", stack);
        // remove newlines and indentation
        let mut cleaned = remove_newline_indent(&mut stack)?;
        cleaned.reverse();
        // debug!("ExprStmt cleaned {:?}", cleaned);
        let mut expr_stmt = ExprStmt::default();
        match ExprParser.parse(cleaned.into_iter()) {
            Ok(expr) => expr_stmt.expr = expr,
            Err(e) => bail!(e.to_string()),
        }
        Ok(expr_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // None is the default to avoid a stack-overflow
    // because both UnaryExprStmt or BinaryExprStmt define
    // left and/od right Expr.
    // TODO: add validation around Expr::None
    None,
    Value(ValueStmt),
    Unary(UnaryExprStmt),
    Binary(BinaryExprStmt),
}

impl Default for Expr {
    fn default() -> Self {
        Expr::None
    }
}

//
// Expression value statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValueStmt {
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    String(String),
    Bool(bool),
    Int(isize),
    UInt(usize),
    Float(f64),
    Func(FuncValueStmt),
    Struct(StructValueStmt),

    // Anything defined by:
    // const, let, func args
    Var(String),

    // TBD how this works
    Package(String),
    Enum(EnumValueStmt),
    Tuple(TupleValueStmt),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

//
// Expression statements and operators
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnaryExprStmt {
    pub op: UnaryOp,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BinaryExprStmt {
    pub lhs: Box<Expr>,
    pub op: Op,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Assign(AssignOp),
    Boolean(BooleanOp),
    Logical(LogicalOp),
    Bitwise(BitwiseOp),
    Arith(ArithOp),
}

impl Default for Op {
    fn default() -> Self {
        Op::Assign(AssignOp::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    // =
    Eq,
    // +=
    Add,
    // -=
    Sub,
    // /=
    Div,
    // //=
    FloorDiv,
    // *=
    Mul,
    // **=
    Pow,
    // %=
    Mod,
    // &=
    BitwiseAnd,
    // |=
    BitwiseOr,
    // ^=
    BitwiseXOr,
    // <<=
    BitwiseShiftL,
    // >>=
    BitwiseShiftR,
}

impl Default for AssignOp {
    fn default() -> Self {
        AssignOp::Eq
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BooleanOp {
    // ==
    EqEq,
    // !=
    NotEq,
    // >=
    Gte,
    // <
    Gt,
    // <=
    Lte,
    // <
    Lt,
}

impl Default for BooleanOp {
    fn default() -> Self {
        BooleanOp::EqEq
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    // &&
    And,
    // ||
    Or,
    // !
    Not,
}

impl Default for LogicalOp {
    fn default() -> Self {
        LogicalOp::And
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitwiseOp {
    // &
    And,
    // |
    Or,
    // ^
    XOr,
    // <<
    ShiftL,
    // >>
    ShiftR,
}

impl Default for BitwiseOp {
    fn default() -> Self {
        BitwiseOp::And
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithOp {
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // /
    Div,
    // //
    FloorDiv,
    // %
    Mod,
    // **
    Pow,
}

impl Default for ArithOp {
    fn default() -> Self {
        ArithOp::Add
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    // Default
    None,
    // +
    Plus,
    // -
    Minus,
    // !
    Not,
}

impl Default for UnaryOp {
    fn default() -> Self {
        UnaryOp::None
    }
}

//
// Function statements
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncStmt {
    pub func_name: String,
    pub func_type: FuncTypeStmt,
    pub func_body: FuncBodyStmt,
}

impl Parse for FuncStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncStmt {:?}", stack);
        let mut func_stmt = FuncStmt::default();

        // IdFunc
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdFunc(name) => {
                    func_stmt.func_name = name;
                }
                _ => {
                    bail!("expected IdFunc");
                }
            }
        }
        // this should take
        func_stmt.func_type = FuncTypeStmt::parse(stack)?;

        // parse body stmt
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctFatArrow => {
                    func_stmt.func_body = FuncBodyStmt::parse(stack)?;
                }
                _ => {
                    bail!("expected to parse a function body")
                }
            }
        }

        Ok(func_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncTypeStmt {
    pub is_self: bool,
    pub input_type: Option<FuncInputTypeStmt>,
    pub output_type: Option<FuncOutputTypeStmt>,
}

impl Parse for FuncTypeStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncTypeStmt parse {:?}", stack);
        let mut func_ty_stmt = FuncTypeStmt::default();

        // check for self
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::KwSelf => {
                    func_ty_stmt.is_self = true;
                }
                _ => {
                    stack.push(tok);
                }
            }
        }

        // parse input args...
        // check for input args PunctBraceL
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctBraceL => {
                    // collect input args here...
                    let mut input_type_stack =
                        take_all_until_match_any(stack, vec![Ty::PunctBraceR]);
                    debug!("input_type: {:?}", &input_type_stack);
                    if input_type_stack.len() > 0 {
                        let input_type = FuncInputTypeStmt::parse(&mut input_type_stack)?;
                        func_ty_stmt.input_type = Some(input_type);
                    }
                    // pop PunctBraceR
                    if let Some(tok) = stack.pop() {
                        match tok.ty {
                            Ty::PunctBraceR => {}
                            _ => {
                                stack.push(tok);
                            }
                        }
                    }
                }
                _ => {
                    stack.push(tok);
                }
            }
        }

        // parse output args...
        // take until fat arrow
        let mut output_type_stack = take_all_until_match_any(stack, vec![Ty::PunctFatArrow]);
        if output_type_stack.len() > 0 {
            let output_type = FuncOutputTypeStmt::parse(&mut output_type_stack)?;
            func_ty_stmt.output_type = Some(output_type);
        }
        Ok(func_ty_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncInputTypeStmt {
    pub args: Vec<FuncInputArgTypeStmt>,
}

impl Parse for FuncInputTypeStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncInputTypeStmt parse {:?}", stack);
        let mut func_in_ty_stmt = FuncInputTypeStmt::default();

        // parse input args
        let mut cleaned: Vec<Token> = remove_newline_indent(stack)?;
        let mut arg: Vec<Token> = vec![];
        while let Some(next) = cleaned.pop() {
            match next.ty {
                Ty::IdVar(_) => {
                    if arg.len() > 0 {
                        arg.reverse();
                        func_in_ty_stmt
                            .args
                            .push(FuncInputArgTypeStmt::parse(&mut arg)?);
                        if arg.len() > 0 {
                            debug!(
                                "input arg variable should have no length after parsing FuncInputTypeStmt"
                            );
                            arg.clear();
                        }
                    }
                }
                _ => {}
            }
            arg.push(next);
        }

        // flush arg if it has a length
        if arg.len() > 0 {
            arg.reverse();
            func_in_ty_stmt
                .args
                .push(FuncInputArgTypeStmt::parse(&mut arg)?);
        }

        Ok(func_in_ty_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncInputArgTypeStmt {
    pub arg_name: String,
    pub arg_type: TypeStmt,
}

impl Parse for FuncInputArgTypeStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncInputArgTypeStmt parse {:?}", stack);
        let mut func_in_arg_ty_stmt = FuncInputArgTypeStmt::default();
        // IdVar
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdVar(name) => {
                    func_in_arg_ty_stmt.arg_name = name;
                }
                _ => {
                    bail!("FuncInputArgTypeStmt expected IdVar");
                }
            }
        } else {
            bail!("FuncInputArgTypeStmt expected IdVar");
        }
        // Colon
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctColon => {}
                _ => {
                    bail!("FuncInputArgTypeStmt expected colon after IdVar");
                }
            }
        } else {
            bail!("FuncInputArgTypeStmt expected colon after IdVar");
        }
        // TypeStmt
        func_in_arg_ty_stmt.arg_type = TypeStmt::parse(stack)?;

        Ok(func_in_arg_ty_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncOutputTypeStmt {
    pub output_type: TypeStmt,
}

impl Parse for FuncOutputTypeStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncOutputTypeStmt parse {:?}", stack);
        let mut func_out_ty_stmt = FuncOutputTypeStmt::default();
        func_out_ty_stmt.output_type = TypeStmt::parse(stack)?;
        Ok(func_out_ty_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncBodyStmt {
    pub blocks: Vec<BlockStmt>,
}

impl Parse for FuncBodyStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("FuncBodyStmt parse {:?}", stack);
        let mut func_body_stmt = FuncBodyStmt::default();

        // we need to detect the starting indentation
        let mut indent = 0usize;
        while let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::NL => {}
                Ty::Indent(val) => {
                    indent = val;
                }
                _ => {
                    if indent == 0usize {
                        bail!("expected indentation greater then 0 in function body");
                    }
                    // add back to stack
                    stack.push(tok);

                    let mut block_stmt = BlockStmt::default();

                    // println!("stack before: {:?}", &stack);
                    // This needs to take an entire statement we can parse
                    // and could span multiple lines (for example if/elif/else)
                    // we need to take until the next indent with the same value
                    let mut block_stack = take_all_until_match_any(stack, vec![Ty::Indent(indent)]);
                    // println!("block_stack before: {:?}", &block_stack);

                    if block_stack.contains(&Token { ty: Ty::KwIf }) {
                        debug!(
                            "block stack contains if stmt. continue parsing and take entire stmt {:?}",
                            &stack
                        );

                        // this should be an indent + (KwElseIf or KwElse) or we're done
                        loop {
                            if let Some(tok) = stack.pop() {
                                match tok.ty {
                                    Ty::Indent(val) => {
                                        if indent != val {
                                            bail!("expected same indentation level");
                                        }
                                    }
                                    _ => {
                                        stack.push(tok);
                                        break;
                                    }
                                }
                            }

                            let mut next_stack =
                                take_all_until_match_any(stack, vec![Ty::Indent(indent)]);
                            if next_stack.contains(&Token { ty: Ty::KwElseIf })
                                || next_stack.contains(&Token { ty: Ty::KwElse })
                            {
                                // merge w/ block_stack
                                next_stack.append(&mut vec![Token {
                                    ty: Ty::Indent(indent),
                                }]);
                                next_stack.append(&mut block_stack);
                                block_stack = next_stack;
                            } else {
                                // add back to stack
                                stack.append(&mut next_stack);
                                break;
                            }
                        }
                        // println!("stack after: {:?}", &stack);
                        // println!("block_stack after: {:?}", &block_stack);
                        // panic!("check");
                    }

                    block_stmt.indent = indent;
                    block_stmt = block_stmt.parse(&mut block_stack)?;
                    func_body_stmt.blocks.push(block_stmt);
                }
            }
        }

        Ok(func_body_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

//
// Block statements
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockStmt {
    indent: usize,
    pub blocks: Vec<Block>,
    pub return_type: Option<TypeStmt>,
}

impl ParseSelf for BlockStmt {
    fn parse(mut self, stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("BlockStmt parse {:?}", stack);

        // TODO: this assumes the block fits nicely on one line...
        //       we might need to account for multiline statements
        //       similar to how we can initialize structs

        let mut indent = self.indent.clone();
        while let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::NL => {}
                Ty::Indent(val) => {
                    indent = val;
                    if indent > self.indent {
                        // recursive decent
                        let mut block_stmt = BlockStmt::default();
                        // we need to take until the next indent with the same value
                        let mut block_stack =
                            take_all_until_match_any(stack, vec![Ty::Indent(indent)]);
                        block_stmt.indent = indent;
                        block_stmt = block_stmt.parse(&mut block_stack)?;
                        let block = Block::Block(Box::new(block_stmt));
                        self.blocks.push(block);
                    }
                }
                Ty::KwConst => {
                    let const_stmt = ConstStmt::parse(stack)?;
                    let block = Block::Const(const_stmt);
                    self.blocks.push(block);
                }
                Ty::KwLet => {
                    let let_stmt = LetStmt::parse(stack)?;
                    let block = Block::Let(let_stmt);
                    self.blocks.push(block);
                }
                Ty::KwFor => {
                    todo!("block for stmt");
                }
                Ty::KwIf => {
                    // add back if so we can parse later using a while loop
                    stack.push(tok);
                    let mut if_stmt = IfStmt::default();
                    if_stmt.indent = indent;
                    if_stmt = if_stmt.parse(stack)?;
                    let block = Block::If(if_stmt);
                    self.blocks.push(block);
                }
                Ty::KwMatch => {
                    todo!("block match stmt");
                }
                Ty::KwReturn => {
                    // take until NL
                    let mut ret_stack = take_all_until_match_any(stack, vec![Ty::NL]);
                    let ret_stmt = ReturnStmt::parse(&mut ret_stack)?;
                    let block = Block::Return(ret_stmt);
                    self.blocks.push(block);
                }
                Ty::IdVar(_) => {
                    // take until NL
                    stack.push(tok);
                    let mut assign_stack = take_all_until_match_any(stack, vec![Ty::NL]);
                    let assign_stmt = AssignStmt::parse(&mut assign_stack)?;
                    let block = Block::Assign(assign_stmt);
                    self.blocks.push(block);
                }
                Ty::IdFunc(_) => {
                    // take until NL
                    stack.push(tok);
                    let expr_stmt = ExprStmt::parse(stack)?;
                    let block = Block::Expr(expr_stmt);
                    self.blocks.push(block);
                }
                Ty::Comment(_) => {}
                _ => {
                    // unexpected token
                    bail!("unexpected token parsing BlockStmt: {:?}", &tok.ty);
                }
            }
        }

        Ok(self)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    None,
    Assign(AssignStmt),
    Block(Box<BlockStmt>),
    Const(ConstStmt),
    Expr(ExprStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    If(IfStmt),
    Let(LetStmt),
    Match(MatchStmt),
    Return(ReturnStmt),
    While(WhileStmt),
}

impl Default for Block {
    fn default() -> Self {
        Block::None
    }
}

//
// Return statement
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReturnStmt {
    pub expr: ExprStmt,
    pub return_type: Option<TypeStmt>,
}

impl Parse for ReturnStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("ReturnStmt parse {:?}", stack);
        let mut ret_stmt = ReturnStmt::default();
        ret_stmt.expr = ExprStmt::parse(stack)?;
        // fill this in during a later pass
        // ret_stmt.return_type = TypeStmt::parse(stack)?;
        Ok(ret_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

//
// If statements
//
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IfStmt {
    indent: usize,
    // if ...
    pub if_cond: IfConditionStmt,
    // elif ...
    pub elif_cond: Vec<IfConditionStmt>,
    // else ...
    pub else_cond: Option<IfElseStmt>,
}

impl ParseSelf for IfStmt {
    fn parse(mut self, stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("IfStmt parse {:?}", stack);
        let mut indent = self.indent;
        while let Some(tok) = stack.pop() {
            // debug!("IfStmt token {:?} (indent {})", tok.ty, &indent);
            match tok.ty {
                Ty::NL => {}
                Ty::Indent(val) => {
                    indent = val;
                    debug!("IfStmt indent: {}", indent);
                    // parse inner block here?
                }
                Ty::KwIf => {
                    let mut if_cond = IfConditionStmt::default();
                    let mut if_stack = take_all_until_match_any(stack, vec![Ty::Indent(indent)]);
                    if_cond.indent = indent;
                    if_cond = if_cond.parse(&mut if_stack)?;
                    self.if_cond = if_cond;
                }
                Ty::KwElseIf => {
                    let mut elif_cond = IfConditionStmt::default();
                    let mut elif_stack = take_all_until_match_any(stack, vec![Ty::Indent(indent)]);
                    elif_cond.indent = indent;
                    elif_cond = elif_cond.parse(&mut elif_stack)?;
                    self.elif_cond.push(elif_cond);
                }
                Ty::KwElse => {
                    let mut else_cond = IfElseStmt::default();
                    let mut else_stack = take_all_until_match_any(stack, vec![Ty::Indent(indent)]);
                    else_cond.indent = indent;
                    else_cond = else_cond.parse(&mut else_stack)?;
                    self.else_cond = Some(else_cond);
                }
                _ => {}
            }
        }

        Ok(self)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IfConditionStmt {
    indent: usize,
    pub condition: ConditionStmt,
    pub blocks: Vec<BlockStmt>,
}

impl ParseSelf for IfConditionStmt {
    fn parse(mut self, stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("IfConditionStmt parse {:?}", stack);

        // take until KwThen...
        let mut cond_stack = take_all_until_match_any(stack, vec![Ty::KwThen]);
        self.condition = ConditionStmt::parse(&mut cond_stack)?;

        // should be KwThen
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::KwThen => {}
                _ => {
                    stack.push(tok);
                    bail!("expected KWThen");
                }
            }
        }

        loop {
            let mut block_stack = take_all_until_match_any(stack, vec![Ty::Indent(self.indent)]);
            if block_stack.len() == 0 {
                break;
            }
            let mut block_stmt = BlockStmt::default();
            block_stmt.indent = self.indent;
            block_stmt = block_stmt.parse(&mut block_stack)?;
            self.blocks.push(block_stmt);
        }
        Ok(self)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IfElseStmt {
    indent: usize,
    pub blocks: Vec<BlockStmt>,
}

impl ParseSelf for IfElseStmt {
    fn parse(mut self, stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("IfElseStmt parse {:?}", stack);
        loop {
            let mut block_stack = take_all_until_match_any(stack, vec![Ty::Indent(self.indent)]);
            if block_stack.len() == 0 {
                break;
            }
            let mut block_stmt = BlockStmt::default();
            block_stmt.indent = self.indent;
            block_stmt = block_stmt.parse(&mut block_stack)?;
            self.blocks.push(block_stmt);
        }

        Ok(self)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

// ConditionStmt
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConditionStmt {
    pub expr: ExprStmt,
}

impl Parse for ConditionStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        debug!("ConditionStmt parse {:?}", stack);
        let mut cond_stmt = ConditionStmt::default();
        cond_stmt.expr = ExprStmt::parse(stack)?;
        Ok(cond_stmt)
    }
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

///
///
///
///
///
///
///
/// Not implemented yet
///
///
///
///
///
///
///

#[derive(Debug, Clone, Default, PartialEq)]
struct ForStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct ForInStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct MatchStmt {
    value: ValueStmt,
    arms: Vec<MatchArmStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct MatchArmStmt {
    arm_expr: ExprStmt,
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct WhileStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct TraitStmt {}

// Struct and Enum trait implementations
#[derive(Debug, Clone, Default, PartialEq)]
struct ImplTraitStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct TestStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct TestCaseStmt {}
