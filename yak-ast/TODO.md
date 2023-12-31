
# Top-Level Statements

## ConstStmt
  - [x] Parse
    - [x] AssignStmt
      - [x] ExprStmt

## EnumStmt
  - [x] Parse
    - [x] Struct Type
    - [x] Tuple Type

## FuncStmt
  - [x] Parse
    - [x] FuncTypeStmt
    - [x] FuncInputTypeStmt
    - [x] FuncOutputTypeStmt
    - [x] FuncBodyStmt

## LetStmt
  - [x] Parse
    - [x] AssignStmt
      - [x] ExprStmt

## ListStmt
  - [] Parse
    - [] TypeStmt

## StructStmt
  - [x] Parse
    - [x] TypeStmt
    - [x] StructFieldStmt

## TupleStmt
  - [x] Parse
    - [x] TypeStmt

## TraitStmt
  - [] Parse
    - [] TraitStmt
    - [] TypeStmt
    - [] FuncStmt

## ImplTraitStmt
  - [] Parse
    - [] ImplTraitStmt

## TestCaseStmt
  - [] Parse

## NewTypeStmt
  - [] Parse

## PackageStmt
  - [x] PackageStmt
    - [x] Description
    - [x] VersionStmt
    - [x] FileStmt
    - [x] DependencyStmt
    - [x] ImportStmt
    - [x] ExportStmt

# Blocks
  - [] Parse

## ConstStmt
  - [x] Parse
    - [x] VarStmt & TypeStmt

## LetStmt
  - [x] Parse
    - [x] VarStmt & TypeStmt

## VarStmt
  - [x] Parse

## ForLoopStmt
## ForInStmt
## IfStmt
  - [x] Parse
    - [x] ExprStmt

## MatchStmt
## WhileStmt

# Values

## FuncValueStmt
  - [x] Parse
    - [x] Input

## StructValueStmt
  - [x] Parse
    - [x] TypeStmt
    - [x] StructFieldValueStmt

## EnumValueStmt
  - [] Parse
    - [] Simple
    - [] Struct types
    - [] Tuple types

## TupleValueStmt
  - [] Parse

## ListValueStmt
  Might be hard to tell the diff between a list and tuple
  with how these are currently defined.
  - [] Parse

# Multi-use

## ExprStmt
  - [x] Should have return type stub for verification
    - i.e. Left side defines Type but right doesn't
  - [x] ExprParser (PrattParser)
    - [x] Primary
        - [x] IdType
        - [x] StructValueStmt
        - [x] EnumValueStmt
        - [x] TupleValueStmt
        - [ ] ListValueStmt
      - [x] IdVar
      - [x] IdFunc
      - [x] IdPackage

## TypeStmt
  - [x] Parse
    - [x] TypeStmt
