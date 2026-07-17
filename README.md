# rust-interpreter — a near-complete Lox implementation in Rust

This repository contains a near-complete Rust implementation of the Lox language (from Crafting Interpreters). It provides a full scanner, parser (AST), resolver, and a tree-walk interpreter with a working runtime supporting functions, classes, closures, and native calls.

## Key features

- Lexing
	- Tokenizes numbers (integers & floats), strings, identifiers, keywords, punctuation
	- Supports single-line comments (`//`) and line tracking for error messages

- Parsing
	- Full expression grammar with precedence (grouping, unary, binary, comparison, equality, logical)
	- Call and property access (`call`, `.`)
	- Function declarations and anonymous functions (lambdas)
	- Class declarations and method parsing (including `init` as an initializer)
	- Control flow statements: `if`/`else`, `while`, `for` (desugared to `while`), `print`, `return`
	- Variable declarations and assignments

- Static analysis / resolver
	- Lexical scope tracking with a resolver that computes depth for variable lookups
	- Detects invalid uses of `this`, `return` outside functions, and reads of vars in their own initializer

- Runtime / Interpreter
	- Tree-walk interpreter with nested `Environment`s and depth-based lookups
	- Value types: integers, floats, strings, booleans, `nil`, callables, and instances
	- Arithmetic, comparison, logical operators, and string concatenation
	- Functions: first-class functions, closures (capture environment), `return` handling, arity checks
	- Lambdas: create and call anonymous functions with captured closures
	- Classes & instances: define classes, create instances, instance fields, methods, and method binding (`this`)
	- Initializers (constructor-like `init` methods) return the instance

- Native functions & extras
	- `clock()` native function returning seconds since the epoch
	- Helpful runtime error reporting with source line numbers

## Requirements

- Rust stable toolchain (https://rustup.rs)
- A Unix-like shell to run `your_program.sh`

## Quickstart

Build and run the helper script with a command and input file:

```sh
./your_program.sh <command> test.lox
```

Common commands:

```sh
# Print tokens
./your_program.sh tokenize test.lox

# Print the AST in parenthesized form
./your_program.sh parse test.lox

# Evaluate a single expression (will not work for a full Lox file)
./your_program.sh evaluate test.lox

# Run a program consisting of statements
./your_program.sh run test.lox

# Dump tokens and parsed statements for debugging
./your_program.sh dbg test.lox
```

## Example

Given `test.lox`:

```lox
var x = 7 * 3 / 7 / 1;
print x;
```

You can tokenize, parse, or run the program using the helper script. The interpreter supports integer and float arithmetic, string concatenation, functions, classes, and more.

## Development & tests

- Run the full test suite:

```sh
cargo test
```

- Run specific tests:

```sh
cargo test lexer_tests
cargo test parser_tests
cargo test interpreter_tests
```

## Files of interest

- `src/lexer` — scanning and token representation
- `src/parser` — parser and resolver (scope depth calculation)
- `src/ast` — AST nodes, visitor and printer
- `src/runtime` — interpreter, environment, functions, classes, native callables

## References

- Language design: https://craftinginterpreters.com/the-lox-language.html
- Book: Crafting Interpreters by Robert Nystrom: https://craftinginterpreters.com/
