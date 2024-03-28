use nom::{
    branch::alt,
    bytes::complete::{take_till, take_until, take_while},
    character::{
        complete::{alphanumeric0, alphanumeric1, char, multispace0},
        is_alphanumeric,
    },
    combinator::opt,
    multi::many0,
    number::complete::float,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use crate::{
    errors::{format_parse_error, ParseError, Span},
    tokens::{Command, EvalResult, Expression},
};

/// Macro to reduce boilerplate for arithmetic parsing
macro_rules! parse_operation_expression {
    ($fn_name:ident, $op:expr, $constructor:path) => {
        fn $fn_name(input: Span) -> IResult<Span, Expression, ErrorTree<Span>> {
            preceded(
                tag($op),
                separated_pair(parse_expression, multispace0, parse_expression)
                    .preceded_by(multispace0),
            )
            .map(|(lhs, rhs)| $constructor(Box::new(lhs), Box::new(rhs)))
            .context(concat!("when parsing ", stringify!($op), " expression"))
            .parse(input)
        }
    };
}

/// Macro to reduce boilerplate for argument-less queries
macro_rules! parse_query_expression {
    ($fn_name:ident, $op:expr, $context:expr, $constructor:path) => {
        fn $fn_name(input: Span) -> IResult<Span, Expression, ErrorTree<Span>> {
            tag($op)
                .map(|_| $constructor)
                .context($context)
                .parse(input)
        }
    };
}

/// Macro to reduce boilerplate for parsing a verb
macro_rules! command_parser {
    ($tag:expr, $constructor:path) => {
        tag($tag)
            .context(concat!("parsing as ", stringify!($tag)))
            .map(|_| $constructor as fn(Expression) -> Command)
    };
}

macro_rules! variable_command_parser {
    ($tag:expr, $constructor:path) => {
        tag($tag)
            .context(concat!("parsing as ", stringify!($tag)))
            .map(|_| $constructor as fn(Expression, Expression) -> Command)
    };
}
/// Macro to reduce boilerplate for parsing a verb
macro_rules! control_flow_parser {
    ($tag:expr, $constructor:expr) => {
        tag($tag)
            .context(concat!("parsing as ", stringify!($tag)))
            .map(|_| $constructor as fn(Expression, Vec<Command>) -> Command)
    };
}

/// Parse the given input as a literal value. This will return an instance of `Expression::Value`
/// A literal value must be preceeded by a double quote (`"`).
///
/// # Example
/// ```
/// assert_eq!(parse_value_expression(Span::new("\"TRUE")), Expression::Value(EvalResult::Bool(true)));
/// assert_eq!(parse_value_expression(Span::new("\"FALSE")), Expression::Value(EvalResult::Bool(false)));
/// assert_eq!(parse_value_expression(Span::new("\"2.54")), Expression::Value(EvalResult::Float(2.54)))
/// ```
fn parse_value_expression(input: Span) -> IResult<Span, Expression, ErrorTree<Span>> {
    /*
     * A value literal can be one of the following: a float (f32), or a boolean. However, since Logo represents
     * booleans as the strings "TRUE" and "FALSE", we cannot use nom's built-in bool parser. Instead, we will parse
     * them as strings, using tag.
     */
    alt((
        // parse number
        float
            // Instead of a string, we want to return the corresponding enum instance
            .map(|res: f32| Expression::Value(EvalResult::Float(res)))
            .context("parsing literal value as float"),
        // Parse 'true' boolean
        tag("TRUE")
            // The parsed value does not matter here. Rather, if the parser succeeds at all, we return an instance of the enum, disregarding the parsed string.
            .map(|_| Expression::Value(EvalResult::Bool(true)))
            .context("parsing literal value as boolean 'true'"),
        // Parse 'false' bolean
        tag("FALSE")
            // The parsed value does not matter here. Rather, if the parser succeeds at all, we return an instance of the enum, disregarding the parsed string.
            .map(|_| Expression::Value(EvalResult::Bool(false)))
            .context("parsing literal value as boolean 'true'"),
        // Parse variable name
        take_till(|c: char| -> bool { c == ' ' || c == '"' || c == '\n' })
            .map(|name: Span| {
                Expression::Value(EvalResult::String(name.into_fragment().to_owned()))
            })
            .context("parsing variable name"),
    ))
    // The specifications for the Logo language requires that a value be preceeded by a singular double-quote
    .preceded_by(tag("\""))
    .context("parsing literal value")
    .parse(input)
}

/// Parse the given input as a variable. This will return an instance of `Expression::GetVariable`
/// A variable must be preceeded by a double quote (`:`).
///
/// # Example
/// ```
/// assert_eq!(parse_value_expression(Span::new("\"TRUE")), Expression::Value(EvalResult::Bool(true)));
/// assert_eq!(parse_value_expression(Span::new("\"FALSE")), Expression::Value(EvalResult::Bool(false)));
/// assert_eq!(parse_value_expression(Span::new("\"2.54")), Expression::Value(EvalResult::Float(2.54)))
/// ```
fn parse_getvariable_expression(input: Span) -> IResult<Span, Expression, ErrorTree<Span>> {
    delimited(
        tag(":"),
        take_till(|c: char| -> bool { c == ' ' || c == '"' || c == '\n' }),
        multispace0,
    )
    // We want to return a token instead of the actual float
    .map(|res: Span| -> Expression {
        Expression::GetVariable(Box::new(Expression::Variable(EvalResult::String(
            res.into_fragment().into(),
        ))))
    })
    // Additional context for error messages
    .context("parsing variable")
    .parse(input)
}

parse_query_expression!(
    parse_xcor_expression,
    "XCOR",
    "parsing x-coordinate query",
    Expression::XCor
);
parse_query_expression!(
    parse_ycor_expression,
    "YCOR",
    "parsing y-coordinate query",
    Expression::YCor
);
parse_query_expression!(
    parse_heading_expression,
    "HEADING",
    "parsing heading query",
    Expression::Heading
);
parse_query_expression!(
    parse_colour_expression,
    "COLOR",
    "parsing colour query",
    Expression::Colour
);

fn parse_comment(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    preceded(tag("//"), take_until("\n"))
        .map(|_| Command::Comment)
        .context("parsing comment")
        .parse(input)
}

parse_operation_expression!(parse_addition_expression, "+", Expression::Add);
parse_operation_expression!(parse_subtraction_expression, "-", Expression::Subtract);
parse_operation_expression!(parse_multiplication_expression, "*", Expression::Multiply);
parse_operation_expression!(parse_division_expression, "/", Expression::Divide);
parse_operation_expression!(parse_equality_expression, "EQ", Expression::Equals);
parse_operation_expression!(parse_inequality_expression, "NE", Expression::NotEquals);
parse_operation_expression!(parse_greater_than_expression, "GT", Expression::GreaterThan);
parse_operation_expression!(parse_less_than_expression, "LT", Expression::LessThan);
parse_operation_expression!(parse_and_expression, "AND", Expression::And);
parse_operation_expression!(parse_or_expression, "OR", Expression::Or);

fn parse_expression(input: Span) -> IResult<Span, Expression, ErrorTree<Span>> {
    alt((
        parse_value_expression,
        parse_getvariable_expression,
        parse_addition_expression,
        parse_subtraction_expression,
        parse_multiplication_expression,
        parse_division_expression,
        parse_equality_expression,
        parse_inequality_expression,
        parse_greater_than_expression,
        parse_less_than_expression,
        parse_and_expression,
        parse_or_expression,
        parse_xcor_expression,
        parse_ycor_expression,
        parse_colour_expression,
        parse_heading_expression,
    ))
    // .cut()
    .delimited_by(multispace0)
    .context("parsing expression")
    .parse(input)
}

fn parse_pen_state_commands(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    alt((
        tag("PENUP")
            .context("parsing as PENUP")
            .map(|_| Command::PenUp),
        tag("PENDOWN")
            .context("parsing as PENDOWN")
            .map(|_| Command::PenDown),
    ))
    .context("parsing as pen state command")
    .parse(input)
}

fn parse_single_expression_commands(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    let parse_verb = alt((
        command_parser!("FORWARD", Command::Forward),
        command_parser!("BACK", Command::Back),
        command_parser!("LEFT", Command::Left),
        command_parser!("RIGHT", Command::Right),
        command_parser!("SETPENCOLOR", Command::SetPenColor),
        command_parser!("TURN", Command::Turn),
        command_parser!("SETHEADING", Command::SetHeading),
        command_parser!("SETX", Command::SetX),
        command_parser!("SETY", Command::SetY),
    ))
    .context("parsing verb for a single expression command");

    separated_pair(parse_verb, multispace0, parse_expression)
        .map(|(verb, expression)| verb(expression))
        .parse(input)
}

fn parse_variable_manipulation_commands(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    let parse_verb = alt((
        variable_command_parser!("MAKE", Command::MakeVariable),
        variable_command_parser!("ADDASSIGN", Command::Increment),
    ))
    .context("parsing verb for a variable manipulation command");

    separated_pair(
        parse_verb,
        multispace0,
        separated_pair(parse_expression, multispace0, parse_expression),
    )
    .map(|(verb, (name, val))| verb(name, val))
    .parse(input)
}

fn parse_control_flow_commands(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    let verb = alt((
        control_flow_parser!("IF", Command::If),
        control_flow_parser!("WHILE", Command::While),
    ))
    .context("parsing verb for a control flow command");

    let commands = parse_commands_many;

    separated_pair(
        verb,
        multispace0,
        separated_pair(
            parse_expression.context("parsing condition for a control flow expression"),
            multispace0,
            delimited(
                tag("[").context("parsing opening delimiter for a control flow expression"),
                commands.context("parsing commands inside control flow expression"),
                tag("]").context("parsing closing delimiters for a control flow expression"),
            )
            .context("parsing body of a control flow expression"),
        ),
    )
    .context("parsing a control flow expression")
    .map(|(verb, (conditions, commands))| verb(conditions, commands))
    .parse(input)
}

fn parse_procedure_definition(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    delimited(
        tag("TO"),
        tuple((alphanumeric0, many0(parse_expression), parse_commands_many)),
        tag("END"),
    )
    .map(|(name, args, commands)| {
        Command::ProcedureDefine(
            Expression::Value(EvalResult::String(name.to_string())),
            args,
            commands,
        )
    })
    .parse(input)
}

fn parse_procedure_invocation(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    let name = alphanumeric1.context("parsing procedure name for invocation");
    let arguments = many0(parse_expression).context("parsing arguments for a procedure invocation");
    separated_pair(name, multispace0, opt(arguments))
        .map(|(name, args): (Span, Option<Vec<Expression>>)| -> Command {
            let args: Vec<Expression> = match args {
                Some(val) => val,
                None => Vec::new(),
            };

            Command::ProcedureExec(name.into_fragment().to_string(), args)
        })
        .parse(dbg!(input))
}

fn parse_command_expression(input: Span) -> IResult<Span, Command, ErrorTree<Span>> {
    alt((
        parse_comment,
        parse_pen_state_commands,
        parse_single_expression_commands,
        parse_control_flow_commands,
        parse_variable_manipulation_commands,
        // For some reason, the "TO" in the procedure definition is being interpreted as a procedure invocation
        parse_procedure_definition,
        parse_procedure_invocation,
    ))
    .delimited_by(multispace0)
    .context("parsing a single command")
    .parse(dbg!(input))
}

fn parse_commands_many(input: Span) -> IResult<Span, Vec<Command>, ErrorTree<Span>> {
    many0(parse_command_expression)
        // Ignore comments
        .map(|res: Vec<Command>| -> Vec<Command> {
            res.into_iter()
                .filter_map(|x: Command| -> Option<Command> {
                    match x {
                        Command::Comment => None,
                        _ => Some(x),
                    }
                })
                .collect()
        })
        .context("parsing multiple commands")
        .parse(input)
}

pub fn parse(input: &str) -> Result<Vec<Command>, ParseError> {
    match parse_commands_many
        // Cut is necessary to get full backtrace
        .cut()
        // Throw an error if there are any unparsed strings
        .all_consuming()
        .context("parsing program")
        .parse(Span::new(input))
    {
        Ok((_, res)) => Ok(res),
        Err(e) => {
            println!("{:?}", e);
            match e {
                // In nom, Incomplete represents a parse that is unsuccessful due to a lack of information,
                // usually in the context of streaming parsers. It's the parser's way of saying "I can't
                // determine what this is, I need more information please." We already have all the information
                // we'll ever get from the start, so this error should never occur
                nom::Err::Incomplete(_) => unreachable!("We're not using streaming parsers"),
                // For the other two errors that may actually happen, we want to format them for miette to use.
                nom::Err::Error(e) => Err(format_parse_error(input, e)),
                nom::Err::Failure(e) => Err(format_parse_error(input, e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::Program;

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn pen_commands_shouldnt_consume_anything_else() {
        let input = "PENDOWNextra";

        let (remainder, res): (Span, Command) =
            parse_pen_state_commands(Span::new(input)).expect("This should be valid");
        assert_eq!(
            (remainder.into_fragment(), res),
            ("extra", Command::PenDown)
        );
    }

    #[test]
    fn procedure_definition() {
        let input: &str = "TO Line\nPENDOWN\nFORWARD \"50\nPENUP\nEND";
        let expected: Vec<Command> = vec![Command::ProcedureDefine(
            Expression::Value(EvalResult::String(input.to_string())),
            Vec::new(),
            vec![
                Command::PenDown,
                Command::Forward(Expression::Value(EvalResult::Float(50.0))),
                Command::PenUp,
            ],
        )];

        let res: Vec<Command> = parse(input).expect("this should be valid");
        assert_eq!(res, expected)
    }
    #[test]
    fn procedure_invocation_inside_control_flow() {
        let input: &str = "IF \"TRUE [\nBox\n]";
        let expected: Vec<Command> = vec![Command::If(
            Expression::Value(EvalResult::Bool(true)),
            vec![Command::ProcedureExec(String::from("Box"), Vec::new())],
        )];
        let res: Vec<Command> = parse(input).expect("this should be valid");
        assert_eq!(res, expected);
    }

    #[test]
    fn tolerate_whitespace() {
        let input: &str = "  PENUP  ";
        let expected: Command = Command::PenUp;
        let (_, res): (_, Command) =
            parse_command_expression(Span::new(input)).expect("valid syntax");
        assert_eq!(res, expected);
        let input: &str = "\nPENUP\n";
        let expected: Command = Command::PenUp;
        let (_, res): (_, Command) =
            parse_command_expression(Span::new(input)).expect("valid syntax");
        assert_eq!(res, expected);
        let input: &str = "\nPENUP\n\nPENDOWN\n";
        let expected: Vec<Command> = vec![Command::PenUp, Command::PenDown];
        let (_, res): (_, Vec<Command>) =
            parse_commands_many(Span::new(input)).expect("valid syntax");
        assert_eq!(res, expected);
    }
    #[test]
    fn multiple() {
        let input: &str = "PENUP\nFORWARD \"10\nPENDOWN";
        let expected: Vec<Command> = vec![
            Command::PenUp,
            Command::Forward(Expression::Value(EvalResult::Float(10.0))),
            Command::PenDown,
        ];
        let (_, res): (_, Vec<Command>) =
            parse_commands_many(Span::new(input)).expect("valid syntax");
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_value() {
        let input = "\"100";
        let (_, res) = parse_value_expression(Span::new(input)).expect("This should be valid");
        assert_eq!(res, Expression::Value(EvalResult::Float(100.0)));

        let input = "\"TRUE";
        let (_, res) = parse_value_expression(Span::new(input)).expect("This should be valid");
        assert_eq!(res, Expression::Value(EvalResult::Bool(true)));

        let input = "\"var_name1";
        let (_, res) = parse_value_expression(Span::new(input)).expect("This should be valid");
        assert_eq!(
            res,
            Expression::Value(EvalResult::String(String::from("var_name1")))
        );
    }

    #[test]
    fn parse_recursive_expression() {
        let input = "EQ + \"1 \"1 \"2";
        let (_, res) = parse_expression(Span::new(input)).unwrap();
        assert_eq!(
            res,
            Expression::Equals(
                Box::new(Expression::Add(
                    Box::new(Expression::Value(EvalResult::Float(1.0))),
                    Box::new(Expression::Value(EvalResult::Float(1.0)))
                )),
                Box::new(Expression::Value(EvalResult::Float(2.0)))
            )
        )
    }
    #[test]
    fn conditional() {
        let input = "IF EQ + \"1 \"1 \"2 [PENUP\nFORWARD \"50\nPENDOWN\n]";
        let expected = Command::If(
            Expression::Equals(
                Box::new(Expression::Add(
                    Box::new(Expression::Value(EvalResult::Float(1.0))),
                    Box::new(Expression::Value(EvalResult::Float(1.0))),
                )),
                Box::new(Expression::Value(EvalResult::Float(2.0))),
            ),
            vec![
                Command::PenUp,
                Command::Forward(Expression::Value(EvalResult::Float(50.0))),
                Command::PenDown,
            ],
        );
        let (_, result) =
            parse_control_flow_commands(Span::new(input)).expect("This should be valid syntax");
        assert_eq!(result, expected);
    }

    macro_rules! float_operations_strategy {
        ($fn:ident, $op:expr) => {
            fn $fn() -> impl Strategy<Value = (String, f32, f32)> {
                (any::<f32>(), any::<f32>())
                    .prop_map(move |(a, b)| (format!("{} \"{} \"{}", $op, a, b), a, b))
            }
        };
    }

    macro_rules! bool_operations_strategy {
        ($fn:ident, $op:expr) => {
            fn $fn() -> impl Strategy<Value = (String, bool, bool)> {
                (any::<bool>(), any::<bool>()).prop_map(move |(a, b)| {
                    let a_str = if a { "TRUE" } else { "FALSE" };
                    let b_str = if b { "TRUE" } else { "FALSE" };
                    (format!("{} \"{} \"{}", $op, a_str, b_str), a, b)
                })
            }
        };
    }
    // macro_rules! bool_operations_strategy {
    //     ($fn:ident, $op:expr) => {
    //         fn $fn() -> impl Strategy<Value = (String, bool, bool)> {
    //             (any::<bool>(), any::<bool>())
    //                 .prop_map(move |(a, b)| (format!("{} \"{} \"{}", $op, a, b), a, b))
    //         }
    //     };
    // }
    float_operations_strategy!(addition_test, "+");
    float_operations_strategy!(subtraction_test, "-");
    float_operations_strategy!(multiplication_test, "*");
    float_operations_strategy!(division_test, "/");
    float_operations_strategy!(float_equals_test, "EQ");
    float_operations_strategy!(float_notequals_test, "NE");
    float_operations_strategy!(float_gt_test, "GT");
    float_operations_strategy!(float_lt_test, "LT");
    bool_operations_strategy!(bool_equals_test, "EQ");
    bool_operations_strategy!(bool_notequals_test, "NE");
    bool_operations_strategy!(bool_gt_test, "GT");
    bool_operations_strategy!(bool_lt_test, "LT");

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100000))]

        #[test]
        fn test_parse_addition_expression((input, a, b) in addition_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::Add(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse addition expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }

        #[test]
        fn test_parse_subtraction_expression((input, a, b) in subtraction_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::Subtract(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse addition expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }

        #[test]
        fn test_parse_multiplication_expression((input, a, b) in multiplication_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::Multiply(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse multiplication expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }

        #[test]
        fn test_parse_division_expression((input, a, b) in division_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::Divide(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse addition division or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }

        #[test]
        fn test_parse_float_equals_expression((input, a, b) in float_equals_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::Equals(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse equals expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }
        #[test]
        fn test_parse_float_notequals_expression((input, a, b) in float_notequals_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::NotEquals(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse not equals expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }
        #[test]
        fn test_parse_float_greaterthan_expression((input, a, b) in float_gt_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::GreaterThan(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse graeter than expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }

        #[test]
        fn test_parse_float_lessthan_expression((input, a, b) in float_lt_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::LessThan(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Float(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Float(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse less than expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }
        #[test]
        fn test_parse_bool_equals_expression((input, a, b) in bool_equals_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::Equals(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Bool(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Bool(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse equals expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }
        #[test]
        fn test_parse_bool_notequals_expression((input, a, b) in bool_notequals_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::NotEquals(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Bool(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Bool(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse not equals expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }
        #[test]
        fn test_parse_bool_greaterthan_expression((input, a, b) in bool_gt_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::GreaterThan(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Bool(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Bool(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse graeter than expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }

        #[test]
        fn test_parse_bool_lessthan_expression((input, a, b) in bool_lt_test()) {
            let span = Span::new(&input);
            match parse_expression(span) {
                Ok((remaining, Expression::LessThan(lhs, rhs))) => {
                    // Ensure the expression was fully consumed
                    assert!(remaining.fragment().is_empty(), "Input was not fully consumed");

                    // Example assertions (you'll need to replace these with actual logic to extract values from `lhs` and `rhs`)
                    // Dummy program for evaluation
                    let context = Program::new(Vec::new());

                    let lhs_val = lhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(lhs_val, EvalResult::Bool(a), "LHS value does not match expected");

                    let rhs_val = rhs.eval(&context).expect("A simple Expression::Value should not fail to evaluate");
                    assert_eq!(rhs_val, EvalResult::Bool(b), "RHS value does not match expected");
                },

                Err(e) => panic!("Failed to parse less than expression or incorrect expression type: {:?}", e),
                _ => panic!("This really shouldn't happen."),
            }
        }
    }
}
