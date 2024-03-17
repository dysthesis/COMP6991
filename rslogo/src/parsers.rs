use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, space1},
    combinator::map,
    error::context,
    multi::many0,
    sequence::{delimited, separated_pair},
    IResult,
};

#[derive(PartialEq, Debug)]
pub enum Token {
    PenUp,
    PenDown,
    Forward(i32),
    Back(i32),
    Left(i32),
    Right(i32),
}
use miette::{Context, Diagnostic, IntoDiagnostic, NamedSource, Result, SourceSpan};
use nom_supreme::{error::ErrorTree, tag::complete::tag as tag_supreme};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Cannot parse!")]
#[diagnostic(code(parser::parse_error), help("Placeholder help text"))]
struct MyParseError {
    #[source_code]
    src: NamedSource<String>,
    #[label("Caused by")]
    cause: SourceSpan,
}

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
fn parse_pen_state(input: &str) -> IResult<&str, Token, ErrorTree<&str>> {
    context(
        "When parsing pen state commands",
        delimited(
            multispace0,
            alt((
                map(tag_supreme("PENUP"), |_| Token::PenUp),
                map(tag_supreme("PENDOWN"), |_| Token::PenDown),
            )),
            multispace0,
        ),
    )(input)
}

fn parse_directions(input: &str) -> IResult<&str, Token, ErrorTree<&str>> {
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

        // TODO:  THIS SHOULD RETURN AN ERROR OBJECT!
        _ => unreachable!(),
    };

    Ok((input, result))
}

fn parse_one(input: &str) -> IResult<&str, Token, ErrorTree<&str>> {
    // Put all of the parsers inside here.
    let (remainder, result) = alt((parse_directions, parse_pen_state))(input)?;

    Ok((remainder, result))
}

pub fn parse(input: &str) -> Result<(&str, Vec<Token>)> {
    // TODO: This should probably be all_consuming!
    Ok(many0(parse_one)(input)
        .into_diagnostic()
        .wrap_err("Parsing failed.")?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_states() {
        let penup = "PENUP";
        let (_, result) = parse_pen_state(penup).unwrap();
        assert_eq!(result, Token::PenUp);

        let pendown = "PENDOWN";
        let (_, result) = parse_pen_state(pendown).unwrap();
        assert_eq!(result, Token::PenDown);
    }

    #[test]
    fn valid_states_with_whitespace() {
        let this_is_correct = " PENUP ";
        let (_, result) = parse_pen_state(this_is_correct).unwrap();
        assert_eq!(result, Token::PenUp);

        let this_is_also_correct = "\nPENUP\n";
        let (_, result) = parse_pen_state(this_is_also_correct).unwrap();
        assert_eq!(result, Token::PenUp);
    }

    #[test]
    fn invalid_states() {
        let invalid = "PENSIDEWAYS";
        assert!(parse_pen_state(invalid).is_err());
    }

    #[test]
    fn valid_directions() {
        let forward = "FORWARD 10";
        let (_, result) = parse_directions(forward).unwrap();
        assert_eq!(result, Token::Forward(10));

        let back = "BACK 10";
        let (_, result) = parse_directions(back).unwrap();
        assert_eq!(result, Token::Back(10));

        let left = "LEFT 10";
        let (_, result) = parse_directions(left).unwrap();
        assert_eq!(result, Token::Left(10));

        let right = "RIGHT 10";
        let (_, result) = parse_directions(right).unwrap();
        assert_eq!(result, Token::Right(10));
    }

    #[test]
    fn valid_directions_with_whitespace() {
        let this_is_correct = " BACK 5 ";
        let (_, result) = parse_directions(this_is_correct).unwrap();
        assert_eq!(result, Token::Back(5));

        let this_is_also_correct = "\nRIGHT 20\n";
        let (_, result) = parse_directions(this_is_also_correct).unwrap();
        assert_eq!(result, Token::Right(20));
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
        let invalid = "BACK4";
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn valid_parse_one() {
        let input = "PENUP\nFORWARD 4\nPENDOWN";
        let (remainder, penup) = parse_one(input).unwrap();
        assert_eq!(penup, Token::PenUp);

        let (remainder, forward) = parse_one(remainder).unwrap();
        assert_eq!(forward, Token::Forward(4));

        let (remainder, pendown) = parse_one(remainder).unwrap();
        assert_eq!(pendown, Token::PenDown);
        assert_eq!(remainder, "");
    }

    #[test]
    fn valid_parse() {
        let input = "PENUP\nBACK 16\nRIGHT 10\nPENDOWN\nextra";
        let (_, result) = match parse(input) {
            Ok(result) => result,
            Err(e) => panic!("This shouldn't fail! Error: {:?}", e),
        };
        assert_eq!(
            result,
            vec![
                Token::PenUp,
                Token::Back(16),
                Token::Right(10),
                Token::PenDown
            ]
        );
    }
}
