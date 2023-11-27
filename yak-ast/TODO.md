
# Top-Level Statements

## ConstStmt
  - [] Parse
    - [] AssignStmt
      - [] ExprStmt
  - [] Validation

## EnumStmt
  - [x] Parse
    - [x] Struct Type
    - [x] Tuple Type
  - [] Validation

## FuncStmt
  - [] Parse
    - [] FuncTypeStmt
    - [] FuncInputTypeStmt
    - [] FuncOutputTypeStmt
    - [] FuncBodyStmt

## ImplTraitStmt
  - [] Parse
    - [] ImplTraitStmt

## LetStmt
  - [] Parse
    - [] AssignStmt
      - [] ExprStmt
  - [] Validation

## ListStmt

## StructStmt
  - [x] Parse
    - [x] TypeStmt
    - [x] StructFieldStmt

## TupleStmt

## TraitStmt
  - [] Parse
    - [] TraitStmt
    - [] TypeStmt
    - [] FuncStmt

## TestCaseStmt

## NewTypeStmt

## PackageStmt

# Blocks

## ConstStmt
## LetStmt
## VarStmt
## ForLoopStmt
## ForInStmt
## IfStmt
## MatchStmt
## WhileStmt

# Values

## EnumValueStmt
## FuncValueStmt
## StructValueStmt
  - [x] Parse
    - [x] TypeStmt
    - [x] StructFieldValueStmt
## TupleValueStmt
## ListValueStmt

# Multi-use

## ExprStmt
  - [] Should have return type stub for verification
    - i.e. Left side defines Type but right doesn't
  - [] ExprParser (PrattParser)
    - [] Primary
      - IdType
        - StructValueStmt
        - EnumValueStmt
        - TupleValueStmt
        - ListValueStmt
      - IdFunc
      - IdPackage

## TypeStmt
  - [x] Parse
    - [x] TypeStmt
