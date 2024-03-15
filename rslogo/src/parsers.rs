use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace0, sequence::preceded,
    IResult,
};

/// This enum contains the list of valid Logo commands
#[derive(Debug, PartialEq)]
enum Command {
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

fn pen_up(input: &str) -> IResult<&str, Command> {
    preceded(multispace0, tag("PENUP"))(input).map(|(next_input, _)| (next_input, Command::PenUp))
}

fn pen_down(input: &str) -> IResult<&str, Command> {
    preceded(multispace0, tag("PENDOWN"))(input)
        .map(|(next_input, _)| (next_input, Command::PenDown))
}

fn command(input: &str) -> IResult<&str, Command> {
    alt((
        pen_up, pen_down,
        // Add other parsers here
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_penup() {
        let input: &str = "PENUP";
        assert_eq!(pen_up(input), Ok(("", Command::PenUp)));
    }

    #[test]
    fn valid_pendown() {
        let input: &str = "PENDOWN";
        assert_eq!(pen_down(input), Ok(("", Command::PenDown)));
    }

    #[test]
    fn invalid_penup() {
        let input: &str = "DEFINITELYNOT";
        assert!(pen_up(input).is_err());
    }

    #[test]
    fn invalid_pendown() {
        let input: &str = "DEFINITELYNOT";
        assert!(pen_down(input).is_err());
    }

    #[test]
    fn trailing_penup() {
        let input: &str = "PENUP extra";
        assert_eq!(pen_up(input), Ok((" extra", Command::PenUp)));
    }

    #[test]
    fn trailing_pendown() {
        let input: &str = "PENDOWN extra";
        assert_eq!(pen_down(input), Ok((" extra", Command::PenDown)));
    }
}
