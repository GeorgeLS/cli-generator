# Overview

Disclaimer: This project is **WIP!**

The aim of this project is to act as a mini "metaprogramming language" in order to easily generate cli definition and
parsing\
code for different languages as well as autocompletion scripts for shells.

## Syntax of input language

The syntax of the input language is basically the same as you would define a `Rust struct` with derive macros (e.g
using `clap`)\
but without the `derive` keywords.

Here's an input example:

```rust
#[main]
struct Cli {
    #[short, long]
    int_field: i64,
    #[short, long]
    optional_string_field: Option<String>,
}
```

You can have a look in the `examples` folder with the input files as well as with their generated files and source files
using those.

Since the project is work in progress, I won't provide a documentation with all the available keywords and their usage
right now, but I will once things are stable.

## Under the hood

The tool performs the following steps when being run:

- Lexing of the input (see `src/lexer.rs`)
- Parsing of the input (see `src/parse.rs`)
- Semantic analysis of the parsed input (see `src/semantic.rs`)
- Generation of the cli ("backend") (see `src/generate`)

Right now, it will only generate to `C++` but my goal is to add more languages as well as autocompletion script
generation for different shells.

## Error reporting

If the input is not syntactically or semantically correct, then a nice error report message is provided trying to
explain what is wrong as good as possible.

Here's a parsing error example:

Input:

```rust
#[main]
struct Cli

#[short]
values: Vec<u32>,
}
```

Output:

```bash
error: Parser error
  |
1 | #[main]
2 | struct Cli
3 |     #[short]
  |     ^ Unexpected token
4 |     values: Vec<u32>,
5 | }
  |
  = help: Tokens can be any of: {
```

Here we understand that we are missing a left bracket `{`.

Here's a semantic error example where we define the same field twice:

Input:

```rust
#[main]
struct Cli {
    #[short]
    values: Vec<u32>,
    #[short, long]
    foo: bool,
    #[short, long]
    values: i16,
}

```

Output:

```bash
error: Multiple field definition
   |
 8 |     #[short, long]
 9 |     values: i16,
   |     ^^^^^^ Redefinition of field
10 | }
   |
 4 |     #[short]
 5 |     values: Vec<u32>,
   |     ------ info: Has already been defined here
 6 |     #[short, long]
   |
```

## Usage

You can use `cargo` in order to compile and run the executable. Assuming you have the tool installed you can run:

```bash
cli-generator -i <input_path> -o <output_path>
```

