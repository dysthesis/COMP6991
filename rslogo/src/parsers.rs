use nom::{branch::alt, bytes::complete::tag, IResult, Parser};

#[derive(PartialEq, Debug)]
enum Token {
    PenUp,
    PenDown,
    Forward(i32),
    Backward(i32),
    Left(i32),
    Right(i32),
}

type Program = Vec<Token>;

fn parse_pen_state(input: &str) -> IResult<&str, Token> {
    let (input, parsed): (&str, &str) = alt((tag("PENUP"), tag("PENDOWN"))).parse(input)?;

    let result = match parsed {
        "PENUP" => Token::PenUp,

        "PENDOWN" => Token::PenDown,

        _ => {
            // TODO: Make this return an actual error
            todo!()
        }
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
}
