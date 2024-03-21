use std::ops::RangeInclusive;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, space1},
    combinator::map,
    error::Error,
    multi::many0,
    sequence::{preceded, separated_pair},
    Finish, IResult, Parser,
};

#[derive(PartialEq, Debug)]
pub enum Token {
    Comment,
    PenUp,
    PenDown,
    Forward(i32),
    Back(i32),
    Left(i32),
    Right(i32),
    SetPenColor(i32),
}

pub type Span<'a> = LocatedSpan<&'a str>;

use nom_locate::LocatedSpan;
use nom_supreme::{
    error::{ErrorTree, Expectation},
    tag::complete::tag as tag_supreme,
    ParserExt,
};

use crate::errors;

fn parse_comment(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
    map(preceded(tag("//"), take_until("\n")), |_| Token::Comment)
        .context("When attempting to parse a comment")
        .parse(input)
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
fn parse_pen_state(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
    /*
     * Delimited is used to get rid of whatever whitespace comes before
     * or after the command.
     */

    alt((
        map(tag_supreme("PENUP"), |_| Token::PenUp).context("When attempting to parse as PENUP"),
        map(tag_supreme("PENDOWN"), |_| Token::PenDown)
            .context("When attempting to parse as PENDOWN"),
    ))
    .context("When attempting to parse pen state commands")
    .parse(input)
}

fn parse_pen_color(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
    match separated_pair(
        tag_supreme("SETPENCOLOR"),
        multispace0,
        preceded(tag("\""), nom::character::complete::i32),
    )
    .context("When attempting to parse as SETPENCOLOR")
    .parse(input)
    {
        Ok((remainder, (_, value))) if RangeInclusive::new(0, 15).contains(&value) => {
            let err: nom::Err<ErrorTree<Span>> = nom::Err::Error(ErrorTree::Base {
                location: remainder,
                kind: nom_supreme::error::BaseErrorKind::Expected(Expectation::Digit),
            });
            Result::Err(err)
        }
        Ok((remainder, (_, value))) => Ok((remainder, Token::SetPenColor(value))),
        Err(e) => Err(e),
    }
}

fn parse_directions(input: Span) -> IResult<Span, Token, ErrorTree<Span>> {
    let (input, (direction, distance)) = separated_pair(
        // Recognise any of these strings as a direction
        alt((
            tag("FORWARD").context("When attempting to parse as FORWARD"),
            tag("BACK").context("When attempting to parse as BACK"),
            tag("LEFT").context("When attempting to parse as LEFT"),
            tag("RIGHT").context("When attempting to parse as RIGHT"),
        )),
        // The direction and distance must be separated with a space
        space1,
        // Ensure that there is at least one digit for the distance
        nom::character::complete::i32.preceded_by(tag("\"")),
    )
    // Add context in case of errors
    .context("When parsing turtle direction commands")
    .parse(input)?;

    // Match the resulting string with the corresponding token.
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
    let (remainder, result) = alt((
        parse_directions,
        parse_pen_state,
        parse_comment,
        parse_pen_color,
    ))
    // Ignore surrounding whitespace, if any.
    .delimited_by(multispace0)
    // Convert Error to Failure (unrecoverable) to get more backtrace.
    // See: https://stackoverflow.com/questions/74993188/how-to-propagate-nom-fail-context-out-of-many0
    // Note that this may be detrimental to larger parser chains, but it should be fine here: https://github.com/rust-bakery/nom/issues/1527
    // .cut() // TODO: investigate why this causes issues with parse()  for some reason
    // Add context to potential error messages
    .context("When trying to determine the parser to use")
    .parse(input)?;

    Ok((remainder, result))
}

pub fn parse(input: Span) -> Result<Vec<Token>, errors::ParseError> {
    let res = many0(parse_one)
        // Fail if there are any unparsed text
        .all_consuming()
        // Add context to potential error messages
        .context("When trying to parse the file")
        .parse(input)
        // Convert errors formats to more standard formats (rather than nom's)
        .finish();

    match res {
        // all_consuming should ensure there's no string left, so we can discard the first span
        Ok((_, res)) => Ok(res),
        Err(e) => Err(errors::format_parse_error(input.into_fragment(), e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_states() {
        let penup: Span = Span::new("PENUP");
        let (_, result): (_, Token) = parse_pen_state(penup).unwrap();
        assert_eq!(result, Token::PenUp);

        let pendown: Span = Span::new("PENDOWN");
        let (_, result): (_, Token) = parse_pen_state(pendown).unwrap();
        assert_eq!(result, Token::PenDown);
    }

    #[test]
    fn invalid_states() {
        let invalid: Span = Span::new("PENSIDEWAYS");
        assert!(parse_pen_state(invalid).is_err());
    }

    #[test]
    fn valid_directions() {
        let forward: Span = Span::new("FORWARD \"10");
        let (_, result): (_, Token) = parse_directions(forward).unwrap();
        assert_eq!(result, Token::Forward(10));

        let back: Span = Span::new("BACK \"10");
        let (_, result): (_, Token) = parse_directions(back).unwrap();
        assert_eq!(result, Token::Back(10));

        let left: Span = Span::new("LEFT \"10");
        let (_, result): (_, Token) = parse_directions(left).unwrap();
        assert_eq!(result, Token::Left(10));

        let right: Span = Span::new("RIGHT \"10");
        let (_, result): (_, Token) = parse_directions(right).unwrap();
        assert_eq!(result, Token::Right(10));
    }

    #[test]
    fn invalid_directions() {
        let invalid: Span = Span::new("NOWHERE \"10");
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn invalid_directions_no_distance() {
        let invalid: Span = Span::new("FORWARD");
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn invalid_directions_no_whitespace() {
        let invalid: Span = Span::new("BACK\"4");
        assert!(parse_directions(invalid).is_err());
    }

    #[test]
    fn valid_parse_one() {
        let input: Span = Span::new("PENUP\nFORWARD \"4\nPENDOWN");
        let (remainder, penup): (Span, Token) = parse_one(input).unwrap();
        assert_eq!(penup, Token::PenUp);

        let (remainder, forward): (Span, Token) = parse_one(remainder).unwrap();
        assert_eq!(forward, Token::Forward(4));

        let (remainder, pendown): (Span, Token) = parse_one(remainder).unwrap();
        assert_eq!(pendown, Token::PenDown);
        assert_eq!(remainder.into_fragment(), "");
    }

    #[test]
    fn valid_parse() {
        let input: Span = Span::new("PENUP\nBACK \"16\nRIGHT \"10\nPENDOWN");
        let result: Vec<Token> = match parse(input) {
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
