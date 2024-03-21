use crate::errors::InterpreterError;
use std::collections::HashMap;
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

/// Macro to reduce boilerplate for logical operations
macro_rules! logical_operation {
    ($op:tt, $lhs:expr, $rhs:expr, $context:expr, $err_msg:expr) => {{
        let lhs_result: EvalResult = $lhs.eval($context)?;
        let rhs_result: EvalResult = $rhs.eval($context)?;

        match (lhs_result, rhs_result) {
            (EvalResult::Bool(lhs_val), EvalResult::Bool(rhs_val)) => {
                Ok(EvalResult::Bool(lhs_val $op rhs_val))
            }
            _ => Err(InterpreterError::unsupported_operation($err_msg)),
        }
    }};
}

#[derive(Copy, Clone)] // I think there might be a better way of doing this, but bools and f32 are small and cheap anyways
pub(crate) enum EvalResult {
    Bool(bool),
    Float(f32),
}

pub(crate) enum Expression {
    Value(EvalResult),
    GetVariable(String),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Equals(Box<Expression>, Box<Expression>),
    NotEquals(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Evaluates this expression. When successful, returns an instance of EvalResult (either a boolean or f32).
    fn eval(&self, context: &Program) -> Result<EvalResult, InterpreterError> {
        match self {
            Expression::Value(value) => Ok(*value),
            Expression::GetVariable(key) => {
                let result = context.variables.get(key);
                match result {
                    Some(value) => Ok(*value),
                    None => Err(InterpreterError::undefined_var(key.as_str())),
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
            Expression::Equals(lhs, rhs) => {
                logical_operation!(==, lhs, rhs, context, "comparison of non-booleans")
            }
            Expression::NotEquals(lhs, rhs) => {
                logical_operation!(!=, lhs, rhs, context, "comparison of non-booleans")
            }
            Expression::GreaterThan(lhs, rhs) => {
                logical_operation!(>, lhs, rhs, context, "comparison of non-booleans")
            }
            Expression::LessThan(lhs, rhs) => {
                logical_operation!(<, lhs, rhs, context, "comparison of non-booleans")
            }
            Expression::And(lhs, rhs) => {
                logical_operation!(&&, lhs, rhs, context, "logical operation of non-booleans")
            }
            Expression::Or(lhs, rhs) => {
                logical_operation!(||, lhs, rhs, context, "logical operation of non-booleans")
            }
        }
    }
}

/// This is a list of executable commands for the logo language. They may take in strings, Expressions, or vectors of Commands as argument
pub(crate) enum Command {
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
}

impl Command {
    /// Run the command token
    fn execute(&self, context: &mut Program) {
        match self {
            Command::PenUp => todo!(),
            Command::PenDown => todo!(),
            Command::Forward(_) => todo!(),
            Command::Back(_) => todo!(),
            Command::Left(_) => todo!(),
            Command::Right(_) => todo!(),
            Command::SetPenColor(_) => todo!(),
            Command::Turn(_) => todo!(),
            Command::SetHeading(_) => todo!(),
            Command::SetX(_) => todo!(),
            Command::SetY(_) => todo!(),
            Command::MakeVariable(name, value) => todo!(),
            Command::SetVariable(_, _) => todo!(),
            Command::If(_, _) => todo!(),
            Command::While(_, _) => todo!(),
        }
    }
}

/// The parsed logo program.
struct Program {
    /// List of commands contained in the program. This will be iterated through and executed.
    commands: Vec<Command>,

    /// List of variables defined in the program.
    variables: HashMap<String, EvalResult>,
}

impl Program {
    /// Create a new program, with an empty `commands` vector and `variables` hash map.
    fn new() -> Self {
        Program {
            commands: Vec::new(),
            variables: HashMap::new(),
        }
    }

    /// Execute the program by iterating through the `commands` vector and executing them.
    fn execute(&mut self) {
        // We can take the command vector as they're not going to be used again after this
        let commands = std::mem::take(&mut self.commands);
        commands.into_iter().for_each(|command: Command| {
            command.execute(self);
        });
    }
}
