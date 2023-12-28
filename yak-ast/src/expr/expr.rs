use crate::expr::pratt::{Affix, Associativity, NoError, PrattParser, Precedence, Result};
use crate::{
    ArithOp, Balance, BinaryExprStmt, BitwiseOp, BooleanOp, EnumValueStmt, Expr, FuncValueStmt,
    LogicalOp, Op, Parse, StructValueStmt, TupleValueStmt, UnaryExprStmt, UnaryOp, Value,
    ValueStmt,
};
use yak_lexer::token::TokenType as Ty;
use yak_lexer::Token;

// 0  non-binding operators
// 10 assignment operators = *= /= //= += -= ...
// >> 20 ternary operator ? : (can't use ? for returning errors)
// 30 logical operator ||
// 40 logical operator &&
// 50 boolean operators == != > >= < <=
// 60 bitwise operator |
// 70 bitwise operator ^
// 80 bitwise operator &
// 90 bitwise operator << >>
// 100 infix operator + -
// 110 infix operator * / % //
// 120 infix operator **
// 130 cast type as
// 140	unary operators
//      + - !
// 150 return errors ?
// 160 var1.field :func [ (
// 170 paths

pub(crate) struct ExprParser;

impl<I> PrattParser<I> for ExprParser
where
    I: Iterator<Item = Token>,
{
    type Error = NoError;
    type Input = Token;
    type Output = Expr;

    // Query information about an operator (Affix, Precedence, Associativity)
    fn query(&mut self, tok: &Self::Input) -> Result<Affix> {
        let affix = match tok.ty {
            // Logical
            // || &&
            Ty::OpLogicalOr => Affix::Infix(Precedence(30), Associativity::Left),
            Ty::OpLogicalAnd => Affix::Infix(Precedence(40), Associativity::Left),

            // Boolean
            // == != < <= > >=
            Ty::OpEqEq => Affix::Infix(Precedence(50), Associativity::Left),
            Ty::OpNotEq => Affix::Infix(Precedence(50), Associativity::Left),
            Ty::OpLt => Affix::Infix(Precedence(50), Associativity::Left),
            Ty::OpLte => Affix::Infix(Precedence(50), Associativity::Left),
            Ty::OpGt => Affix::Infix(Precedence(50), Associativity::Left),
            Ty::OpGte => Affix::Infix(Precedence(50), Associativity::Left),

            // Bitwise
            // | ^ & << >>
            Ty::OpBitwiseOr => Affix::Infix(Precedence(60), Associativity::Right),
            Ty::OpBitwiseXOr => Affix::Infix(Precedence(70), Associativity::Right),
            Ty::OpBitwiseAnd => Affix::Infix(Precedence(80), Associativity::Right),
            Ty::OpBitwiseShiftL => Affix::Infix(Precedence(90), Associativity::Right),
            Ty::OpBitwiseShiftR => Affix::Infix(Precedence(90), Associativity::Right),

            // Arithmetic
            // + - * / // **
            Ty::OpAdd => Affix::Infix(Precedence(100), Associativity::Left),
            Ty::OpSub => Affix::Infix(Precedence(100), Associativity::Left),
            Ty::OpMul => Affix::Infix(Precedence(110), Associativity::Left),
            Ty::OpDiv => Affix::Infix(Precedence(110), Associativity::Left),
            Ty::OpFloorDiv => Affix::Infix(Precedence(110), Associativity::Left),
            Ty::OpMod => Affix::Infix(Precedence(110), Associativity::Left),
            Ty::OpPow => Affix::Infix(Precedence(120), Associativity::Left),

            // KwAs goes here...
            // Ty::KwAs => Affix::Infix(Precedence(130), Associativity::Left),
            // - + !
            Ty::OpUnaryMinus => Affix::Prefix(Precedence(140)),
            Ty::OpUnaryPlus => Affix::Prefix(Precedence(140)),
            Ty::OpUnaryNot | Ty::PunctExclamation => Affix::Prefix(Precedence(140)),

            // Literals
            Ty::PunctParenL => Affix::Nilfix,
            Ty::LitString(_) => Affix::Nilfix,
            Ty::LitNumber(_) => Affix::Nilfix,
            Ty::LitBoolean(_) => Affix::Nilfix,
            Ty::IdFunc(_) => Affix::Nilfix,
            Ty::IdPackage(_) => Affix::Nilfix,
            Ty::IdTrait(_) => Affix::Nilfix,
            Ty::IdType(_) => Affix::Nilfix,
            Ty::IdVar(_) => Affix::Nilfix,
            _ => {
                println!("ExprParser.query: unreachable token #{:?}", &tok);
                unreachable!()
            }
        };
        Ok(affix)
    }

    // Construct a primary expression, e.g. a number
    fn primary(&mut self, tok: Self::Input, inputs: &mut core::iter::Peekable<I>) -> Result<Expr> {
        let result = match tok.ty {
            Ty::LitBoolean(lit) => Ok(Expr::Value(ValueStmt {
                value: Value::Bool(
                    lit.parse()
                        .expect("Unable to convert boolean literal to bool"),
                ),
            })),
            Ty::LitString(lit) => {
                let mut clean = lit.as_str();
                if clean.starts_with("\"") {
                    clean = clean.strip_prefix("\"").unwrap();
                }
                if clean.ends_with("\"") {
                    clean = clean.strip_suffix("\"").unwrap();
                }
                Ok(Expr::Value(ValueStmt {
                    // remove start/end quotes
                    value: Value::String(clean.to_string()),
                }))
            }
            Ty::LitNumber(lit) => {
                if lit.contains("-") {
                    let int: isize = lit.parse().expect(
                        format!(
                            "Unable to convert number literal \"{}\" into isize",
                            lit.as_str()
                        )
                        .as_str(),
                    );
                    Ok(Expr::Value(ValueStmt {
                        value: Value::Int(int),
                    }))
                } else if lit.contains(".") {
                    let float: f64 = lit.parse().expect(
                        format!(
                            "Unable to convert number literal \"{}\" into f64",
                            lit.as_str()
                        )
                        .as_str(),
                    );
                    Ok(Expr::Value(ValueStmt {
                        value: Value::Float(float),
                    }))
                } else {
                    // usize or isize?
                    let uint: usize = lit.parse().expect(
                        format!(
                            "Unable to convert number literal \"{}\" into usize",
                            lit.as_str()
                        )
                        .as_str(),
                    );
                    // we default integers to isize
                    // if the value isn't greater than the max
                    if uint > isize::MAX as usize {
                        Ok(Expr::Value(ValueStmt {
                            value: Value::UInt(uint),
                        }))
                    } else {
                        Ok(Expr::Value(ValueStmt {
                            value: Value::Int(uint as isize),
                        }))
                    }
                }
            }
            Ty::IdVar(v) => Ok(Expr::Value(ValueStmt {
                value: Value::Var(v),
            })),
            Ty::IdPackage(v) => Ok(Expr::Value(ValueStmt {
                value: Value::Package(v),
            })),
            Ty::IdFunc(_) => {
                // this should take the entire func call
                // and is similar to a StructValueStmt
                let mut balance = Balance::default();
                balance.brace_l += 1;

                let mut group: Vec<Token> = vec![];
                while let Some(next_tok) = inputs.next() {
                    match next_tok.ty {
                        Ty::PunctBraceL => {
                            balance.brace_l += 1;
                        }
                        Ty::PunctBraceR => {
                            balance.brace_r += 1;
                        }
                        _ => {}
                    }
                    if balance.balanced_braces() {
                        break;
                    } else {
                        group.push(next_tok);
                    }
                }
                // make this a stack
                group.reverse();
                // add the IdType back
                group.push(tok);

                let func_val_stmt =
                    FuncValueStmt::parse(&mut group).expect("unable to parse FuncValueStmt");
                Ok(Expr::Value(ValueStmt {
                    value: Value::Func(func_val_stmt),
                }))
            }
            Ty::IdType(_) => {
                // IdType can be one of:
                // - StructValueStmt
                //      - MyStruct { field: value ... }
                // - EnumValueStmt
                //      - Struct
                //          - MyEnum::IdType
                //          - MyEnum::IdType { field: value ... }
                //      - Tuple
                //          - MyEnum::IdType { value1 value2 ... }
                // - TupleValueStmt
                //      - MyTuple { value1 value2 ... }

                // Next, determine what type we have here...
                // if the next token is a double-colon then we
                // have EnumValueStmt
                // let mut is_enum = false;
                // if let Some(tok) = inputs.peek() {
                //     match tok.ty {
                //         Ty::PunctDoubleColon => {
                //             is_enum = true;
                //         }
                //         _ => {}
                //     }
                // }

                // if is_enum {
                //     println!("is_enum=true");
                //     todo!("ExprParser is missing enum support");
                // }

                // This should take the entire `Type {}` value call
                // and is either a StructValueStmt or TupleValueStmt
                let mut balance = Balance::default();
                balance.brace_l += 1;

                let mut group: Vec<Token> = vec![];
                while let Some(next_tok) = inputs.next() {
                    match next_tok.ty {
                        Ty::PunctBraceL => {
                            balance.brace_l += 1;
                        }
                        Ty::PunctBraceR => {
                            balance.brace_r += 1;
                        }
                        _ => {}
                    }
                    if balance.balanced_braces() {
                        break;
                    } else {
                        group.push(next_tok);
                    }
                }
                // make this a stack
                group.reverse();
                // add the IdType back
                group.push(tok);

                let expr = if group.contains(&Token { ty: Ty::PunctColon }) {
                    // StructValueStmt
                    let struct_value = StructValueStmt::parse(&mut group)
                        .expect("unable to parse StructValueStmt");
                    Ok(Expr::Value(ValueStmt {
                        value: Value::Struct(struct_value),
                    }))
                } else if group.contains(&Token {
                    ty: Ty::PunctDoubleColon,
                }) {
                    // EnumValueStmt
                    let enum_value =
                        EnumValueStmt::parse(&mut group).expect("unable to parse EnumValueStmt");
                    Ok(Expr::Value(ValueStmt {
                        value: Value::Enum(enum_value),
                    }))
                } else {
                    // TupleValueStmt
                    let tup_value =
                        TupleValueStmt::parse(&mut group).expect("unable to parse TupleValueStmt");
                    Ok(Expr::Value(ValueStmt {
                        value: Value::Tuple(tup_value),
                    }))
                };

                expr
            }

            Ty::PunctParenL => {
                // we need to collect all tokens until we reach
                // the closing paren. this becomes our next expr parsing
                let mut balance = Balance::default();
                balance.paren_l += 1;

                let mut group: Vec<Token> = vec![];
                while let Some(next_tok) = inputs.next() {
                    match next_tok.ty {
                        Ty::PunctParenL => {
                            balance.paren_l += 1;
                        }
                        Ty::PunctParenR => {
                            balance.paren_r += 1;
                        }
                        _ => {}
                    }
                    if balance.balanced_parens() {
                        break;
                    } else {
                        group.push(next_tok);
                    }
                }
                match self.parse(&mut group.into_iter()) {
                    Ok(expr) => Ok(expr),
                    Err(err) => {
                        println!("Err: #{:}", &err);
                        unreachable!()
                    }
                }
                // unreachable!()
                // Err(PrattError::UserError("Missing closing paren for group"))
            }

            // We don't expect traits inside expressions
            // Ty::IdTrait(v) => {}
            _ => {
                println!("ExprParser.primary: unreachable token #{:?}", &tok);
                unreachable!()
            }
        };
        result
    }

    // Construct a binary infix expression, e.g. 1+1
    // convert Token => Operator and return an Expr
    fn infix(&mut self, lhs: Expr, tok: Self::Input, rhs: Expr) -> Result<Expr> {
        let op = match tok.ty {
            // Logical
            Ty::OpLogicalOr => Op::Logical(LogicalOp::Or),
            Ty::OpLogicalAnd => Op::Logical(LogicalOp::And),
            // Arithmetic
            Ty::OpAdd => Op::Arith(ArithOp::Add),
            Ty::OpSub => Op::Arith(ArithOp::Sub),
            Ty::OpMul => Op::Arith(ArithOp::Mul),
            Ty::OpDiv => Op::Arith(ArithOp::Div),
            Ty::OpFloorDiv => Op::Arith(ArithOp::FloorDiv),
            Ty::OpMod => Op::Arith(ArithOp::Mod),
            Ty::OpPow => Op::Arith(ArithOp::Pow),
            // Boolean
            Ty::OpEqEq => Op::Boolean(BooleanOp::EqEq),
            Ty::OpNotEq => Op::Boolean(BooleanOp::NotEq),
            Ty::OpGt => Op::Boolean(BooleanOp::Gt),
            Ty::OpGte => Op::Boolean(BooleanOp::Gte),
            Ty::OpLt => Op::Boolean(BooleanOp::Lt),
            Ty::OpLte => Op::Boolean(BooleanOp::Lte),
            // Bitwise
            Ty::OpBitwiseAnd => Op::Bitwise(BitwiseOp::And),
            Ty::OpBitwiseOr => Op::Bitwise(BitwiseOp::Or),
            Ty::OpBitwiseXOr => Op::Bitwise(BitwiseOp::XOr),
            Ty::OpBitwiseShiftL => Op::Bitwise(BitwiseOp::ShiftL),
            Ty::OpBitwiseShiftR => Op::Bitwise(BitwiseOp::ShiftR),
            _ => {
                println!("ExprParser.infix: unreachable token #{:?}", &tok);
                unreachable!()
            }
        };
        Ok(Expr::Binary(BinaryExprStmt {
            lhs: Box::new(lhs),
            op: op,
            rhs: Box::new(rhs),
        }))
    }

    // Construct a unary prefix expression, e.g. !1
    fn prefix(&mut self, tok: Self::Input, rhs: Expr) -> Result<Expr> {
        let op = match tok.ty {
            Ty::OpUnaryPlus => UnaryOp::Plus,
            Ty::OpUnaryMinus => UnaryOp::Minus,
            Ty::OpUnaryNot | Ty::PunctExclamation => UnaryOp::Not,
            _ => {
                println!("ExprParser.prefix: unreachable token #{:?}", &tok);
                unreachable!()
            }
        };
        Ok(Expr::Unary(UnaryExprStmt {
            op: op,
            rhs: Box::new(rhs),
        }))
    }

    // Construct a unary postfix expression, e.g. 1?
    fn postfix(&mut self, _lhs: Expr, tok: Self::Input) -> Result<Expr> {
        let _op = match tok.ty {
            // TokenTree::Postfix('?') => UnOpKind::Try,
            _ => {
                println!("ExprParser.postfix: unreachable token #{:?}", &tok);
                unreachable!()
            }
        };
        // Ok(Expr::UnOp(op, Box::new(lhs)))
    }
}
