#[cfg(test)]
use crate::expr::pratt::{NoError, PrattError, PrattParser};
#[cfg(test)]
use crate::{
    expr::expr::ExprParser, ArithOp, AssignOp, AssignStmt, Ast, BinaryExprStmt, Block, BlockStmt,
    ConstStmt, Expr, ExprStmt, FuncArgValueStmt, FuncBodyStmt, FuncInputArgTypeStmt,
    FuncInputTypeStmt, FuncOutputTypeStmt, FuncStmt, FuncTypeStmt, FuncValueStmt, Op,
    PackageDependencyStmt, PackageExportStmt, PackageFileStmt, PackageImportStmt, PackageStmt,
    PackageSymbol, PackageSymbolStmt, ReturnStmt, StructFieldStmt, StructFieldValueStmt,
    StructStmt, StructValueStmt, TypeStmt, Value, ValueStmt, VarTypeStmt,
};
#[cfg(test)]
use yak_lexer::token::TokenType as Ty;
#[cfg(test)]
use yak_lexer::Lexer;
#[cfg(test)]
use yak_lexer::Token;

#[test]
fn test_package() {
    let src = "
package     \"my.pkg\"
description \"This is my description\"
version     \"1.0.0\"
dependencies {
    local.pkg1 \"http://github.com/yak-pkg/my.pkg\"
    pkg1       \"http://github.com/yak-pkg/pkg1@v1\"
    some.pkg2  \"http://github.com/yak-pkg/some-pkg2@v1\"
}
files {
  \"./file1.yak\"
  \"./file2.yak\"
  \"./file3.yak\"
}
import {
  some.pkg1 {
    :func1
    :func1 as :func_other
    const_var1
    const_var1 as const_var2
    Type1
    Type2 as TypeOther
    ^Trait1
    ^Trait2 as ^TraitOther
  }
}
export {
  :func1
  const_var1
  Struct1
  Enum1
  ^Trait1
}
";

    let expected = PackageStmt {
        package_id: "\"my.pkg\"".into(),
        description: "\"This is my description\"".into(),
        version: "\"1.0.0\"".into(),
        files: [
            PackageFileStmt {
                path: "\"./file1.yak\"".into(),
            },
            PackageFileStmt {
                path: "\"./file2.yak\"".into(),
            },
            PackageFileStmt {
                path: "\"./file3.yak\"".into(),
            },
        ]
        .to_vec(),
        dependencies: [
            PackageDependencyStmt {
                package_id: "local.pkg1".into(),
                path: "\"http://github.com/yak-pkg/my.pkg\"".into(),
            },
            PackageDependencyStmt {
                package_id: "pkg1".into(),
                path: "\"http://github.com/yak-pkg/pkg1@v1\"".into(),
            },
            PackageDependencyStmt {
                package_id: "some.pkg2".into(),
                path: "\"http://github.com/yak-pkg/some-pkg2@v1\"".into(),
            },
        ]
        .to_vec(),
        imports: [PackageImportStmt {
            package_id: "some.pkg1".into(),
            as_package_id: None,
            symbols: [
                PackageSymbolStmt {
                    symbol: PackageSymbol::Func(":func1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Func(":func1".into()),
                    as_symbol: Some(PackageSymbol::Func(":func_other".into())),
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Var("const_var1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Var("const_var1".into()),
                    as_symbol: Some(PackageSymbol::Var("const_var2".into())),
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Type("Type1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Type("Type2".into()),
                    as_symbol: Some(PackageSymbol::Type("TypeOther".into())),
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Trait("^Trait2".into()),
                    as_symbol: Some(PackageSymbol::Trait("^TraitOther".into())),
                },
            ]
            .to_vec(),
        }]
        .to_vec(),
        exports: PackageExportStmt {
            symbols: [
                PackageSymbolStmt {
                    symbol: PackageSymbol::Func(":func1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Var("const_var1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Type("Struct1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Type("Enum1".into()),
                    as_symbol: None,
                },
                PackageSymbolStmt {
                    symbol: PackageSymbol::Trait("^Trait1".into()),
                    as_symbol: None,
                },
            ]
            .to_vec(),
        },
    };

    let mut ast = Ast::from_source(src);
    let _ = ast.parse_package();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.package, expected);
}

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
    let _ = ast.parse();

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
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
fn test_var_func_arg_expr() {
    let src = "
const result = :func1 {
  a: a.b + b + c
  b: (1 + 2) / 3
}";

    let expected = &ConstStmt {
        assign: AssignStmt {
            var_type: VarTypeStmt {
                var_name: "result".into(),
                var_type: None,
            },
            op: Op::Assign(AssignOp::Eq),
            expr: ExprStmt {
                expr: Expr::Value(ValueStmt {
                    value: Value::Func(FuncValueStmt {
                        func_name: ":func1".into(),
                        args: [
                            FuncArgValueStmt {
                                arg_name: "a".into(),
                                arg_value: ExprStmt {
                                    expr: Expr::Binary(BinaryExprStmt {
                                        lhs: Box::new(Expr::Binary(BinaryExprStmt {
                                            lhs: Box::new(Expr::Value(ValueStmt {
                                                value: Value::Package("a.b".into()),
                                            })),
                                            op: Op::Arith(ArithOp::Add),
                                            rhs: Box::new(Expr::Value(ValueStmt {
                                                value: Value::Var("b".into()),
                                            })),
                                        })),
                                        op: Op::Arith(ArithOp::Add),
                                        rhs: Box::new(Expr::Value(ValueStmt {
                                            value: Value::Var("c".into()),
                                        })),
                                    }),
                                },
                            },
                            FuncArgValueStmt {
                                arg_name: "b".into(),
                                arg_value: ExprStmt {
                                    expr: Expr::Binary(BinaryExprStmt {
                                        lhs: Box::new(Expr::Binary(BinaryExprStmt {
                                            lhs: Box::new(Expr::Value(ValueStmt {
                                                value: Value::Int(1),
                                            })),
                                            op: Op::Arith(ArithOp::Add),
                                            rhs: Box::new(Expr::Value(ValueStmt {
                                                value: Value::Int(2),
                                            })),
                                        })),
                                        op: Op::Arith(ArithOp::Div),
                                        rhs: Box::new(Expr::Value(ValueStmt {
                                            value: Value::Int(3),
                                        })),
                                    }),
                                },
                            },
                        ]
                        .to_vec(),
                    }),
                }),
            },
        },
    };

    let mut ast = Ast::from_source(src);
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();

    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
}

#[test]
fn test_var_struct_field_expr() {
    let src = "
const struct_var =
  MyStruct
    field1: 1 + 2 / var1
    field2: var1
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
                        fields: [
                            StructFieldValueStmt {
                                field_name: "field1".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Binary(BinaryExprStmt {
                                        lhs: Box::new(Expr::Value(ValueStmt {
                                            value: Value::Int(1),
                                        })),
                                        op: Op::Arith(ArithOp::Add),
                                        rhs: Box::new(Expr::Binary(BinaryExprStmt {
                                            lhs: Box::new(Expr::Value(ValueStmt {
                                                value: Value::Int(2),
                                            })),
                                            op: Op::Arith(ArithOp::Div),
                                            rhs: Box::new(Expr::Value(ValueStmt {
                                                value: Value::Var("var1".into()),
                                            })),
                                        })),
                                    }),
                                },
                            },
                            StructFieldValueStmt {
                                field_name: "field2".into(),
                                field_value: ExprStmt {
                                    expr: Expr::Value(ValueStmt {
                                        value: Value::Var("var1".into()),
                                    }),
                                },
                            },
                        ]
                        .to_vec(),
                    }),
                }),
            },
        },
    };

    let mut ast = Ast::from_source(src);
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.consts.get(0), Some(expected));
}

#[test]
fn test_var_struct_enum_expr() {
    let src = "
const struct_var =
  MyStruct
    field_enum_simple: MyEnum::X
    field_enum_struct: MyEnum::MyStruct { a: \"a\" b: \"b\" }
    field_enum_tuple: MyEnum::MyTuple { \"a\" \"b\" }
";
    // TODO: come back to this once Expr parsing is fully complete
    // let _expected = &ConstStmt {
    //     assign: AssignStmt {
    //         var_type: VarTypeStmt {
    //             var_name: "struct_var".into(),
    //             var_type: None,
    //         },
    //         op: Op::Assign(AssignOp::Eq),
    //         expr: ExprStmt {
    //             expr: Expr::Value(ValueStmt {
    //                 value: Value::Struct(StructValueStmt {
    //                     struct_type: TypeStmt {
    //                         type_name: "MyStruct".into(),
    //                         generics: None,
    //                     },
    //                     fields: vec![StructFieldValueStmt {
    //                         field_name: "field_enum".into(),
    //                         field_value: ExprStmt {
    //                             expr: Expr::Value(ValueStmt {
    //                                 value: Value::Bool(false),
    //                             }),
    //                         },
    //                     }],
    //                 }),
    //             }),
    //         },
    //     },
    // };

    let mut ast = Ast::from_source(src);
    let _ = ast.parse();
    // assert_eq!(ast.parsed.errors.len(), 0);
    // assert_eq!(ast.parsed.consts.get(0), Some(expected));
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
    let _ = ast.parse();
}

#[test]
fn test_fn_simple() {
    let src = "
fn :fn1 {} String =>
  return \"Hello\"
";
    let expected = &FuncStmt {
        func_name: ":fn1".into(),
        func_type: FuncTypeStmt {
            is_self: false,
            input_type: None,
            output_type: Some(FuncOutputTypeStmt {
                output_type: TypeStmt {
                    type_name: "String".into(),
                    generics: None,
                },
            }),
        },
        func_body: FuncBodyStmt {
            blocks: [BlockStmt {
                indent: 2,
                blocks: [Block::Return(ReturnStmt {
                    expr: ExprStmt {
                        expr: Expr::Value(ValueStmt {
                            value: Value::String("Hello".into()),
                        }),
                    },
                    return_type: None,
                })]
                .to_vec(),
                return_type: None,
            }]
            .to_vec(),
        },
    };

    let mut ast = Ast::from_source(src);
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.funcs.get(0), Some(expected));
}

#[test]
fn test_fn_type_signature() {
    let src = "
fn :fn1 self {
    a: String
    b: int
  } String =>
  return \"Hello\"
";

    let expected = &FuncStmt {
        func_name: ":fn1".into(),
        func_type: FuncTypeStmt {
            is_self: true,
            input_type: Some(FuncInputTypeStmt {
                args: [
                    FuncInputArgTypeStmt {
                        arg_name: "a".into(),
                        arg_type: TypeStmt {
                            type_name: "String".into(),
                            generics: None,
                        },
                    },
                    FuncInputArgTypeStmt {
                        arg_name: "b".into(),
                        arg_type: TypeStmt {
                            type_name: "int32".into(),
                            generics: None,
                        },
                    },
                ]
                .to_vec(),
            }),
            output_type: Some(FuncOutputTypeStmt {
                output_type: TypeStmt {
                    type_name: "String".into(),
                    generics: None,
                },
            }),
        },
        func_body: FuncBodyStmt {
            blocks: [BlockStmt {
                indent: 2,
                blocks: [Block::Return(ReturnStmt {
                    expr: ExprStmt {
                        expr: Expr::Value(ValueStmt {
                            value: Value::String("Hello".into()),
                        }),
                    },
                    return_type: None,
                })]
                .to_vec(),
                return_type: None,
            }]
            .to_vec(),
        },
    };

    let mut ast = Ast::from_source(src);
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.funcs.get(0), Some(expected));
}

#[test]
fn test_fn_blocks() {
    let src = "
fn :fn1 {} String =>
  let a = \"a\"
  let b = \"b\"
  let c = \"b\"
  let d = \"\"
  if a == b then
    a = \"b\"
    d = \"one\"
  elif a == c then
    a = \"c\"
    d = \"two\"
  else
    a = \"default\"
    d = \"default\"

  :println { arg1: \"hello\" arg2: \"world\" }
  return d
";

    let mut ast = Ast::from_source(src);
    let _ = ast.parse();
    // assert_eq!(ast.parsed.errors.len(), 0);
    // assert_eq!(ast.parsed.funcs.get(0), Some(expected));
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
    let _ = ast.parse();
    assert_eq!(ast.parsed.errors.len(), 0);
    assert_eq!(ast.parsed.structs.get(0), Some(expected));
}
