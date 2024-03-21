use nom_supreme::error::{BaseErrorKind, ErrorTree, GenericErrorTree, StackContext};

use crate::parsers;

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

pub fn format_parse_error<'a>(input: &'a str, e: ErrorTree<parsers::Span<'a>>) -> ParseError<'a> {
    match e {
        GenericErrorTree::Base { location, kind } => {
            let offset = location.location_offset().into();
            ParseError {
                src: input,
                span: miette::SourceSpan::new(offset, 0_u8.into()),
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
                        span: miette::SourceSpan::new(offset, 0_u8.into()),
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
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum InterpreterError {
    #[error("Variable not found: {0}")]
    UndefinedVariable(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
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
}
