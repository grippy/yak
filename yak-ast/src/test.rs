#[cfg(test)]
use crate::pratt::{NoError, PrattError, PrattParser};
#[cfg(test)]
use crate::{
    expr::ExprParser, AssignOp, AssignStmt, Ast, ConstStmt, Expr, ExprStmt, Op, StructFieldStmt,
    StructFieldValueStmt, StructStmt, StructValueStmt, TypeStmt, Value, ValueStmt, VarTypeStmt,
};
#[cfg(test)]
use yak_lexer::token::TokenType as Ty;
#[cfg(test)]
use yak_lexer::Lexer;
#[cfg(test)]
use yak_lexer::Token;

#[cfg(test)]
fn pratt_parser(src: &str) -> Result<Expr, PrattError<Token, NoError>> {
    let mut lexer = Lexer::from_source(src);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);

    let tokens: Vec<Token> = lexer
        .tokens
        .into_iter()
        // .map(|tok| tok.ty)
        .filter(|tok| match tok.ty {
            Ty::Indent(_) | Ty::Sp | Ty::NL => false,
            _ => true,
        })
        .collect();

    println!("filtered: {:#?}", tokens);
    let expr = ExprParser.parse(tokens.into_iter());
    println!("parsed: {:#?}", &expr);
    expr
}

#[test]
fn test_expr_arithmetic() {
    let src = "(1 + 2 / 3 * -(2 - 4) + (x % 3) ** 2 / 5 // 4.0)";
    let _ = pratt_parser(src);
}

#[test]
fn test_expr_boolean() {
    let src = "(x != 1 && !true) || (x < 1 || y <= z) && (z > 10 || z >= 10)";
    let _ = pratt_parser(src);
}

#[test]
fn test_expr_bitwise() {
    let src = "(1 << 8) & (10 >> 2) | 1 ^ 10";
    let _ = pratt_parser(src);
}

#[test]
fn test_expr_id_type() {
    let src = "
MyStruct[X] {
  a: (123 + 2)
  b: !true
}
";
    let _ = pratt_parser(src);
}

#[test]
fn test_var_complex_generic() {
    let src = "const z: X[Y[Z[A B]]] = \"hello\"";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_string() {
    let src = "const x = \"123\"";
    let mut ast = Ast::from_source(src);
    ast.parse();

    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "x".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::String("123".into()),
                }),
            },
        },
    };
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_basic_int() {
    let src = "const x = 123";
    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "x".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Int(123),
                }),
            },
        },
    };
    let mut ast = Ast::from_source(src);
    ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_basic_int_arithmetic_expr() {
    let src = "const expr_int = 1 - 2";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_uint() {
    let src = format!("const x = {}", usize::MAX);
    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "x".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::UInt(usize::MAX),
                }),
            },
        },
    };
    let mut ast = Ast::from_source(src.as_str());
    ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_basic_float() {
    let src = "const x = 123.99";
    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "x".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Float(123.99),
                }),
            },
        },
    };
    let mut ast = Ast::from_source(src);
    ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_basic_unary_boolean() {
    let src = "const unary_bool = !true";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_boolean() {
    let src = "const basic_bool = true";
    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "basic_bool".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Bool(true),
                }),
            },
        },
    };
    let mut ast = Ast::from_source(src);
    ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_basic_unary_func() {
    let src = "const unary_func = +:func1 {}";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_func() {
    let src = "const basic_func = :func1 { a: \"\" }";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_unary_var() {
    let src = "const unary_var = -var1";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_var() {
    let src = "const basic_var = var1";
    let mut _ast = Ast::from_source(src);
    // ast.parse();
}

#[test]
fn test_var_basic_type_struct_expr() {
    let src = "
const struct_var =
  MyStruct {
    a: 123
  }";

    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "struct_var".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Struct(StructValueStmt {
                        struct_type: TypeStmt {
                            type_name: "MyStruct".into(),
                            generics: None,
                        },
                        fields: vec![StructFieldValueStmt {
                            field_name: "a".into(),
                            field_value: ExprStmt {
                                expr: Expr::Value(ValueStmt {
                                    value: Value::Int(123),
                                }),
                            },
                        }],
                    }),
                }),
            },
        },
    };

    let mut ast = Ast::from_source(src);
    ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_complex_type_struct_expr() {
    let src = format!(
        "
const struct_var =
  MyStruct
    field_bool_false: false
    field_bool_true: true
    field_float: 1.9
    field_float_max: {}.0
    field_int: 1
    field_int_max: {}
    field_uint: {}
",
        f64::MAX,
        isize::MAX,
        usize::MAX - isize::MAX as usize
    );

    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "struct_var".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Struct(StructValueStmt {
                        struct_type: TypeStmt {
                            type_name: "MyStruct".into(),
                            generics: None,
                        },
                        fields: vec![
                            StructFieldValueStmt {
                                field_name: "field_bool_false".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Bool(false),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field_bool_true".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Bool(true),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field_float".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Float(1.9),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field_float_max".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Float(1.7976931348623157e308),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field_int".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Int(1),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field_int_max".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Int(9223372036854775807),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field_uint".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::UInt(9223372036854775808),
                                    }),
                                },
                            },
                        ],
                    }),
                }),
            },
        },
    };

    let mut ast = Ast::from_source(src.as_str());
    ast.parse();

    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_struct_enum_expr() {
    let src = "
const struct_var =
  MyStruct
    field_enum: MyEnum::X
";

    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "struct_var".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Struct(StructValueStmt {
                        struct_type: TypeStmt {
                            type_name: "MyStruct".into(),
                            generics: None,
                        },
                        fields: vec![StructFieldValueStmt {
                            field_name: "field_enum".into(),
                            field_value: ExprStmt {
                                expr: Expr::Value(ValueStmt {
                                    value: Value::Bool(false),
                                }),
                            },
                        }],
                    }),
                }),
            },
        },
    };

    let mut ast = Ast::from_source(src);
    ast.parse();

    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.constants.get(0), Some(expected));
}

#[test]
fn test_var_basic_type_list_expr() {}

#[test]
fn test_var_basic_type_set_expr() {}

#[test]
fn test_var_basic_type_fn_expr() {}

#[test]
fn test_var_basic_type_option_expr() {}

#[test]
fn test_var_basic_type_byte_expr() {}

#[test]
fn test_enum() {
    let src = "
enum MyEnum
  V1 { str X[Y] }
  V2 {
    X[Y]
  }
  V3 { f1: X[Y[Z]] f2: int }
  V4 {
    f3: str
    f4: int32
  }
  V5
";
    let mut ast = Ast::from_source(src);
    ast.parse();
}

#[test]
fn test_struct_type_generics() {
    let src = "
struct MyStruct[X]
  f1: Something[X]
";

    let expected = &StructStmt {
        struct_type: TypeStmt {
            type_name: "MyStruct".into(),
            generics: Some(Box::new(vec![TypeStmt {
                type_name: "X".into(),
                generics: None,
            }])),
        },
        fields: vec![StructFieldStmt {
            field_name: "f1".into(),
            field_type: TypeStmt {
                type_name: "Something".into(),
                generics: Some(Box::new(vec![TypeStmt {
                    type_name: "X".into(),
                    generics: None,
                }])),
            },
        }],
    };
    let mut ast = Ast::from_source(src);
    ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.structs.get(0), Some(expected));
}
