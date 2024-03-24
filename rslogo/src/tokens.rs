use crate::errors::InterpreterError;
use crate::turtle::Turtle;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::ops::{Add, Div, Mul, Sub};
/// Macro to reduce boilerplate for arithmetic expressions
macro_rules! arithmetic_operation {
    ($op:ident, $lhs:expr, $rhs:expr, $context:expr, $err_msg:expr) => {{
        let lhs_result: EvalResult = $lhs.eval($context)?;
        let rhs_result: EvalResult = $rhs.eval($context)?;

        match (lhs_result, rhs_result) {
            (EvalResult::Float(lhs_val), EvalResult::Float(rhs_val)) => {
                Ok(EvalResult::Float(lhs_val.$op(rhs_val)))
            }
            _ => Err(InterpreterError::unsupported_operation($err_msg)),
        }
    }};
}
/// Macro to reduce boilerplate for arithmetic expressions
macro_rules! comparison {
    ($op:ident, $lhs:expr, $rhs:expr, $context:expr) => {{
        let lhs_result: EvalResult = $lhs.eval($context)?;
        let rhs_result: EvalResult = $rhs.eval($context)?;

        match (lhs_result, rhs_result) {
            (EvalResult::Float(lhs_val), EvalResult::Float(rhs_val)) => {
                Ok(EvalResult::Bool(lhs_val.$op(&rhs_val)))
            }
            (EvalResult::Bool(lhs_val), EvalResult::Bool(rhs_val)) => {
                Ok(EvalResult::Bool(lhs_val.$op(&rhs_val)))
            }
            _ => Err(InterpreterError::unsupported_operation(
                "comparison of different types",
            )),
        }
    }};
}
/// Macro to reduce boilerplate for logical operations
macro_rules! logical_operation {
    ($op:tt, $lhs:expr, $rhs:expr, $context:expr) => {{
        let lhs_result: EvalResult = $lhs.eval($context)?;
        let rhs_result: EvalResult = $rhs.eval($context)?;

        match (lhs_result, rhs_result) {
            (EvalResult::Bool(lhs_val), EvalResult::Bool(rhs_val)) => {
                Ok(EvalResult::Bool(lhs_val $op rhs_val))
            }
            _ => Err(InterpreterError::unsupported_operation("logical operation of non-booleans")),
        }
    }};
}

/// Ensure that only these types can ever be ultimately produced by the evaluation of expressions
#[derive(Debug, Clone, PartialEq)] // I think there might be a better way of doing this, but bools and f32 are small and cheap anyways
pub(crate) enum EvalResult {
    Bool(bool),
    Float(f32),
    String(String),
}

/// Expressions are instructions which returns a value, but do not perform any actions.
/// This is contrary to Commands, which perform actions, but do not return any value.
///
/// Example:
/// ```
/// let lhs = Expression::Value(EvalResult::Float(1));
/// let rhs = Expression::Value(EvalResult::Float(2));
/// assert_eq!(Expression::Add(lhs, rhs), EvalResult::Float(3));
/// ```
#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    /// The most fundamental expression, a value, denoted by a double quote (`"`)
    /// followed by a literal value (either a float, or a boolean).
    /// This would simply evaluate to itself.
    Value(EvalResult),

    /// A variable, denoted by a colon (`:`), followed by a name. This would evaluate to itself.
    Variable(EvalResult),

    /// Query the value for a variable name
    GetVariable(Box<Expression>),

    /// Add two expressions together
    Add(Box<Expression>, Box<Expression>),

    /// Subtract one expression from another
    /// Note that `Subtract(a, b)` is interpreted as `a - b`
    Subtract(Box<Expression>, Box<Expression>),

    /// Multiply two expressions together
    Multiply(Box<Expression>, Box<Expression>),

    /// Divide one expression by another
    /// Note that `Divide(a, b)` is interpreted as `a / b`
    Divide(Box<Expression>, Box<Expression>),

    /// Check if two expressions have equivalent values
    Equals(Box<Expression>, Box<Expression>),

    /// Check if two expressions do not have equivalent values
    NotEquals(Box<Expression>, Box<Expression>),

    /// Check if one expression is strictly greater than the other
    /// Note that `GreaterThan(a, b)` is interpreted as `a > b`.
    GreaterThan(Box<Expression>, Box<Expression>),

    /// Check if one expression is strictly less than the other.
    /// Note that `LessThan(a, b)` is interpreted as `a < b`.
    LessThan(Box<Expression>, Box<Expression>),

    /// Return true if both expressions evaluates to true.
    And(Box<Expression>, Box<Expression>),

    /// Returns true if at least one of the expressions evaluates to true.
    Or(Box<Expression>, Box<Expression>),

    /// Returns the turtle's x-coordinates
    XCor,

    /// Returns the turtle's y-coordinates
    YCor,

    /// Returns the turtle's heading
    Heading,

    /// Returns the pen colour
    Colour,
}

impl Expression {
    /// Evaluates this expression. When successful, returns an instance of EvalResult (either a boolean or f32).
    pub fn eval(&self, context: &Program) -> Result<EvalResult, InterpreterError> {
        match self {
            Expression::Value(value) => Ok(value.clone()),
            Expression::Variable(name) => Ok(name.clone()),
            Expression::GetVariable(key) => {
                let variable_name: String = match key.eval(context) {
                    Ok(name) => match name {
                        EvalResult::Bool(_) => {
                            return Err(InterpreterError::invalid_type("variable name", "boolean"))
                        }
                        EvalResult::Float(_) => {
                            return Err(InterpreterError::invalid_type("variable name", "float"))
                        }
                        EvalResult::String(name) => name,
                    },
                    Err(_) => todo!("Make error for unsuccessful evaluation"),
                };

                match context.variables.get(&variable_name) {
                    Some(val) => Ok(val.clone()),
                    None => Err(InterpreterError::UndefinedVariable(variable_name)),
                }
            }
            Expression::Add(lhs, rhs) => {
                arithmetic_operation!(add, lhs, rhs, context, "addition of booleans")
            }
            Expression::Subtract(lhs, rhs) => {
                arithmetic_operation!(sub, lhs, rhs, context, "subtraction of booleans")
            }
            Expression::Multiply(lhs, rhs) => {
                arithmetic_operation!(mul, lhs, rhs, context, "multiplication of booleans")
            }

            // `Divide`, in particular, needs additional error checking
            Expression::Divide(lhs, rhs) => {
                let divisor = rhs.eval(context)?;
                match divisor {
                    // Additional error checking to prevent divdide by zero errors.
                    EvalResult::Float(val) if val == (0 as f32) => {
                        Err(InterpreterError::division_by_zero())
                    }
                    _ => arithmetic_operation!(div, lhs, rhs, context, "division of booleans"),
                }
            }
            Expression::Equals(lhs, rhs) => comparison!(eq, lhs, rhs, context),
            Expression::NotEquals(lhs, rhs) => comparison!(ne, lhs, rhs, context),
            Expression::GreaterThan(lhs, rhs) => comparison!(gt, lhs, rhs, context),
            Expression::LessThan(lhs, rhs) => comparison!(lt, lhs, rhs, context),
            Expression::And(lhs, rhs) => logical_operation!(&&, lhs, rhs, context),
            Expression::Or(lhs, rhs) => logical_operation!(||, lhs, rhs, context),
            Expression::XCor => {
                let (xcor, _) = context.turtle.get_turtle_coords();
                Ok(EvalResult::Float(xcor))
            }
            Expression::YCor => {
                let (_, ycor) = context.turtle.get_turtle_coords();
                Ok(EvalResult::Float(ycor))
            }
            Expression::Heading => Ok(EvalResult::Float(context.turtle.get_heading())),
            Expression::Colour => Ok(EvalResult::Float(context.turtle.get_pen_colour())),
        }
    }
}

/// This is a list of executable commands for the logo language. They may take in strings, Expressions, or vectors of Commands as argument
#[derive(Debug)]
pub(crate) enum Command {
    Comment,
    /// Command to set the pen state to up.
    PenUp,

    /// Command to set the pen state to down.
    PenDown,

    /// Command to move the pen forward by a certain distance.
    Forward(Expression),

    /// Command to move the pen backward by a certain distance.
    Back(Expression),

    /// Command to move a pen left by a certain distance.
    Left(Expression),

    /// Command to move a pen right by a certain distance.
    Right(Expression),

    /// Command to change the pen colour to a certain value.
    SetPenColor(Expression),

    /// Command to turn the pen by a certain number of degrees.
    Turn(Expression),

    /// Command to set the pen's angle to a specific value, in degrees.
    SetHeading(Expression),

    /// Command to set the X-axis position of the pen to a specific value.
    SetX(Expression),

    /// Command to set the Y-axis position of the pen to a specific value.
    SetY(Expression),

    /// Command to create a new variable.
    MakeVariable(String, Expression),

    /// Command to increment the value of an existing variable by a certain number. Will not work if the variable does not exist yet.
    SetVariable(String, Expression),

    /// Command to execute a set of commands only if an expression evaluates to true
    If(Expression, Vec<Command>),

    /// Command to repeatedly execute a set of command as long as an expression evaluates to true
    While(Expression, Vec<Command>),

    /// A set of commands (`Vec<Command>`) with its own parameters (`HashMap<String, Expression>`).
    /// The variables are global. The hashmap will be merged with the global variables hashmap
    Procedure(HashMap<String, Expression>, Vec<Command>),
}

impl Command {
    /// Run the command token
    fn execute(&self, context: &mut Program) -> Result<(), Box<dyn Error>> {
        match self {
            Command::Comment => Ok(()),
            // Pen state manipulation
            Command::PenUp => match context.turtle.set_pen_state(crate::turtle::PenState::Up) {
                crate::turtle::PenState::Up => Ok(()),
                crate::turtle::PenState::Down => Err(Box::new(
                    InterpreterError::unsuccessful_operation("setting the pen state to up"),
                )),
            },
            Command::PenDown => match context.turtle.set_pen_state(crate::turtle::PenState::Down) {
                crate::turtle::PenState::Down => Ok(()),
                crate::turtle::PenState::Up => Err(Box::new(
                    InterpreterError::unsuccessful_operation("setting the pen state to down"),
                )),
            },
            Command::SetPenColor(colour) => match colour.eval(context)? {
                EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                    "pen colour",
                    "boolean",
                ))),
                EvalResult::Float(val) => {
                    context.turtle.set_pen_colour(val)?;
                    Ok(())
                }
                EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                    "pen colour",
                    "string",
                ))),
            },

            // Turtle movement
            Command::Forward(distance) => {
                let value: EvalResult = distance.eval(context)?;
                match value {
                    EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "string",
                    ))),
                    EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "boolean",
                    ))),
                    EvalResult::Float(forward_distance) => {
                        let _ = context.turtle.move_turtle(None, Some(forward_distance));
                        Ok(())
                    }
                }
            }
            Command::Back(distance) => {
                let value: EvalResult = distance.eval(context)?;
                match value {
                    EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "string",
                    ))),
                    EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "boolean",
                    ))),
                    EvalResult::Float(backward_distance) => {
                        let _ = context.turtle.move_turtle(None, Some(-backward_distance));
                        Ok(())
                    }
                }
            }
            Command::Left(distance) => {
                let value: EvalResult = distance.eval(context)?;
                match value {
                    EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "string",
                    ))),
                    EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "boolean",
                    ))),
                    EvalResult::Float(leftward_distance) => {
                        let _ = context.turtle.move_turtle(Some(-leftward_distance), None);
                        Ok(())
                    }
                }
            }
            Command::Right(distance) => {
                let value: EvalResult = distance.eval(context)?;
                match value {
                    EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "string",
                    ))),
                    EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                        "distance", "boolean",
                    ))),
                    EvalResult::Float(rightward_distance) => {
                        let _ = context.turtle.move_turtle(Some(rightward_distance), None);
                        Ok(())
                    }
                }
            }

            // Turtle state manipulation
            Command::Turn(angle) => match angle.eval(context)? {
                EvalResult::Bool(_) => {
                    Err(Box::new(InterpreterError::invalid_type("angle", "bool")))
                }
                EvalResult::Float(val) => {
                    context.turtle.turn(val)?;
                    Ok(())
                }
                EvalResult::String(_) => {
                    Err(Box::new(InterpreterError::invalid_type("angle", "string")))
                }
            },
            Command::SetHeading(angle) => match angle.eval(context)? {
                EvalResult::Bool(_) => {
                    Err(Box::new(InterpreterError::invalid_type("angle", "bool")))
                }
                EvalResult::Float(val) => {
                    context.turtle.set_heading(val)?;
                    Ok(())
                }
                EvalResult::String(_) => {
                    Err(Box::new(InterpreterError::invalid_type("angle", "string")))
                }
            },
            Command::SetX(x) => match x.eval(context)? {
                EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                    "coordinate",
                    "bool",
                ))),
                EvalResult::Float(val) => {
                    context.turtle.set_coordinates(Some(val), None)?;
                    Ok(())
                }
                EvalResult::String(_) => {
                    Err(Box::new(InterpreterError::invalid_type("bool", "string")))
                }
            },
            Command::SetY(y) => match y.eval(context)? {
                EvalResult::Bool(_) => Err(Box::new(InterpreterError::invalid_type(
                    "coordinate",
                    "bool",
                ))),
                EvalResult::Float(val) => {
                    context.turtle.set_coordinates(None, Some(val))?;
                    Ok(())
                }
                EvalResult::String(_) => {
                    Err(Box::new(InterpreterError::invalid_type("bool", "string")))
                }
            },

            // Variable manipulation
            Command::MakeVariable(name, value) => {
                context
                    .variables
                    .insert(name.to_owned(), value.eval(context)?);
                Ok(())
            }
            Command::SetVariable(name, value) => match context.variables.contains_key(name) {
                true => {
                    context
                        .variables
                        .insert(name.to_owned(), value.eval(context)?);
                    Ok(())
                }
                false => Err(Box::new(InterpreterError::undefined_var(name))),
            },

            // Control flow
            Command::If(expression, commands) => match expression.eval(context)? {
                EvalResult::Bool(condition) => {
                    if condition {
                        // Iteratively execute each command and filter for errors
                        let errors: Vec<Box<dyn Error>> = commands
                            .iter()
                            .map(|x: &Command| -> Result<(), Box<dyn Error>> { x.execute(context) })
                            .filter(|x: &Result<(), Box<dyn Error>>| x.is_err())
                            .map(|x: Result<(), Box<dyn Error>>| -> Box<dyn Error> {
                                x.expect_err("We filtered for errors")
                            })
                            .collect();

                        // If there are errors, we return an error
                        match errors.is_empty() {
                            false => Err(Box::new(InterpreterError::unsuccessful_operation(
                                "conditional statement",
                            ))),

                            // If there are no errors, we're all good
                            true => Ok(()),
                        }
                    } else {
                        Ok(())
                    }
                }

                // Invalid types
                EvalResult::Float(_) => Err(Box::new(InterpreterError::invalid_type(
                    "condition",
                    "float",
                ))),
                EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                    "condition",
                    "string",
                ))),
            },
            Command::While(expression, commands) => match expression.eval(context)? {
                EvalResult::Bool(condition) => {
                    let mut mutable_condition: bool = condition;
                    while mutable_condition {
                        // Update the mutable condition at the start of the loop
                        mutable_condition = match expression.eval(context)? {
                            EvalResult::Bool(val) => val,

                            // Invalid types
                            EvalResult::Float(_) => {
                                return Err(Box::new(InterpreterError::invalid_type(
                                    "condition",
                                    "float",
                                )))
                            }
                            EvalResult::String(_) => {
                                return Err(Box::new(InterpreterError::invalid_type(
                                    "condition",
                                    "string",
                                )))
                            }
                        };

                        // Iteratively execute each command and filter for errors
                        let errors: Vec<Result<(), Box<dyn Error>>> = commands
                            .iter()
                            .map(|x: &Command| -> Result<(), Box<dyn Error>> { x.execute(context) })
                            .filter(|x: &Result<(), Box<dyn Error>>| x.is_err())
                            .collect();

                        // If there are errors, we return an error
                        match errors.is_empty() {
                            false => {
                                return Err(Box::new(InterpreterError::unsuccessful_operation(
                                    "conditional statement",
                                )));
                            }

                            // If there are no errors, we're all good
                            true => (),
                        }
                    }
                    Ok(())
                }
                EvalResult::Float(_) => Err(Box::new(InterpreterError::invalid_type(
                    "condition",
                    "float",
                ))),
                EvalResult::String(_) => Err(Box::new(InterpreterError::invalid_type(
                    "condition",
                    "string",
                ))),
            },

            Command::Procedure(parameters, commands) => {
                /*
                 * Since procedure parameters must be global variables as well, we need to merge the `parameters`
                 * hash map with the `context.variables` hash map. However, `context.variables` is of type
                 * `HashMap<String, EvalResult>`, whereas `parameters` is of type `HashMap<String, Expression>`.
                 * Therefore, we need to evaluate these expressions before merging the hashmaps.
                 */
                let evaluated_parameters: HashMap<String, EvalResult> = parameters
                    .iter()
                    .try_fold(HashMap::new(), |mut acc, (key, val)| -> Result<HashMap<String, EvalResult>, Box<InterpreterError>> {
                        match val.eval(context) {
                            Ok(res) => {
                                acc.insert(key.to_owned(), res);
                                Ok(acc)
                            }
                            Err(e) => Err(Box::new(e)),
                        }
                    })?;

                // Now that the expressions are evaluated, we can merge it with `context.variables`
                context.variables.extend(evaluated_parameters);

                // Iterate through the vector of commands and collect any execution errors
                let errors: Vec<Result<(), Box<dyn Error>>> = commands
                    .iter()
                    .map(|x: &Command| -> Result<(), Box<dyn Error>> { x.execute(context) })
                    .filter(|x: &Result<(), Box<dyn Error>>| x.is_err())
                    .collect();

                match errors.is_empty() {
                    // Procedure terminated successfully
                    true => Ok(()),

                    // Procedure failed
                    false => Err(Box::new(InterpreterError::unsuccessful_operation(
                        "procedure",
                    ))),
                }
            }
        }
    }
}

/// The parsed logo program.
pub struct Program {
    /// List of commands contained in the program. This will be iterated through and executed.
    pub(crate) commands: Vec<Command>,

    /// List of variables defined in the program.
    variables: HashMap<String, EvalResult>,

    /// The turtle itself
    turtle: Turtle,
}

impl Program {
    /// Create a new program, with an empty `commands` vector and `variables` hash map.
    pub fn new(commands: Vec<Command>) -> Self {
        Program {
            commands,
            variables: HashMap::new(),
            turtle: Turtle::new(),
        }
    }

    /// Execute the program by iterating through the `commands` vector and executing them.
    /// Returns a vector of errors
    pub fn execute(&mut self) -> Vec<Box<dyn Error>> {
        // We can take the command vector as they're not going to be used again after this
        let commands: Vec<Command> = std::mem::take(&mut self.commands);
        let mut result: Vec<Result<(), Box<dyn Error>>> = Vec::new();
        commands.into_iter().for_each(|command: Command| {
            let curr_result = command.execute(self);
            result.push(curr_result);
        });

        // return only the errors
        let errors: Vec<Box<dyn Error>> = result
            .into_iter()
            .filter(|x| x.is_err())
            .map(|x| x.unwrap_err())
            .collect();

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn valid_add() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));
        assert_eq!(
            Expression::Add(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            Expression::Value(EvalResult::Float(3_f32))
                .eval(&context)
                .unwrap(),
        );
    }
    #[test]
    fn valid_sub() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));
        assert_eq!(
            Expression::Subtract(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            Expression::Value(EvalResult::Float(-1_f32))
                .eval(&context)
                .unwrap(),
        );
    }
    #[test]
    fn valid_multiply() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));
        assert_eq!(
            Expression::Multiply(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            Expression::Value(EvalResult::Float(2_f32))
                .eval(&context)
                .unwrap(),
        );
    }
    #[test]
    fn valid_divide() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));
        assert_eq!(
            Expression::Divide(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            Expression::Value(EvalResult::Float(0.5_f32))
                .eval(&context)
                .unwrap(),
        );
    }
    #[test]
    fn invalid_divide_by_zero() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(0_f32));

        assert_eq!(
            Expression::Divide(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::division_by_zero())
        )
    }

    #[test]
    fn invalid_arithmetic_on_bool() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::Add(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "addition of booleans"
            ))
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));
        assert_eq!(
            Expression::Subtract(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "subtraction of booleans"
            ))
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));
        assert_eq!(
            Expression::Multiply(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "multiplication of booleans"
            ))
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));
        assert_eq!(
            Expression::Divide(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "division of booleans"
            ))
        );
    }

    #[test]
    fn valid_and() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::And(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );

        let lhs: Expression = Expression::Value(EvalResult::Bool(false));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::And(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(true));

        assert_eq!(
            Expression::And(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }
    #[test]
    fn valid_or() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::Or(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );

        let lhs: Expression = Expression::Value(EvalResult::Bool(false));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::Or(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(true));

        assert_eq!(
            Expression::Or(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }

    #[test]
    fn invalid_logic_on_float() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::And(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "logical operation of non-booleans"
            ))
        );
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::Or(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "logical operation of non-booleans"
            ))
        );
    }

    #[test]
    fn valid_greater_than_float() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::GreaterThan(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::GreaterThan(Box::new(rhs), Box::new(lhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }
    #[test]
    fn valid_greater_than_bool() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::GreaterThan(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::GreaterThan(Box::new(rhs), Box::new(lhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
    }
    #[test]
    fn valid_less_than_float() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::LessThan(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::LessThan(Box::new(rhs), Box::new(lhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
    }
    #[test]
    fn valid_less_than_bool() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::LessThan(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::LessThan(Box::new(rhs), Box::new(lhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }

    #[test]
    fn valid_equals_float() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(1_f32));

        assert_eq!(
            Expression::Equals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::Equals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
    }
    #[test]
    fn valid_not_equals_float() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(1_f32));

        assert_eq!(
            Expression::NotEquals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );

        let lhs: Expression = Expression::Value(EvalResult::Float(1_f32));
        let rhs: Expression = Expression::Value(EvalResult::Float(2_f32));

        assert_eq!(
            Expression::NotEquals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }

    #[test]
    fn valid_not_equals_bool() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(true));

        assert_eq!(
            Expression::NotEquals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::NotEquals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }
    #[test]
    fn valid_equals_bool() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(true));

        assert_eq!(
            Expression::Equals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Bool(false));

        assert_eq!(
            Expression::Equals(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(false)
        );
    }

    #[test]
    fn invalid_equals() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Float(1_f32));

        assert_eq!(
            Expression::Equals(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "comparison of different types"
            ))
        )
    }

    #[test]
    fn invalid_not_equals() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Value(EvalResult::Bool(true));
        let rhs: Expression = Expression::Value(EvalResult::Float(1_f32));

        assert_eq!(
            Expression::NotEquals(Box::new(lhs), Box::new(rhs)).eval(&context),
            Err(InterpreterError::unsupported_operation(
                "comparison of different types"
            ))
        )
    }

    #[test]
    fn integration() {
        // Dummy program to satisfy parameter
        let context: Program = Program::new(Vec::new());

        let lhs: Expression = Expression::Divide(
            Box::new(Expression::Add(
                Box::new(Expression::Value(EvalResult::Float(2_f32))),
                Box::new(Expression::Value(EvalResult::Float(2_f32))),
            )),
            Box::new(Expression::Value(EvalResult::Float(2_f32))),
        ); // should evaluate to 2
        let rhs: Expression = Expression::Multiply(
            Box::new(Expression::Subtract(
                Box::new(Expression::Value(EvalResult::Float(5_f32))),
                Box::new(Expression::Value(EvalResult::Float(2_f32))),
            )),
            Box::new(Expression::Add(
                Box::new(Expression::Value(EvalResult::Float(2_f32))),
                Box::new(Expression::Value(EvalResult::Float(2_f32))),
            )),
        ); // should evaluate to 12

        assert_eq!(lhs.eval(&context).unwrap(), EvalResult::Float(2_f32));
        assert_eq!(rhs.eval(&context).unwrap(), EvalResult::Float(12_f32));
        assert_eq!(
            Expression::LessThan(Box::new(lhs), Box::new(rhs))
                .eval(&context)
                .unwrap(),
            EvalResult::Bool(true)
        );
    }

    #[test]
    fn penup_command() {
        let mut program = Program::new(Vec::new());
        program.turtle.set_pen_state(crate::turtle::PenState::Down);
        program.commands.push(Command::PenUp);
        program.execute();
        assert_eq!(program.turtle.get_pen_state(), &crate::turtle::PenState::Up);
    }

    #[test]
    fn pendown_command() {
        let mut program = Program::new(Vec::new());
        program.commands.push(Command::PenDown);
        program.execute();
        assert_eq!(
            program.turtle.get_pen_state(),
            &crate::turtle::PenState::Down
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100000))]
        // This doesn't seem to work because of weird floating point stuff
        // #[test]
        // fn add_floats_correctly(lhs in proptest::num::f32::NORMAL, rhs in proptest::num::f32::NORMAL) {
        //     let context = Program::new(Vec::new()); // Assuming this creates a suitable context for evaluation
        //     let lhs_expr = Expression::Value(EvalResult::Float(lhs));
        //     let rhs_expr = Expression::Value(EvalResult::Float(rhs));

        //     let add_expr = Expression::Add(Box::new(lhs_expr), Box::new(rhs_expr));

        //     // Evaluate the addition expression
        //     match add_expr.eval(&context) {
        //         Ok(EvalResult::Float(result)) => {
        //             // Assert the property: The result should be approximately equal to the sum of lhs and rhs
        //             prop_assert!((result - (lhs + rhs)).abs() < f32::EPSILON.abs());
        //         },
        //         _ => prop_assert!(false, "Expected Float result from addition"),
        //     }
        // }

        // This doesn't seem to work because of weird floating point stuff
        // #[test]
        // fn move_turtle_correctly(movements in proptest::collection::vec((proptest::num::f32::NORMAL, proptest::num::f32::NORMAL), 0..1000)) {
        //     let (x_incr, y_incr) = movements
        //         .par_iter()
        //         .fold(|| (0.0, 0.0), |acc, &x| (acc.0 + x.0, acc.1 + x.1))
        //         .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));

        //     let commands: Vec<Command> = movements
        //         .par_iter()
        //         .flat_map(|(x, y)| {
        //             let mut cmds: Vec<Command> = Vec::new();
        //             // Handle the x movement
        //             if *x > 0.0 {
        //                 cmds.push(Command::Right(Expression::Value(EvalResult::Float((*x).abs()))));
        //             } else if *x < 0.0 {
        //                 cmds.push(Command::Left(Expression::Value(EvalResult::Float((*x).abs()))));
        //             }

        //             // Handle the y movement
        //             if *y > 0.0 {
        //                 cmds.push(Command::Forward(Expression::Value(EvalResult::Float((*y).abs()))));
        //             } else if *y < 0.0 {
        //                 cmds.push(Command::Back(Expression::Value(EvalResult::Float((*y).abs()))));
        //             }

        //             cmds.into_par_iter()
        //         })
        //         .collect();

        //     let mut program = Program::new(commands);
        //     let (start_x, start_y) = program.turtle.get_turtle_coords();
        //     let errors = program.execute();

        //     prop_assert!(errors.is_empty());
        //     let (end_x, end_y) = program.turtle.get_turtle_coords();

        //     prop_assert!(( end_x - (start_x + x_incr) ).abs() < f32::EPSILON.abs());
        //     prop_assert!(( end_y - (start_y + y_incr) ).abs() < f32::EPSILON.abs());
        // }

        #[test]
        fn set_colour_correctly(colour in any::<f32>()) {
           let mut program = Program::new(vec![Command::SetPenColor(Expression::Value(EvalResult::Float(colour)))]);
            let errors = program.execute();
            match colour {
                // Valid colour range
                colour if (0.0..=15.0).contains(&colour) => {
                    assert!(errors.is_empty());
                    assert_eq!(program.turtle.get_pen_colour(), colour);
                },

                // Everything else is invalid
                _ => {
                    assert!(!errors.is_empty());
                    assert_eq!(program.turtle.get_pen_colour(), 0.0);
                },
            };
        }

        #[test]
        fn turn_turtle_correctly(angles in any::<Vec<f32>>()) {
            let mut commands: Vec<Command> = Vec::new();

            let mut num_expected_failures: usize = 0;
            let mut expected_change_in_angle: f32 = 0.0;
            for angle in angles {
                if (0.0..=360.0).contains(&angle) {
                    expected_change_in_angle += angle;
                } else {
                    num_expected_failures += 1;
                }
                commands.push(Command::Turn(Expression::Value(EvalResult::Float(angle))));
            }

            let mut program: Program = Program::new(commands);
            let errors = program.execute();

            assert_eq!(errors.len(), num_expected_failures);
            assert_eq!(program.turtle.get_heading(), expected_change_in_angle);
        }
    }
}
