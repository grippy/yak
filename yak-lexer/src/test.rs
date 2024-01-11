#[cfg(test)]
use crate::{Lexer, Token, TokenType::*};

#[test]
fn special_punctuation() {
    let source = "{} [] ()";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: PunctBraceL },
        Token { ty: PunctBraceR },
        Token { ty: Sp },
        Token { ty: PunctBracketL },
        Token { ty: PunctBracketR },
        Token { ty: Sp },
        Token { ty: PunctParenL },
        Token { ty: PunctParenR },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_colon() {
    let source = ": ::";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: PunctColon },
        Token { ty: Sp },
        Token {
            ty: PunctDoubleColon,
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_pipe() {
    let source = "| |= ||";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpBitwiseOr },
        Token { ty: Sp },
        Token {
            ty: OpAssignBitwiseOr,
        },
        Token { ty: Sp },
        Token { ty: OpLogicalOr },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_carrot() {
    let source = "^ ^= ^Trait1";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpBitwiseXOr },
        Token { ty: Sp },
        Token {
            ty: OpAssignBitwiseXOr,
        },
        Token { ty: Sp },
        Token {
            ty: IdTrait("^Trait1".into()),
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_ampersand() {
    let source = "& && &=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpBitwiseAnd },
        Token { ty: Sp },
        Token { ty: OpLogicalAnd },
        Token { ty: Sp },
        Token {
            ty: OpAssignBitwiseAnd,
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_percent() {
    let source = "% %=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpMod },
        Token { ty: Sp },
        Token { ty: OpAssignMod },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_forward_slash() {
    let source = "/ /= // //=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpDiv },
        Token { ty: Sp },
        Token { ty: OpAssignDiv },
        Token { ty: Sp },
        Token { ty: OpFloorDiv },
        Token { ty: Sp },
        Token {
            ty: OpAssignFloorDiv,
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_minus() {
    let source = "- -=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpSub },
        Token { ty: Sp },
        Token { ty: OpAssignSub },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_plus() {
    let source = "+ +=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpAdd },
        Token { ty: Sp },
        Token { ty: OpAssignAdd },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_angle_left() {
    let source = "< <= << <<=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpLt },
        Token { ty: Sp },
        Token { ty: OpLte },
        Token { ty: Sp },
        Token {
            ty: OpBitwiseShiftL,
        },
        Token { ty: Sp },
        Token {
            ty: OpAssignBitwiseShiftL,
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_angle_right() {
    let source = "> >= >> >>=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpGt },
        Token { ty: Sp },
        Token { ty: OpGte },
        Token { ty: Sp },
        Token {
            ty: OpBitwiseShiftR,
        },
        Token { ty: Sp },
        Token {
            ty: OpAssignBitwiseShiftR,
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_equal() {
    let source = "= == =>";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpAssignEq },
        Token { ty: Sp },
        Token { ty: OpEqEq },
        Token { ty: Sp },
        Token { ty: PunctFatArrow },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_exclamation() {
    let source = "! !=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    let expected = vec![
        Token { ty: Indent(0) },
        Token {
            ty: PunctExclamation,
        },
        Token { ty: Sp },
        Token { ty: OpNotEq },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn special_asterisk() {
    let source = "* *= **=";
    let mut lexer = Lexer::from_source(source);
    lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: OpMul },
        Token { ty: Sp },
        Token { ty: OpAssignMul },
        Token { ty: Sp },
        Token { ty: OpAssignPow },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn let_string() {
    let source = "let x = \"hello the thing = == => ! != <<= << < > >> >= >>= + += - -= / // /= //= * *= ** **= % & &= && | |= || : :: {} [] ()\"";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    //println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token {
            ty: Indent(
                0,
            ),
        },
        Token {
            ty: KwLet,
        },
        Token {
            ty: Sp,
        },
        Token {
            ty: IdVar(
                "x".into(),
            ),
        },
        Token {
            ty: Sp,
        },
        Token {
            ty: OpAssignEq,
        },
        Token {
            ty: Sp,
        },
        Token {
            ty: LitString(
                "\"hello the thing = == => ! != <<= << < > >> >= >>= + += - -= / // /= //= * *= ** **= % & &= && | |= || : :: {} [] ()\"".into(),
            ),
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn literals() {
    let source = "-1 -1.0 -.0009 +1 +1.0 +.0009 1 1.0 .0009 true false \"This my string\"";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token {
            ty: LitNumber("-1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("-1.0".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("-.0009".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("+1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("+1.0".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("+.0009".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber("1.0".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitNumber(".0009".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitBoolean("true".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitBoolean("false".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitString("\"This my string\"".into()),
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn identities() {
    let source =
        " _var1 my.pkg._var1 var2 pkg1 my.pkg2 Type1 my.pkg.Type1 ^Trait1 my.pkg^Trait1 :func1 my.pkg:func1";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(1) },
        Token {
            ty: IdVar("_var1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdVar("my.pkg._var1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdVar("var2".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdVar("pkg1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdPackage("my.pkg2".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdType("Type1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdType("my.pkg.Type1".into()),
        },
        Token { ty: Sp },
        // Token { ty: OpBitwiseXOr },
        Token {
            ty: IdTrait("^Trait1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdPackage("my.pkg".into()),
        },
        Token {
            ty: IdTrait("^Trait1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdFunc(":func1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdPackage("my.pkg".into()),
        },
        Token {
            ty: IdFunc(":func1".into()),
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn comment() {
    let source = "# this my comment\nlet x=1 #comment1";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token {
            ty: Comment("# this my comment".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwLet },
        Token { ty: Sp },
        Token {
            ty: IdVar("x".into()),
        },
        Token { ty: OpAssignEq },
        Token {
            ty: LitNumber("1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: Comment("#comment1".into()),
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn keywords() {
    let source = "as case const else elif enum for if in lazy let match return test testcase trait type while";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: KwAs },
        Token { ty: Sp },
        Token { ty: KwCase },
        Token { ty: Sp },
        Token { ty: KwConst },
        Token { ty: Sp },
        Token { ty: KwElse },
        Token { ty: Sp },
        Token { ty: KwElseIf },
        Token { ty: Sp },
        Token { ty: KwEnum },
        Token { ty: Sp },
        Token { ty: KwFor },
        Token { ty: Sp },
        Token { ty: KwIf },
        Token { ty: Sp },
        Token { ty: KwIn },
        Token { ty: Sp },
        Token { ty: KwLazy },
        Token { ty: Sp },
        Token { ty: KwLet },
        Token { ty: Sp },
        Token { ty: KwMatch },
        Token { ty: Sp },
        Token { ty: KwReturn },
        Token { ty: Sp },
        Token { ty: KwTest },
        Token { ty: Sp },
        Token { ty: KwTestCase },
        Token { ty: Sp },
        Token { ty: KwTrait },
        Token { ty: Sp },
        Token { ty: KwType },
        Token { ty: Sp },
        Token { ty: KwWhile },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn primitives() {
    let source = "bool byte float float32 float64 int int8 int16 int32 int64 uint uint8 uint16 uint32 uint64";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: PrBool },
        Token { ty: Sp },
        Token { ty: PrByte },
        Token { ty: Sp },
        Token { ty: PrFloat32 },
        Token { ty: Sp },
        Token { ty: PrFloat32 },
        Token { ty: Sp },
        Token { ty: PrFloat64 },
        Token { ty: Sp },
        Token { ty: PrInt32 },
        Token { ty: Sp },
        Token { ty: PrInt8 },
        Token { ty: Sp },
        Token { ty: PrInt16 },
        Token { ty: Sp },
        Token { ty: PrInt32 },
        Token { ty: Sp },
        Token { ty: PrInt64 },
        Token { ty: Sp },
        Token { ty: PrUInt32 },
        Token { ty: Sp },
        Token { ty: PrUInt8 },
        Token { ty: Sp },
        Token { ty: PrUInt16 },
        Token { ty: Sp },
        Token { ty: PrUInt32 },
        Token { ty: Sp },
        Token { ty: PrUInt64 },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn strings() {
    let source = "
let x = \"This is my string\"
let y = \"This is my \\\"string\\\"\"
let z = \"1
2
3\"
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwLet },
        Token { ty: Sp },
        Token {
            ty: IdVar("x".into()),
        },
        Token { ty: Sp },
        Token { ty: OpAssignEq },
        Token { ty: Sp },
        Token {
            ty: LitString("\"This is my string\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwLet },
        Token { ty: Sp },
        Token {
            ty: IdVar("y".into()),
        },
        Token { ty: Sp },
        Token { ty: OpAssignEq },
        Token { ty: Sp },
        Token {
            ty: LitString("\"This is my \\\"string\\\"\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwLet },
        Token { ty: Sp },
        Token {
            ty: IdVar("z".into()),
        },
        Token { ty: Sp },
        Token { ty: OpAssignEq },
        Token { ty: Sp },
        Token {
            ty: LitString("\"1\n2\n3\"".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn struct_basic() {
    let source = "
struct MyStruct
  a: int
  b: str
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwStruct },
        Token { ty: Sp },
        Token {
            ty: IdType("MyStruct".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdVar("a".into()),
        },
        Token { ty: PunctColon },
        Token { ty: Sp },
        Token { ty: PrInt32 },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdVar("b".into()),
        },
        Token { ty: PunctColon },
        Token { ty: Sp },
        Token { ty: PrStr },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn enum_basic() {
    let source = "
enum MyEnum
  A { str int }
  B { a: str b: int }
  C
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwEnum },
        Token { ty: Sp },
        Token {
            ty: IdType("MyEnum".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdType("A".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token { ty: PrStr },
        Token { ty: Sp },
        Token { ty: PrInt32 },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdType("B".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token {
            ty: IdVar("a".into()),
        },
        Token { ty: PunctColon },
        Token { ty: Sp },
        Token { ty: PrStr },
        Token { ty: Sp },
        Token {
            ty: IdVar("b".into()),
        },
        Token { ty: PunctColon },
        Token { ty: Sp },
        Token { ty: PrInt32 },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdType("C".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn fn_basic() {
    let source = "
fn :func1 self { name: str } str =>
  return \"hello\"
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwFn },
        Token { ty: Sp },
        Token {
            ty: IdFunc(":func1".into()),
        },
        Token { ty: Sp },
        Token { ty: KwSelf },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token {
            ty: IdVar("name".into()),
        },
        Token { ty: PunctColon },
        Token { ty: Sp },
        Token { ty: PrStr },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: Sp },
        Token { ty: PrStr },
        Token { ty: Sp },
        Token { ty: PunctFatArrow },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token { ty: KwReturn },
        Token { ty: Sp },
        Token {
            ty: LitString("\"hello\"".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn trait_basic() {
    let source = "
trait ^MyTrait
    type X = str
    fn :fn1 {} Self::X
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwTrait },
        Token { ty: Sp },
        Token {
            ty: IdTrait("^MyTrait".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(4) },
        Token { ty: KwType },
        Token { ty: Sp },
        Token {
            ty: IdType("X".into()),
        },
        Token { ty: Sp },
        Token { ty: OpAssignEq },
        Token { ty: Sp },
        Token { ty: PrStr },
        Token { ty: NL },
        Token { ty: Indent(4) },
        Token { ty: KwFn },
        Token { ty: Sp },
        Token {
            ty: IdFunc(":fn1".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: PunctBraceR },
        Token { ty: Sp },
        Token {
            ty: SpecialTypeSelf,
        },
        Token {
            ty: PunctDoubleColon,
        },
        Token {
            ty: IdType("X".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn impl_trait_basic() {
    let source = "
impl Struct1 ^MyTrait
  fn :fn1 {} Self::X =>
    return \"hello\"
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwImpl },
        Token { ty: Sp },
        Token {
            ty: IdType("Struct1".into()),
        },
        Token { ty: Sp },
        // Token { ty: OpBitwiseXOr },
        Token {
            ty: IdTrait("^MyTrait".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token { ty: KwFn },
        Token { ty: Sp },
        Token {
            ty: IdFunc(":fn1".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: PunctBraceR },
        Token { ty: Sp },
        Token {
            ty: SpecialTypeSelf,
        },
        Token {
            ty: PunctDoubleColon,
        },
        Token {
            ty: IdType("X".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctFatArrow },
        Token { ty: NL },
        Token { ty: Indent(4) },
        Token { ty: KwReturn },
        Token { ty: Sp },
        Token {
            ty: LitString("\"hello\"".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn if_basic() {
    let source = "
if x > 0 then
  # gt zero
elif x < 0 then
  # lt zero
else
  # zero
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwIf },
        Token { ty: Sp },
        Token {
            ty: IdVar("x".into()),
        },
        Token { ty: Sp },
        Token { ty: OpGt },
        Token { ty: Sp },
        Token {
            ty: LitNumber("0".into()),
        },
        Token { ty: Sp },
        Token { ty: KwThen },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: Comment("# gt zero".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwElseIf },
        Token { ty: Sp },
        Token {
            ty: IdVar("x".into()),
        },
        Token { ty: Sp },
        Token { ty: OpLt },
        Token { ty: Sp },
        Token {
            ty: LitNumber("0".into()),
        },
        Token { ty: Sp },
        Token { ty: KwThen },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: Comment("# lt zero".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwElse },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: Comment("# zero".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn match_basic() {
    let source = "
match x
  case X { a } =>
    # is X
  else =>
    # default
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwMatch },
        Token { ty: Sp },
        Token {
            ty: IdVar("x".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token { ty: KwCase },
        Token { ty: Sp },
        Token {
            ty: IdType("X".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token {
            ty: IdVar("a".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: Sp },
        Token { ty: PunctFatArrow },
        Token { ty: NL },
        Token { ty: Indent(4) },
        Token {
            ty: Comment("# is X".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token { ty: KwElse },
        Token { ty: Sp },
        Token { ty: PunctFatArrow },
        Token { ty: NL },
        Token { ty: Indent(4) },
        Token {
            ty: Comment("# default".into()),
        },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

#[test]
fn expr_basic() {
    let source = "(1 + 2 / 3 * -(2 - 4) + (x % 3)) / !5";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: Indent(0) },
        Token { ty: PunctParenL },
        Token {
            ty: LitNumber("1".into()),
        },
        Token { ty: Sp },
        Token { ty: OpAdd },
        Token { ty: Sp },
        Token {
            ty: LitNumber("2".into()),
        },
        Token { ty: Sp },
        Token { ty: OpDiv },
        Token { ty: Sp },
        Token {
            ty: LitNumber("3".into()),
        },
        Token { ty: Sp },
        Token { ty: OpMul },
        Token { ty: Sp },
        Token { ty: OpUnaryMinus },
        Token { ty: PunctParenL },
        Token {
            ty: LitNumber("2".into()),
        },
        Token { ty: Sp },
        Token { ty: OpSub },
        Token { ty: Sp },
        Token {
            ty: LitNumber("4".into()),
        },
        Token { ty: PunctParenR },
        Token { ty: Sp },
        Token { ty: OpAdd },
        Token { ty: Sp },
        Token { ty: PunctParenL },
        Token {
            ty: IdVar("x".into()),
        },
        Token { ty: Sp },
        Token { ty: OpMod },
        Token { ty: Sp },
        Token {
            ty: LitNumber("3".into()),
        },
        Token { ty: PunctParenR },
        Token { ty: PunctParenR },
        Token { ty: Sp },
        Token { ty: OpDiv },
        Token { ty: Sp },
        Token {
            ty: PunctExclamation,
        },
        Token {
            ty: LitNumber("5".into()),
        },
    ];
    assert_eq!(lexer.tokens, expected);
}

// #[test]
// fn for_basic() {
//     let source = "";
//     let mut lexer = Lexer::from_source(source);
//     let _ = lexer.parse();
//     // println!("tokens: {:#?}", lexer.tokens);
//     let expected = vec![];
//     assert_eq!(lexer.tokens, expected);
// }

// #[test]
// fn while_basic() {
//     let source = "";
//     let mut lexer = Lexer::from_source(source);
//     let _ = lexer.parse();
//     // println!("tokens: {:#?}", lexer.tokens);
//     let expected = vec![];
//     assert_eq!(lexer.tokens, expected);
// }

// #[test]
// fn loop_basic() {
//     let source = "";
//     let mut lexer = Lexer::from_source(source);
//     let _ = lexer.parse();
//     // println!("tokens: {:#?}", lexer.tokens);
//     let expected = vec![];
//     assert_eq!(lexer.tokens, expected);
// }

#[test]
fn package() {
    let source = "
package     \"my.pkg\"
version     \"1.0.0\"
description \"My pkg does fun things...\"
files {
  \"./file1.yak\"
  \"./file2.yak\"
}
dependencies {
  pkg1.v1 \"http://github.com/Org1/repo1/pkg1@v1.1\"
  my.pkg2 \"../my.pkg2\"
}
import {
  pkg1.v1 { A b ^C :f1 }
}
export {
  abc
  Struct1 { a b }
  Struct2 { * }
  ^Trait1
  :func1
}
";
    let mut lexer = Lexer::from_source(source);
    let _ = lexer.parse();
    // println!("tokens: {:#?}", lexer.tokens);
    let expected = vec![
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwPackage },
        Token { ty: Sp },
        Token { ty: Sp },
        Token { ty: Sp },
        Token { ty: Sp },
        Token { ty: Sp },
        Token {
            ty: LitString("\"my.pkg\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwVersion },
        Token { ty: Sp },
        Token { ty: Sp },
        Token { ty: Sp },
        Token { ty: Sp },
        Token { ty: Sp },
        Token {
            ty: LitString("\"1.0.0\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwDescription },
        Token { ty: Sp },
        Token {
            ty: LitString("\"My pkg does fun things...\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwFiles },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: LitString("\"./file1.yak\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: LitString("\"./file2.yak\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwDependencies },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdPackage("pkg1.v1".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitString("\"http://github.com/Org1/repo1/pkg1@v1.1\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdPackage("my.pkg2".into()),
        },
        Token { ty: Sp },
        Token {
            ty: LitString("\"../my.pkg2\"".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwImport },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdPackage("pkg1.v1".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token {
            ty: IdType("A".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdVar("b".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdTrait("^C".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdFunc(":f1".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: KwExport },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdVar("abc".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdType("Struct1".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token {
            ty: IdVar("a".into()),
        },
        Token { ty: Sp },
        Token {
            ty: IdVar("b".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdType("Struct2".into()),
        },
        Token { ty: Sp },
        Token { ty: PunctBraceL },
        Token { ty: Sp },
        Token { ty: OpMul },
        Token { ty: Sp },
        Token { ty: PunctBraceR },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdTrait("^Trait1".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(2) },
        Token {
            ty: IdFunc(":func1".into()),
        },
        Token { ty: NL },
        Token { ty: Indent(0) },
        Token { ty: PunctBraceR },
        Token { ty: NL },
    ];
    assert_eq!(lexer.tokens, expected);
}

// Test Template
// #[test]
// fn test_() {
//     let source = "";
//     let mut lexer = Lexer::from_source(source);
//     let _ = lexer.parse();
//     // println!("tokens: {:#?}", lexer.tokens);
//     let expected = vec![];
//     assert_eq!(lexer.tokens, expected);
// }
