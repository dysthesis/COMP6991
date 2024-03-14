use nom::{bytes::complete::tag, combinator::map, IResult};

/// This enum contains the list of valid Logo commands
#[derive(Debug, PartialEq)]
enum TurtleCommand {
    PenUp,
    PenDown,
    Forward(f32),
    Back(f32),
    Left(f32),
    Right(f32),
    SetPenColor(f32),
    Turn(f32),
    SetHeading(f32),
    SetX(f32),
    SetY(f32),
}

fn pen_up(input: &str) -> IResult<&str, TurtleCommand> {
    map(tag("PENUP"), |_| TurtleCommand::PenUp)(input)
}

fn pen_down(input: &str) -> IResult<&str, TurtleCommand> {
    map(tag("PENDOWN"), |_| TurtleCommand::PenDown)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[test]
    fn valid_penup() {
        let input: &str = "PENUP";
        assert_eq!(pen_up(input), Ok(("", TurtleCommand::PenUp)));
    }
}
