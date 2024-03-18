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

pub type Span<'a> = LocatedSpan<&'a str>;

use nom_locate::LocatedSpan;
use nom_supreme::{error::ErrorTree, tag::complete::tag as tag_supreme};

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
fn parse_pen_state(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
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

fn parse_directions(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
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

    let result = match direction.into_fragment() {
        "FORWARD" => Token::Forward(distance),
        "BACK" => Token::Back(distance),
        "LEFT" => Token::Left(distance),
        "RIGHT" => Token::Right(distance),

        // TODO:  THIS SHOULD RETURN AN ERROR OBJECT!
        _ => unreachable!(),
    };

    Ok((input, result))
}

fn parse_one(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
    // Put all of the parsers inside here.
    let (remainder, result) = alt((parse_directions, parse_pen_state))(input)?;

    Ok((remainder, result))
}

pub fn parse(input: Span) -> IResult<Span, Vec<Token>, ErrorTree<Span>> {
    // TODO: This should probably be all_consuming!
    Ok(many0(parse_one)(input)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_states() {
        let penup = Span::new("PENUP");
        let (_, result) = parse_pen_state(penup).unwrap();
        assert_eq!(result, Token::PenUp);

        let pendown = Span::new("PENDOWN");
        let (_, result) = parse_pen_state(pendown).unwrap();
        assert_eq!(result, Token::PenDown);
    }

    #[test]
    fn valid_states_with_whitespace() {
        let this_is_correct = Span::new(" PENUP ");
        let (_, result) = parse_pen_state(this_is_correct).unwrap();
        assert_eq!(result, Token::PenUp);

        let this_is_also_correct = Span::new("\nPENUP\n");
        let (_, result) = parse_pen_state(this_is_also_correct).unwrap();
        assert_eq!(result, Token::PenUp);
    }

    #[test]
    fn invalid_states() {
        let invalid = Span::new("PENSIDEWAYS");
        assert!(parse_pen_state(invalid).is_err());
    }

    #[test]
    fn valid_directions() {
        let forward = Span::new("FORWARD 10");
        let (_, result) = parse_directions(forward).unwrap();
        assert_eq!(result, Token::Forward(10));

        let back = Span::new("BACK 10");
        let (_, result) = parse_directions(back).unwrap();
        assert_eq!(result, Token::Back(10));

        let left = Span::new("LEFT 10");
        let (_, result) = parse_directions(left).unwrap();
        assert_eq!(result, Token::Left(10));

        let right = Span::new("RIGHT 10");
        let (_, result) = parse_directions(right).unwrap();
        assert_eq!(result, Token::Right(10));
    }

    #[test]
    fn valid_directions_with_whitespace() {
        let this_is_correct = Span::new(" BACK 5 ");
        let (_, result) = parse_directions(this_is_correct).unwrap();
        assert_eq!(result, Token::Back(5));

        let this_is_also_correct = Span::new("\nRIGHT 20\n");
        let (_, result) = parse_directions(this_is_also_correct).unwrap();
        assert_eq!(result, Token::Right(20));
    }

    #[test]
    fn invalid_directions() {
        let invalid = Span::new("NOWHERE 10");
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn invalid_directions_no_distance() {
        let invalid = Span::new("FORWARD");
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn invalid_directions_no_whitespace() {
        let invalid = Span::new("BACK4");
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn valid_parse_one() {
        let input = Span::new("PENUP\nFORWARD 4\nPENDOWN");
        let (remainder, penup) = parse_one(input).unwrap();
        assert_eq!(penup, Token::PenUp);

        let (remainder, forward) = parse_one(remainder).unwrap();
        assert_eq!(forward, Token::Forward(4));

        let (remainder, pendown) = parse_one(remainder).unwrap();
        assert_eq!(pendown, Token::PenDown);
        assert_eq!(remainder.into_fragment(), "");
    }

    #[test]
    fn valid_parse() {
        let input = Span::new("PENUP\nBACK 16\nRIGHT 10\nPENDOWN\nextra");
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
