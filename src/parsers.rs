use nom::bytes::complete::{take, take_while};
use nom::error::ParseError;
use nom::sequence::delimited;
use nom::{AsChar, IResult};

use nom::{alt, named, tag};

named!(pub br, alt!(tag!("\n") | tag!("\r\n")));
named!(pub sp, alt!(tag!(" ") | tag!("\t") | tag!("\n") | tag!("\r\n")));

// Consumes spaces
pub fn take_sp<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: AsChar,
{
    let whitespaces = " \t\r\n";
    take_while(move |c: <I as nom::InputTakeAtPosition>::Item| whitespaces.contains(c.as_char()))(i)
}

// Applies a parser after consuming all enclosing spaces
pub fn ws<I, O, E: ParseError<I>, F>(parser: F) -> impl Fn(I) -> IResult<I, O, E>
where
    F: Fn(I) -> IResult<I, O, E>,
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    let delim = delimited(take_sp, parser, take_sp);
    move |input: I| delim(input)
}

/// ```
/// use nom::bytes::complete::tag;
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// let parser = mshio::parsers::take_till_parses::<_,_,(_,_),_>(tag("$Hello"));
///
/// assert_eq!(parser("123\n$Hello456"), Ok(("$Hello456", "123\n")));
/// assert_eq!(parser("$Hello456"), Ok(("$Hello456", "")));
/// assert_eq!(parser("llo456"), Err(Err::Error(ParseError::from_error_kind("llo456", ErrorKind::Eof))));
/// ```
pub fn take_till_parses<I, O, E: ParseError<I>, F>(parser: F) -> impl Fn(I) -> IResult<I, I, E>
where
    I: Clone + nom::InputIter + nom::InputTake,
    F: Fn(I) -> IResult<I, O, E>,
{
    move |mut input: I| {
        let mut bytes_taken: usize = 0;
        let org_input = input.clone();
        loop {
            if parser(input.clone()).is_ok() {
                return take(bytes_taken as usize)(org_input);
            }

            bytes_taken += 1;
            match take(bytes_taken)(org_input.clone()) {
                Ok((i, _)) => {
                    input = i;
                }
                e @ Err(_) => return e,
            };
        }
    }
}

/// ```
/// use nom::bytes::complete::tag;
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// let parser = mshio::parsers::take_after_parses::<_,_,(_,_),_>(tag("$Hello"));
///
/// assert_eq!(parser("123\n$Hello456"), Ok(("456", ("123\n", "$Hello"))));
/// assert_eq!(parser("$Hello456"), Ok(("456", ("", "$Hello"))));
/// assert_eq!(parser("llo456"), Err(Err::Error(ParseError::from_error_kind("llo456", ErrorKind::Eof))));
/// ```
pub fn take_after_parses<I, O, E: ParseError<I>, F>(
    parser: F,
) -> impl Fn(I) -> IResult<I, (I, O), E>
where
    I: Clone + nom::InputIter + nom::InputTake,
    F: Fn(I) -> IResult<I, O, E>,
{
    move |mut input: I| {
        let mut bytes_taken: usize = 0;
        let org_input = input.clone();
        loop {
            if let Ok((i, t)) = parser(input.clone()) {
                let (_, c) = take(bytes_taken as usize)(org_input)?;
                return Ok((i, (c, t)));
            }

            bytes_taken += 1;
            match take(bytes_taken)(org_input.clone()) {
                Ok((i, _)) => {
                    input = i;
                }
                Err(e) => return Err(e),
            };
        }
    }
}

/// ```
/// use nom::bytes::complete::tag;
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// let parser = mshio::parsers::delimited_block::<_,_,(_,_),_,_>(tag("$Start"), tag("$End"));
///
/// assert_eq!(parser("$Start\n123\n$End\n456"), Ok(("\n456", "\n123\n")));
/// assert_eq!(parser("$Start$End\n456"), Ok(("\n456", "")));
/// assert_eq!(parser("$Start$End"), Ok(("", "")));
/// ```
pub fn delimited_block<I, O, E: ParseError<I>, F, G>(
    start: F,
    end: G,
) -> impl Fn(I) -> IResult<I, I, E>
where
    I: Clone + nom::InputIter + nom::InputTake,
    F: Fn(I) -> IResult<I, O, E>,
    G: Fn(I) -> IResult<I, O, E>,
{
    let taker = take_after_parses(end);
    move |input: I| {
        let (input, _) = start(input)?;
        let (input, (content, _)) = taker(input)?;

        Ok((input, content))
    }
}

/// ```
/// use std::str::FromStr;
/// use nom::bytes::complete::tag;
/// use nom::character::complete::digit0;
/// use nom::combinator::map;
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// use mshio::parsers::parse_delimited_block;
/// let parser = parse_delimited_block::<_,_,_,_,(_,_),_,_,_>(tag("$Start\n"), tag("$End\n"), map(digit0, FromStr::from_str));
///
/// assert_eq!(parser("$Start\n123\n$End\n456"), Ok(("456", Ok(123))));
/// assert_eq!(parser("$Start\n123\n$End\n"), Ok(("", Ok(123))));
/// assert_eq!(parser("$Start\n$End\n"), Ok(("", i32::from_str(""))));
/// ```
pub fn parse_delimited_block<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    start: F,
    end: G,
    parser: H,
) -> impl Fn(I) -> IResult<I, O3, E>
where
    I: Clone + nom::InputIter + nom::InputTake,
    F: Fn(I) -> IResult<I, O1, E>,
    G: Fn(I) -> IResult<I, O2, E>,
    H: Fn(I) -> IResult<I, O3, E>,
{
    let taker = take_after_parses(end);
    move |input: I| {
        let (input, _) = start(input)?;
        let (input, (content, _)) = taker(input)?;

        let (_, out) = parser(content)?;

        Ok((input, out))
    }
}
