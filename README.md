# Yak

A toy programming language inspired by bits of Rust, Go, Python, and JavaScript.

# Syntax

See the [syntax defined here](./SYNTAX.md).

# Progress

1. Lexer
    - Done: Token parsing
    - Needs work:
        - Source code file, line, and column location
2. AST Parsing
    - See [yak-ast TODO](./yak-ast/TODO.md) for done and needs work.
    - TBD: AST Validation
3. AST -> IR
    - Prototyped:
        - Modules
        - Functions
        - Structs
        - Constants
5. IR Validation
    - TBD: Type checks
5. IR -> LLVM
    - Prototyped:
        - Modules: generates module `.ll` files
        - Functions (stubbed)
        - Structs (stubbed)
        - Constants (stubbed)
6. Linking
    - Done: Links all `.ll` files and generates a package binary using `clang`

# Unknowns

This list is some of the big unknowns to figure out...

- Memory management (gc vs lifetimes/borrow. Needs more research.)
- Memory layouts (can we leverage Rust for this?)
- String vs str: which one should we start with?
- Runtime (what does this look like? Make it synchronous and see if we can leverage Rust threads/channels to start out)

# Env Variables

- `YAK_HOME`: The path to use for the yak home directory.
- `YAK_LOG`: The loglevel to use for the yak-cli binary.
- `YAK_VERSION`: The yak version to use for the yak-cli tools.

```
# Define which yak version to use:
export YAK_HOME="~/.yak"
export YAK_VERSION="0.0.1"
export PATH="$YAK_HOME/yak/$YAK_VERSION/bin:$PATH"
```

# Yak Home

The `yak` home directory is where we store the versioned yak-cli binary, package src code, and compiled package cache.

By default, it should look like this:

```
~/.yak/yak/{version}
~/.yak/yak/{version}/bin
~/.yak/yak/{version}/pkg
~/.yak/yak/{version}/src
```

# yak-cli

The crude beginnings of a cli have been stubbed out to support building and getting packages.

## Build

Build the `yak.pkg` file in the current directory and resolve dependencies:

```
yak-cli build
```

Or, build some other local package relative to the current directory:

```
yak-cli build ../my/pkg1
```

Build artifacts should end up in the `{pkg}/target/module` and `{pkg}/target/bin` or `{pkg}/target/lib` directories.

## Get

Download remote packages locally and build them.

```
yak-cli get https://raw.githubusercontent.com/grippy/yak/master/examples/yak-pkg1
```