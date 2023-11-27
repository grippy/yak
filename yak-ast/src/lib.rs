#![allow(dead_code)]

mod expr;
mod pratt;

mod test;

use yak_lexer::token::TokenType as Ty;
use yak_lexer::{Lexer, Token};

use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use expr::ExprParser;
use pratt::PrattParser;

fn err(msg: &str) -> Error {
    Error::new(ErrorKind::InvalidInput, msg)
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
    constants: Vec<ConstStmt>,
    enums: Vec<EnumStmt>,
    errors: Vec<Error>,
    funcs: Vec<FuncStmt>,
    impl_traits: Vec<ImplTraitStmt>,
    lets: Vec<LetStmt>,
    structs: Vec<StructStmt>,
    traits: Vec<TraitStmt>,
    tests: Vec<TestStmt>,
}

#[derive(Debug)]
pub struct Ast {
    stack: Vec<Token>,
    pub parsed: Parsed,
}

impl Ast {
    fn from_source(source: &str) -> Self {
        let mut lexer = Lexer::from_source(source);
        lexer.parse();
        Ast {
            stack: lexer.tokens_as_stack(),
            parsed: Parsed::default(),
        }
    }
    fn parse(&mut self) {
        // parse top-level rules
        while let Some(token) = self.stack.pop() {
            // println!("ast.parse {:?}", token.ty);
            match token.ty {
                Ty::Sp | Ty::NL => {}
                Ty::Comment(_) => {}
                Ty::Indent(indent) => {
                    if indent > 0 {
                        panic!("top-level indentation not allowed");
                    }
                    match self.top_level() {
                        Some(err) => self.parsed.errors.push(err),
                        _ => {}
                    }
                }
                _ => {
                    println!("toplevel failed to parse: {:?}", token);
                }
            }
        }
    }
    fn top_level(&mut self) -> Option<Error> {
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
                | Ty::KwStruct
                | Ty::KwImpl
                | Ty::KwTest
                | Ty::KwTestCase
                | Ty::KwTrait
                | Ty::KwFn
                | Ty::KwType
                // Package definitions
                | Ty::KwPackage
                | Ty::KwDescription
                | Ty::KwVersion
                | Ty::KwDependencies
                | Ty::KwExport
                | Ty::KwImport
                | Ty::KwFiles => {
                    let mut stack = self.take_toplevel_stmt();
                    match token.ty {
                        Ty::KwConst => {
                            let stmt = ConstStmt::parse(&mut stack);
                            match stmt {
                                Ok(const_stmt) => {
                                    println!("const_stmt {:#?}", const_stmt);
                                    self.parsed.constants.push(const_stmt);
                                },
                                Err(err) => return Some(err)
                            }
                        }
                        Ty::KwEnum => {
                            let stmt = EnumStmt::parse(&mut stack);
                            match stmt {
                                Ok(enum_stmt) => {
                                    println!("enum_stmt {:#?}", enum_stmt);
                                    self.parsed.enums.push(enum_stmt);
                                },
                                Err(err) => return Some(err)
                            }
                        }
                        Ty::KwFn => {
                            todo!()
                        }
                        Ty::KwImpl => {
                            todo!()
                        }
                        Ty::KwLet => {
                            let stmt = LetStmt::parse(&mut stack);
                            match stmt {
                                Ok(let_stmt) => {
                                    println!("let_stmt {:#?}", let_stmt);
                                    self.parsed.lets.push(let_stmt);
                                },
                                Err(err) => return Some(err)
                            }
                        }
                        Ty::KwStruct => {
                            let stmt = StructStmt::parse(&mut stack);
                            match stmt {
                                Ok(struct_stmt) => {
                                    println!("struct_stmt {:#?}", struct_stmt);
                                    self.parsed.structs.push(struct_stmt);
                                },
                                Err(err) => return Some(err)
                            }
                        }
                        Ty::KwTest => {
                            todo!()
                        }
                        Ty::KwTestCase => {
                            todo!()
                        }
                        Ty::KwTrait => {
                            todo!()
                        }
                        Ty::KwType => {
                            todo!()
                        }
                        //
                        // Package keywords
                        //
                        Ty::KwPackage => {
                            todo!()
                        }
                        Ty::KwDescription => {
                            todo!()
                        }
                        Ty::KwVersion => {
                            todo!()
                        }
                        Ty::KwDependencies => {
                            todo!()
                        }
                        Ty::KwExport => {
                            todo!()
                        }
                        Ty::KwImport => {
                            todo!()
                        }
                        Ty::KwFiles => {
                            todo!()
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

fn take_all_include_pattern(
    tokens: &mut Vec<Token>,
    pattern: Vec<Ty>,
) -> Result<Vec<Token>, Error> {
    if pattern.len() == 0 {
        return Err(err("expected pattern length"));
    }

    let mut pattern_iter = pattern.clone().into_iter();
    let pattern_ty = pattern_iter.next().unwrap();
    let mut taken: Vec<Token> = vec![];
    // let mut buf: Vec<Token> = vec![];
    // buf.push(tok);

    while let Some(tok) = tokens.pop() {
        if &tok.ty == &pattern_ty {
            taken.push(tok);

            while let Some(pattern_next_ty) = pattern_iter.next() {
                if tokens.len() > 0 {
                    let next = tokens.pop().unwrap();
                    if &next.ty == &pattern_next_ty {
                        taken.push(next);
                    } else {
                        // incomplete: reset pattern
                        pattern_iter = pattern.clone().into_iter();
                        taken.push(next);
                        break;
                    }
                }
            }

            if pattern_iter.len() == 0 {
                break;
            }
        } else {
            taken.push(tok);
        }
    }
    taken.reverse();
    Ok(taken)
}

fn remove_newline_and_indent(tokens: &mut Vec<Token>) -> Result<Vec<Token>, Error> {
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
        _ => {
            return Err(err(
                "Expected operator (assign, boolean, arithmetic, logical, or bitwise",
            ))
        }
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
                    return Err(err("type statement inner generic expected a type identity"));
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
                    return Err(err(
                        "unable to parse type statement inner generic: expected balanced brackets",
                    ));
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

        return Err(err("expected to find VarTypeStmt"));
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
            return Err(err("type statement has no tokens"));
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
                        return Err(err("type statement expected a type identity"));
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
                return Err(err(
                    "unable to parse type statement inner generic: expected balanced brackets",
                ));
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
            return Err(err("type statement has no tokens"));
        }
        let mut enum_stmt = Self::default();

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::IdType(id) => {
                    enum_stmt.enum_name = id;
                }
                _ => {
                    return Err(err("enum statement expected a type identity"));
                }
            }
        }

        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::NL => {}
                _ => {
                    return Err(err("enum statement expected a newline after identity"));
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
                        return Err(err("enum statement expected enum variant"));
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
        let mut cleaned: Vec<Token> = remove_newline_and_indent(&mut stack)?;
        println!("EnumVariantStmt {:?} (cleaned)", cleaned);
        let mut enum_variant = EnumVariantStmt::default();

        if let Some(tok) = cleaned.pop() {
            match tok.ty {
                Ty::IdType(name) => {
                    enum_variant.variant_name = name;
                }
                _ => {
                    return Err(err("enum variant statement expected indent"));
                }
            }
        } else {
            return Err(err("enum variant statement expected indent"));
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
                                return Err(err("enum variant statement expected closing brace"));
                            }
                        },
                        _ => {
                            return Err(err("enum variant statement expected closing brace"));
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
                            return Err(err("enum variant statement expected variable or type or primitive identity"));
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
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
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
        let mut cleaned: Vec<Token> = remove_newline_and_indent(&mut stack)?;
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
                    return Err(err("expected IdVar"));
                }
            }
        } else {
            return Err(err("expected IdVar"));
        }
        // Colon
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctColon => {}
                _ => {
                    return Err(err("struct field expected colon after variable identity"));
                }
            }
        } else {
            return Err(err("struct field expected colon after variable identity"));
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
                    return Err(err("expected IdVar"));
                }
            }
        } else {
            return Err(err("expected IdVar"));
        }
        // Colon
        if let Some(tok) = stack.pop() {
            match tok.ty {
                Ty::PunctColon => {}
                _ => {
                    return Err(err("struct field expected colon after variable identity"));
                }
            }
        } else {
            return Err(err("struct field expected colon after variable identity"));
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
    fn parse(mut stack: &mut Vec<Token>) -> Result<Self, Error> {
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
    fn parse(stack: &mut Vec<Token>) -> Result<Self, Error> {
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
        let mut cleaned = remove_newline_and_indent(&mut stack)?;
        cleaned.reverse();
        // println!("ExprStmt cleaned {:?}", cleaned);
        let mut expr_stmt = ExprStmt::default();
        match ExprParser.parse(cleaned.into_iter()) {
            Ok(expr) => expr_stmt.expr = expr,
            Err(e) => return Err(err(&e.to_string()[..])),
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
struct PackageStmt {
    package_id: String,
    description: String,
    version: String,
    files: Vec<PackageFileStmt>,
    dependencies: Vec<PackageDependencyStmt>,
    import: Vec<PackageImportStmt>,
    export: Vec<PackageExportStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageDependencyStmt {
    package_id: Option<String>,
    path: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageFileStmt {
    path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageImportStmt {
    package_id: String,
    symbols: Vec<PackageSymbolStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageExportStmt {
    package_id: Option<String>,
    symbols: Vec<PackageSymbolStmt>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PackageSymbolStmt {
    id: String,
    as_: Option<String>,
}
