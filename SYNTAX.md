# Yak Syntax

Yak syntax uses Python-style indentation for nesting.

## Comments

Comments are defined using the pound symbol.

```python
# This is a comment
```

## Braces

Left and right braces are used throughout the language.
They're used for defining function arguments, struct, sets, lists, etc.

## Naming Rules

To make parsing easier, we have some naming conventions that should be followed:

- Packages: Should be lowercase alphanumeric and periods. Example: `my.pkg.name`

- Variables: Should be lowercase alphanumeric and underscores. Example `my_variable` or `_my_variable`.

- Types: Should be `PascalCased` alphanumeric and underscores. Example: `MyType1` or `MyType_1`

- Functions: Should be similar to variables and prefixed with a colon. Example: `:my_func`.

- Traits: Should be similar to Types and prefixed with a carrot symbol. Example: `^MyTrait`

### Packages

A package (i.e. `yak.pkg` file) is a unit of compilation that defines a name, version, dependencies, and what symbols to import and export.

Conceptually, these are similar to how modules work in Go and follow the same idea with how mapping dependencies work.

Package syntax looks like this:

```
package     "pkg.name"
description "A description goes here"
version     "1.0.0"
dependencies {
  pkg.name1 "../path"
  pkg.name2 "http://github.com/org1/repo1/pkg1"
}
import {
  pkg.name1 { Type1 }
  pkg.name2 { Type2 as SomeType }
}
export {
  var1
  MyType1
}
```

## Primitive Data Types

Boolean
- `true`
- `false`

String

- `String` or `str`

Numbers

- `float`, `float32` or `float64`
- `int`, `int8`, `int16`, `int32` or `int64`
- `uint`, `uint8`, `uint16`, `uint32` or `uint64`

## Variables

This is similar to how variables are defined in JavaScript ES6.

### Immutable

```rust
const name: String = "value"
```

### Mutable

```rust
let name: String = "value"
```

## Functions

Function signatures are defined using the following syntax.

```rust
fn :func_name { arg1: Type1 arg2: Type2 ... } ReturnType =>
  const X =
    ReturnType
      hello: "world"
  return some_value
```

- Function names are prefixed with a colon (i.e. `:`).
- Function arguments are separated by whitespace.
- Function bodies are defined by the Fat Arrows.
- Functions returns are explicit.

## Structs

Structs borrow from Rust/Go syntax but with the optional the need for curly braces.

### Keyword

- `struct`

### Definition

```rust
struct MyStruct
  field1: String
```

### Value

As a multiline assignment:
```rust
let my_struct =
  MyStruct
    field1: "value1"
```

Or, as a single line:
```rust
let my_struct = MyStruct { field1: "value1" }
```

## Traits

### Keyword

- `trait`

### Definition

```rust
trait ^MyTrait1
  type T1 = T
  fn :func1 self { arg1: Type1 } String
  fn :func2 self { arg1: Type2 } String
```

### Implementation

```rust
impl MyStruct ^MyTrait1
  fn :func1 self { arg1: Type1 } String =>
    return "Hello"
  fn :func2 self { arg1: Type3 } String =>
    return "Hello"
```

## Enums

Enums borrow from Rust syntax. You can define simple, struct, or tuple enum variants.

### Keyword

- `enum`

### Definition

```rust
enum MyEnum
  SimpleType
  StructType { field1: T1, ... }
  TupleType { T1, T2, ... }
```

### Value

```rust
const my_enum1 = MyEnum::SimpleType
const my_enum2 = MyEnum::StructType {
  field1: "hello"
}
const my_enum2 = MyEnum::StructType {
  "hello"
  "world"
}
```

## Tuples

### Keyword

- `tuple`

### Definition

```rust
tuple TupleType { String String }
```

### Value

```rust
const my_tuple = TupleType { "a" "b" }
```

## Lists

TBD how these should work.

### Keyword

- `list`

### Definition

```rust
list ListType[String]
type ListType = List[T]
```

### Value

```rust
const my_list = ListType::from { "a" "b" }
```

## Generics

Generics are currently stubbed out in the AST pass. These are currently TBD how they'll work. For now, here's how this is defined.


### Definition

```rust

trait ^MyTrait[T]
  type T1 = T
  fn func1 self {} Self::T1

struct MyStruct[T]
  field1: T

impl MyStruct[T] ^MyTrait[T]
  fn func1 self {} T =>
    return T:new {}

```

### Value

```rust
let my_struct =
  MyStruct[String]
    field1: "Hello"
```

## Expressions

An expression is anything that has or returns a value.

We can define multiple types of expressions:

- Boolean
- Arithmetic
- Logical
- Bitwise
- Value (number, strings, boolean, etc.)
- Assignments

### Rules

- Surrounding parenthesis are optional unless noted. _Parenthesis might be required for passing function arguments, tuple/list values, etc. If so, this depends on the complexity of the expression. _

- Implements top-down operator precedence using [Pratt parsing](https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing).

## Control Flow

### Conditions

```python
if [conditions] then
  ...
elif [conditions] then
  ...
else
  ...
```

### For/While/Loop

TBD

### Match/Switch

TBD