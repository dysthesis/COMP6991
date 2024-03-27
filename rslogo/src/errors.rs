use nom_locate::LocatedSpan;
use nom_supreme::error::{BaseErrorKind, ErrorTree, GenericErrorTree, StackContext};

/// Convenient type alias
pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("Parse error")]
pub struct ParseError<'b> {
    #[source_code]
    src: &'b str,

    #[label("{kind}")]
    span: miette::SourceSpan,

    kind: BaseErrorKind<&'b str, Box<dyn std::error::Error + Send + Sync + 'static>>,

    #[related]
    others: Vec<ParseErrorContext<'b>>,
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("Parse error context")]
pub struct ParseErrorContext<'b> {
    #[source_code]
    src: &'b str,
    #[label("{context}")]
    span: miette::SourceSpan,
    context: StackContext<&'b str>,
}

pub fn format_parse_error<'a>(input: &'a str, e: ErrorTree<Span<'a>>) -> ParseError<'a> {
    match e {
        GenericErrorTree::Base { location, kind } => {
            let offset = location.location_offset().into();
            ParseError {
                src: input,
                span: miette::SourceSpan::new(offset, 1_u8.into()),
                kind,
                others: Vec::new(),
            }
        }
        GenericErrorTree::Stack { base, contexts } => {
            let mut base = format_parse_error(input, *base);
            let mut contexts: Vec<ParseErrorContext> = contexts
                .into_iter()
                .map(|(location, context)| {
                    let offset = location.location_offset().into();
                    ParseErrorContext {
                        src: input,
                        span: miette::SourceSpan::new(offset, 1_u8.into()),
                        context,
                    }
                })
                .collect();
            base.others.append(&mut contexts);
            base
        }
        GenericErrorTree::Alt(alt_errors) => {
            // get the error with the most context
            // TODO: figure out what to do on ties
            alt_errors
                .into_iter()
                .map(|e| format_parse_error(input, e))
                .max_by_key(|formatted| formatted.others.len())
                .unwrap()
        }
    }
}

// TOKEN ERRORS
#[derive(thiserror::Error, miette::Diagnostic, Clone, Debug, PartialEq)]
pub enum InterpreterError {
    #[error("Variable not found: {0}")]
    UndefinedVariable(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Invalid type for {0}: {1}")]
    InvalidType(String, String),

    #[error("Unsuccessful operation: {0}")]
    UnsuccessfulOperation(String),
}

impl InterpreterError {
    pub fn undefined_var(name: &str) -> Self {
        InterpreterError::UndefinedVariable(name.into())
    }

    pub fn division_by_zero() -> Self {
        InterpreterError::DivisionByZero
    }

    pub fn unsupported_operation(name: &str) -> Self {
        InterpreterError::UnsupportedOperation(name.into())
    }

    pub fn invalid_type(field: &str, var_type: &str) -> Self {
        InterpreterError::InvalidType(field.into(), var_type.into())
    }

    pub fn unsuccessful_operation(operation: &str) -> Self {
        InterpreterError::UnsuccessfulOperation(operation.into())
    }
}

#[derive(thiserror::Error, miette::Diagnostic, Debug, PartialEq)]
pub enum TurtleError {
    #[error("Colour out of range: {0}")]
    ColourOutOfRange(f32), // TODO: Make miette provide a help message informing the correct range.
    #[error("Angle out of range: {0}")]
    AngleOutOfRange(f32), // TODO: Make miette provide a help message informing the correct range.
    #[error("Invalid coordinates: ({0}, {1})")]
    InvalidCoordinates(f32, f32),
}
