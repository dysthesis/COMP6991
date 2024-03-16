use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, space1},
    multi::many0,
    sequence::{delimited, separated_pair},
    IResult,
};

#[derive(PartialEq, Debug)]
enum Token {
    PenUp,
    PenDown,
    Forward(i32),
    Back(i32),
    Left(i32),
    Right(i32),
}

/// # Parse pen state commands
///
/// This function is responsible for parsing any commands related to
/// modifying the pen's state, including `PENUP` and `PENDOWN`. It returns
/// an instance of `Ok((&str, Token::PenUp))` or `Ok((&str, Token::PenDown))`
/// upon successful parsing, or `Error<&str>` otherwise.
///
/// Example:
/// ```
/// assert_eq!(parse_pen_state("PENUP"), Ok(("", Token::PenUp)));
/// assert_eq!(parse_pen_state("PENDOWN"), Ok(("", Token::PenDown)));
/// ```
fn parse_pen_state(input: &str) -> IResult<&str, Token> {
    let (input, parsed): (&str, &str) = delimited(
        multispace0,
        alt((tag("PENUP"), tag("PENDOWN"))),
        multispace0,
    )(input)?;

    let result = match parsed {
        "PENUP" => Token::PenUp,

        "PENDOWN" => Token::PenDown,

        // TODO: Verify that this is correct
        _ => unreachable!(),
    };

    Ok((input, result))
}

fn parse_directions(input: &str) -> IResult<&str, Token> {
    let (input, (direction, distance)) = delimited(
        // There may or may not be any whitespace before the pattern
        multispace0,
        separated_pair(
            // Recognise any of these strings as a direction
            alt((tag("FORWARD"), tag("BACK"), tag("LEFT"), tag("RIGHT"))),
            // The direction and distance must be separated with a space
            space1,
            // Ensure that there is at least one digit for the distance
            nom::character::complete::i32,
        ),
        // There may or may not be whitespace after the patern
        multispace0,
    )(input)?;

    let result = match direction {
        "FORWARD" => Token::Forward(distance),
        "BACK" => Token::Back(distance),
        "LEFT" => Token::Left(distance),
        "RIGHT" => Token::Right(distance),

        // TODO: Verify that this is correct
        _ => unreachable!(),
    };

    Ok((input, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_states() {
        let penup = "PENUP";
        assert_eq!(parse_pen_state(penup), Ok(("", Token::PenUp)));

        let pendown = "PENDOWN";
        assert_eq!(parse_pen_state(pendown), Ok(("", Token::PenDown)));
    }

    #[test]
    fn valid_states_with_whitespace() {
        let this_is_correct = " PENUP ";
        assert_eq!(parse_pen_state(this_is_correct), Ok(("", Token::PenUp)));

        let this_is_also_correct = "\nPENUP\n";
        assert_eq!(
            parse_pen_state(this_is_also_correct),
            Ok(("", Token::PenUp))
        );
    }

    #[test]
    fn invalid_states() {
        let invalid = "PENSIDEWAYS";
        assert!(parse_pen_state(invalid).is_err());
    }

    #[test]
    fn valid_directions() {
        let forward = "FORWARD 10";
        assert_eq!(parse_directions(forward), Ok(("", Token::Forward(10))));

        let back = "BACK 10";
        assert_eq!(parse_directions(back), Ok(("", Token::Back(10))));

        let left = "LEFT 10";
        assert_eq!(parse_directions(left), Ok(("", Token::Left(10))));

        let right = "RIGHT 10";
        assert_eq!(parse_directions(right), Ok(("", Token::Right(10))));
    }

    #[test]
    fn valid_directions_with_whitespace() {
        let this_is_correct = " BACK 5 ";
        assert_eq!(parse_directions(this_is_correct), Ok(("", Token::Back(5))));

        let this_is_also_correct = "\nRIGHT 20\n";
        assert_eq!(
            parse_directions(this_is_also_correct),
            Ok(("", Token::Right(20)))
        );
    }

    #[test]
    fn invalid_directions() {
        let invalid = "NOWHERE 10";
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn invalid_directions_no_distance() {
        let invalid = "FORWARD";
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn invalid_directions_no_whitespace() {
        let invalid = "BACKWARD4";
        assert!(parse_directions(invalid).is_err());
    }
}
