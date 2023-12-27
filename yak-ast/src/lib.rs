#![allow(dead_code)]
#![allow(private_interfaces)]

mod expr;
mod pratt;

mod test;

use anyhow::{bail, Context, Error, Result};
use expr::ExprParser;
use log::{error, info};
use pratt::PrattParser;
use std::fs;
use std::path::PathBuf;
use yak_core::models::yak_package::{
    Symbol, YakDependency, YakExport, YakFile, YakImport, YakPackage, YakSymbol,
};
use yak_core::models::yak_version::YakVersion;
use yak_lexer::token::TokenType as Ty;
use yak_lexer::{Lexer, Token};

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
            // println!("ast.parse {:?}", token.ty);
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
                                    println!("const_stmt {:#?}", const_stmt);
                                    self.parsed.consts.push(const_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwEnum => {
                            let stmt = EnumStmt::parse(&mut stack);
                            match stmt {
                                Ok(enum_stmt) => {
                                    println!("enum_stmt {:#?}", enum_stmt);
                                    self.parsed.enums.push(enum_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwFn => {
                            todo!("parse fn")
                        }
                        Ty::KwImpl => {
                            todo!("parse impl")
                        }
                        Ty::KwLet => {
                            let stmt = LetStmt::parse(&mut stack);
                            match stmt {
                                Ok(let_stmt) => {
                                    println!("let_stmt {:#?}", let_stmt);
                                    self.parsed.lets.push(let_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwStruct => {
                            let stmt = StructStmt::parse(&mut stack);
                            match stmt {
                                Ok(struct_stmt) => {
                                    println!("struct_stmt {:#?}", struct_stmt);
                                    self.parsed.structs.push(struct_stmt);
                                }
                                Err(err) => return Some(err),
                            }
                        }
                        Ty::KwTest => {
                            todo!("parse test")
                        }
                        Ty::KwTestCase => {
                            todo!("parse testcase")
                        }
                        Ty::KwTrait => {
                            todo!("parse ^Trait")
                        }
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
    // println!("checking for pattern: {:?}", &pattern_iter);

    let pattern_ty = pattern_iter.next().unwrap();
    let mut taken: Vec<Token> = vec![];

    while let Some(tok) = tokens.pop() {
        // println!("> tok: {:?} pattern: {:?}", &tok.ty, &pattern_ty);
        if &tok.ty == &pattern_ty {
            taken.push(tok);
            while let Some(pattern_next_ty) = pattern_iter.next() {
                if tokens.len() > 0 {
                    let next = tokens.pop().unwrap();
                    // println!(
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
    println!("into_type_stmt_generics {:?}", stack);

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
                    println!("failed on: {:?}", tok);
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

//
// **** Ast Stmts
//

//
// ConstStmt
//
#[derive(Debug, Clone, Default, PartialEq)]
struct ConstStmt {
    assign: AssignStmt,
}

impl Parse for ConstStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("ConstStmt {:?}", stack);
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
struct AssignStmt {
    var_type: VarTypeStmt,
    op: Op,
    expr: ExprStmt,
}

impl Parse for AssignStmt {
    // VarTypeStmt Op Expr
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
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
struct VarTypeStmt {
    var_name: String,
    var_type: Option<TypeStmt>,
}

impl Parse for VarTypeStmt {
    // VarTypeStmt Op Expr
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        // VarTypeStmt
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
struct TypeStmt {
    type_name: String,
    generics: Option<Box<Vec<TypeStmt>>>,
}
impl Parse for TypeStmt {
    // TypeStmt
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("parse type_stmt {:?}", stack);
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
                        println!("failed on: {:?}", tok);
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

        println!("has_generics {:?}", &has_generics);
        if let Some(true) = has_generics {
            // parse inner generic type stmts
            // [ T ]

            // We need to account for StructValue types here (i.e. stop before )
            let mut inner = take_all_until_match_any(stack, vec![Ty::PunctBraceL]);
            println!("generics {:?}", &inner);

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
struct EnumStmt {
    enum_name: String,
    variants: Vec<EnumVariantStmt>,
}

impl Parse for EnumStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("EnumStmt {:?}", stack);

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
            // println!("-----");
        }

        // let variants = vec![];
        Ok(enum_stmt)
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EnumVariantType {
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
struct EnumVariantStmt {
    variant_name: String,
    variant_type: EnumVariantType,
}

impl Parse for EnumVariantStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        // remove all ident + NL
        let mut cleaned: Vec<Token> = remove_newline_indent(&mut stack)?;
        println!("EnumVariantStmt {:?} (cleaned)", cleaned);
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
                    // println!("\n####");
                    // println!("before {:?}", &cleaned);
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
                    // println!("after {:?}", &cleaned);
                    // println!("\n####");
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
                            println!("failed on: {:?}", next);
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
struct EnumStructStmt {
    fields: Vec<StructFieldStmt>,
}

impl Parse for EnumStructStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        let mut enum_struct = EnumStructStmt::default();

        let mut field: Vec<Token> = vec![];
        while let Some(next) = stack.pop() {
            match next.ty {
                Ty::IdVar(_) => {
                    if field.len() > 0 {
                        field.reverse();
                        enum_struct.fields.push(StructFieldStmt::parse(&mut field)?);

                        if field.len() > 0 {
                            println!(
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
struct EnumValueStmt {
    enum_name: String,
    variant_name: String,
    variant_value_type: EnumVariantValueType,
}

impl Parse for EnumValueStmt {
    fn parse(_stack: &mut Vec<Token>) -> Result<Self, Error> {
        todo!("missing EnumValueStmt")
    }
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EnumVariantValueType {
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
// Struct statement
//
#[derive(Debug, Clone, Default, PartialEq)]
struct StructStmt {
    struct_type: TypeStmt,
    fields: Vec<StructFieldStmt>,
}

impl Parse for StructStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("StructStmt parse {:?}", &stack);
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
                            println!(
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
struct StructFieldStmt {
    field_name: String,
    field_type: TypeStmt,
}

impl Parse for StructFieldStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
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
struct StructValueStmt {
    struct_type: TypeStmt,
    fields: Vec<StructFieldValueStmt>,
}

impl Parse for StructValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("StructValueStmt parse {:?}", &stack);
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
                    if field.len() > 0 {
                        field.reverse();
                        struct_value_stmt
                            .fields
                            .push(StructFieldValueStmt::parse(&mut field)?);
                        if field.len() > 0 {
                            println!(
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
struct StructFieldValueStmt {
    field_name: String,
    field_value: ExprStmt,
}

impl Parse for StructFieldValueStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("StructFieldValueStmt parse {:?}", &stack);
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
// Tuple type statement
//
#[derive(Debug, Clone, Default, PartialEq)]
struct TupleStmt {
    types: Vec<TypeStmt>,
}

impl Parse for TupleStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        let mut tuple_type = Self::default();
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

#[derive(Debug, Clone, Default, PartialEq)]
struct TupleValueStmt {
    fields: Vec<TupleFieldValueStmt>,
}

impl Parse for TupleValueStmt {
    fn parse(mut _stack: &mut Vec<Token>) -> Result<Self, Error> {
        todo!("missing ")
    }
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct TupleFieldValueStmt {
    field_value: ExprStmt,
}
impl Parse for TupleFieldValueStmt {
    fn parse(_stack: &mut Vec<Token>) -> Result<Self, Error> {
        todo!("missing TupleFieldValueStmt")
    }
    fn validate(&self) -> Result<(), Error> {
        todo!()
    }
}

//
// Func statements
//
#[derive(Debug, Clone, Default, PartialEq)]
struct FuncValueStmt {
    func_name: String,
    input_args: FuncInputValueStmt,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncInputValueStmt {
    args: Vec<FuncInputArgValueStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncInputArgValueStmt {
    arg_name: String,
    arg_value: ExprStmt,
}

//
// Let statement
//
#[derive(Debug, Clone, Default, PartialEq)]
struct LetStmt {
    assign: AssignStmt,
}

impl Parse for LetStmt {
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("LetStmt {:?}", stack);
        let assign = AssignStmt::parse(stack)?;
        Ok(LetStmt { assign })
    }

    fn validate(&self) -> Result<(), Error> {
        // this should validate that the op is an assignment eq
        Ok(())
    }
}

// Variables

#[derive(Debug, Clone, Default, PartialEq)]
struct VarStmt {
    var_name: String,
    assign: AssignStmt,
}

// ExprStmt
#[derive(Debug, Clone, Default, PartialEq)]
struct ExprStmt {
    expr: Expr,
}

impl Parse for ExprStmt {
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
        println!("ExprStmt {:?}", stack);
        // remove newlines and indentation
        let mut cleaned = remove_newline_indent(&mut stack)?;
        cleaned.reverse();
        // println!("ExprStmt cleaned {:?}", cleaned);
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
enum Expr {
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

#[derive(Debug, Clone, Default, PartialEq)]
struct ValueStmt {
    value: Value,
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    None,
    String(String),
    Bool(bool),
    Int(isize),
    UInt(usize),
    Float(f64),
    Struct(StructValueStmt),

    // Anything defined by:
    // const, let, func args
    Var(String),

    // TBD how this works
    Package(String),
    Func(FuncValueStmt),
    Enum(EnumValueStmt),
    Tuple(TupleValueStmt),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct UnaryExprStmt {
    op: UnaryOp,
    rhs: Box<Expr>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct BinaryExprStmt {
    lhs: Box<Expr>,
    op: Op,
    rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
enum Op {
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
enum AssignOp {
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
enum BooleanOp {
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
enum LogicalOp {
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
enum BitwiseOp {
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
enum ArithOp {
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
enum UnaryOp {
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

///
///
///
///
///
///
///
/// Not implement yet

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncStmt {
    func_name: String,
    func_type: FuncTypeStmt,
    func_body: FuncBodyStmt,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncTypeStmt {
    is_self: bool,
    input_type: Option<FuncInputTypeStmt>,
    output_type: Option<FuncOutputTypeStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncInputTypeStmt {
    args: Vec<FuncInputArgTypeStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncInputArgTypeStmt {
    arg_name: String,
    arg_type: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncOutputTypeStmt {
    output_type: TypeStmt,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FuncBodyStmt {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ForStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct ForInStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct IfStmt {
    // if ...
    if_: IfConditionStmt,
    // elif ...
    elif_: Option<Vec<IfConditionStmt>>,
    // else ...
    else_: Option<IfConditionStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct IfConditionStmt {
    condition: ConditionStmt,
    block: BlockStmt,
}

// ConditionStmt might be able to wrap an ExprStmt
// and it needs to test if the ExprStmt returns a boolean
#[derive(Debug, Clone, Default, PartialEq)]
struct ConditionStmt {
    expr: ExprStmt,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ReturnStmt {
    expr: ExprStmt,
    return_type: Option<TypeStmt>,
}

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
struct TraitStmt {}

// Struct and Enum trait implementations
#[derive(Debug, Clone, Default, PartialEq)]
struct ImplTraitStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct TestStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct TestCaseStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct WhileStmt {}

#[derive(Debug, Clone, Default, PartialEq)]
struct BlockStmt {
    blocks: Vec<Block>,
    return_type: Option<TypeStmt>,
}

#[derive(Debug, Clone, PartialEq)]
enum Block {
    None,
    Assign(AssignStmt),
    Block(Box<BlockStmt>),
    For(ForStmt),
    ForIn(ForInStmt),
    If(IfStmt),
    Let(LetStmt),
    Const(ConstStmt),
    Match(MatchStmt),
    Return(ReturnStmt),
    While(WhileStmt),
}

impl Default for Block {
    fn default() -> Self {
        Block::None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageStmt {
    package_id: String,
    description: String,
    version: String,
    files: Vec<PackageFileStmt>,
    dependencies: Vec<PackageDependencyStmt>,
    imports: Vec<PackageImportStmt>,
    exports: PackageExportStmt,
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
                    println!("deps stack: {:?}\n", stack);
                    // can't reuse deps here because of how anyhow results work
                    let mut deps = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut deps = remove_newline_indent_space(&mut deps)?;
                    println!("deps: {:?}\n\n", deps);

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
                    // println!("export stack: {:?}\n", stack);
                    let mut exports = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut exports = remove_newline_indent_space(&mut exports)?;
                    // println!("exports: {:?}\n\n", exports);

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
                            // trait
                            Ty::OpBitwiseXOr => {
                                // we have a trait next...
                                if let Some(next) = exports.pop() {
                                    match next.ty {
                                        Ty::IdType(ty) => {
                                            let mut sym_stmt = PackageSymbolStmt::default();
                                            sym_stmt.symbol =
                                                PackageSymbol::Trait(format!("^{}", ty));
                                            pkg_stmt.exports.symbols.push(sym_stmt);
                                        }
                                        _ => {
                                            bail!("failed to parse package export. Expected IdType for ^Trait")
                                        }
                                    }
                                }
                            }
                            Ty::PunctBraceR => {
                                break;
                            }
                            _ => {
                                bail!("failed to parse package export. Expected PunctBraceR")
                            }
                        }
                    }
                }
                Ty::KwImport => {
                    println!("import stack: {:?}\n", stack);
                    let mut imports = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut imports = remove_newline_indent_space(&mut imports)?;
                    println!("imports: {:?}\n\n", imports);

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
                                            println!("push next: {:?}", next);
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
                                                        println!("parse type sym: {:?}", sym);
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
                                                    // trait
                                                    Ty::OpBitwiseXOr => {
                                                        // we have a trait next...
                                                        let mut sym_stmt =
                                                            PackageSymbolStmt::default();
                                                        // primary
                                                        if let Some(next) = imports.pop() {
                                                            match next.ty {
                                                                Ty::IdType(ty) => {
                                                                    sym_stmt.symbol =
                                                                        PackageSymbol::Trait(
                                                                            format!("^{}", ty),
                                                                        );
                                                                }
                                                                _ => {
                                                                    bail!("failed to parse package import. Expected IdType after OpBitwiseXOr for trait")
                                                                }
                                                            }
                                                        }
                                                        // check alias
                                                        if let Some(next) = imports.pop() {
                                                            match next.ty {
                                                                Ty::KwAs => {
                                                                    if let Some(next) =
                                                                        imports.pop()
                                                                    {
                                                                        match next.ty {
                                                                    Ty::OpBitwiseXOr => {
                                                                        if let Some(next) = imports.pop() {
                                                                            match next.ty {
                                                                                Ty::IdType(ty) => {
                                                                                    sym_stmt.as_symbol =
                                                                                        Some(PackageSymbol::Trait(
                                                                                            format!("^{}", ty),
                                                                                        ));
                                                                                    imp_stmt.symbols.push(sym_stmt);
                                                                                }
                                                                                _ => {
                                                                                    bail!("failed to parse package import. KwAs doesn't match type Trait")
                                                                                }
                                                                            }
                                                                        }
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
                    // println!("files stack: {:?}\n", stack);
                    let mut files = take_all_include_pattern(
                        stack,
                        vec![Ty::NL, Ty::Indent(0), Ty::PunctBraceR],
                    )?;
                    let mut files = remove_newline_indent_space(&mut files)?;
                    // println!("files: {:?}\n\n", files);
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
struct PackageDependencyStmt {
    package_id: String,
    path: String,
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
struct PackageFileStmt {
    path: String,
}

impl Into<YakFile> for PackageFileStmt {
    fn into(self) -> YakFile {
        let mut yak_file = YakFile::default();
        yak_file.path = clean_quotes(self.path);
        yak_file
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageImportStmt {
    package_id: String,
    as_package_id: Option<String>,
    // we need to explicitly import symbols
    // from a package to make them available
    symbols: Vec<PackageSymbolStmt>,
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
struct PackageExportStmt {
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
enum PackageSymbol {
    None,
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
            PackageSymbol::Var(s) => Symbol::Var(clean_quotes(s)),
            PackageSymbol::Func(s) => Symbol::Func(clean_quotes(s)),
            PackageSymbol::Type(s) => Symbol::Type(clean_quotes(s)),
            PackageSymbol::Trait(s) => Symbol::Trait(clean_quotes(s)),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageSymbolStmt {
    symbol: PackageSymbol,
    as_symbol: Option<PackageSymbol>,
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
