# Modules

## Command line interface (`cli`)

This module will be responsible for parsing the command-line arguments using `clap`. This includes:

- The path to the Logo program file
- Output image path
- Dimensions of the output image

## Logo parser (`parser`)

This will parse the Logo program into tokens and construct an abstract syntax tree (AST).

### Crates

#### `nom`

Used to create the parsers for different commands. It is a parser combinator library.

## AST and command representation (`ast`)

This module will contain the definition for the AST data structure, representing different Logo commands. Enums will be used for different command types and structs for their parameters.

## Turtle state management (`turtle`)

This module will maintain the current state of the turtle, including the following parameters:

- Position
- Orientation
- Pen state

This module is also responsible for applying transformations to the turtle's state based on the executed commands.

## Command execution engine (`executor`)

This module is responsible for traversing the AST and execute the commands accordingly, updating both the state of the turtle and the drawing.

The visitor pattern will be utilised to ensure separation between the command execution and turtle state management.

## Drawing backend (`renderer`)

This module will interface with the `unsvg` crate to render the turtle's movements ot an SVG or PNG image based on the command executions.

## Error handling and reporting (`error`)

This module will define and manage error types for parsing, execution, and rendering phases.

This module should utilise Rust's `Result` and `Error` traits for error handling.

### Crates

#### `thiserror`

Used to simplify the definition and implementation of custom error types by deriving the `std::error:Error` trait and supporting error conversion.

#### `miette`

Used to enhance error reporting and user experience by providing rich diagnostic information, including code snippets, context, and suggestions.
