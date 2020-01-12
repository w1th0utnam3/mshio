use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_while};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::delimited;
use nom::{AsChar, IResult};

/// Consumes the whole input
///
/// ```
/// let parser = mshio::parsers::anything::<_,(_,_)>;
///
/// assert_eq!(parser(""), Ok(("", "")));
/// assert_eq!(parser("123"), Ok(("", "123")));
/// assert_eq!(parser("123 456\t\n "), Ok(("", "123 456\t\n ")));
/// ```
#[allow(dead_code)]
pub fn anything<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: nom::InputLength + nom::InputIter + nom::InputTake,
{
    take(i.input_len())(i)
}

/// Parses successfully if the input is empty and returns it
///
/// ```
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// let parser = mshio::parsers::eof::<_,(_,_)>;
///
/// assert_eq!(parser(""), Ok(("", "")));
/// assert_eq!(parser("123"), Err(Err::Error(ParseError::from_error_kind("123", ErrorKind::Eof))));
/// ```
pub fn eof<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: Clone + nom::InputLength,
{
    if i.input_len() == 0 {
        Ok((i.clone(), i))
    } else {
        Err(nom::Err::Error(ParseError::from_error_kind(i, ErrorKind::Eof)))
    }
}

/// Consumes a single linebreak
///
/// ```
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// let parser = mshio::parsers::br::<_,(_,_)>;
///
/// assert_eq!(parser("\n123"), Ok(("123", "\n")));
/// assert_eq!(parser("\r\n123456"), Ok(("123456", "\r\n")));
/// assert_eq!(parser("123"), Err(Err::Error(ParseError::from_error_kind("123", ErrorKind::Tag))));
/// ```
pub fn br<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: Clone + nom::InputTake + nom::Compare<&'static str>,
{
    alt((tag("\n"), tag("\r\n")))(i)
}

/// Consumes a single whitespaces character
///
/// ```
/// use nom::Err;
/// use nom::error::{ErrorKind, ParseError};
///
/// let parser = mshio::parsers::sp::<_,(_,_)>;
///
/// assert_eq!(parser(" 123"), Ok(("123", " ")));
/// assert_eq!(parser("\t123"), Ok(("123", "\t")));
/// assert_eq!(parser("\n123"), Ok(("123", "\n")));
/// assert_eq!(parser("\r\n123"), Ok(("123", "\r\n")));
/// assert_eq!(parser("123"), Err(Err::Error(ParseError::from_error_kind("123", ErrorKind::Tag))));
/// ```
pub fn sp<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: Clone + nom::InputTake + nom::Compare<&'static str>,
{
    alt((tag(" "), tag("\t"), tag("\n"), tag("\r\n")))(i)
}

/// Consumes all preceding whitespaces
///
/// ```
/// let parser = mshio::parsers::take_sp::<_,(_,_)>;
///
/// assert_eq!(parser(" \t \n\r\n123"), Ok(("123", " \t \n\r\n")));
/// assert_eq!(parser("123"), Ok(("123", "")));
/// ```
pub fn take_sp<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: AsChar,
{
    take_while(|c: <I as nom::InputTakeAtPosition>::Item| " \t\r\n".contains(c.as_char()))(i)
}

/// Removes preceding whitespaces, applies parser and consumes trailing whitespaces
///
/// ```
/// use std::str::FromStr;
/// use nom::character::complete::digit0;
/// use nom::combinator::map;
///
/// let parser = mshio::parsers::ws::<_,_,(_,_),_>(map(digit0, FromStr::from_str));
///
/// assert_eq!(parser(" \t \n\r\n123  \n"), Ok(("", Ok(123))));
/// assert_eq!(parser(" \t \n\r\n123  \n456"), Ok(("456", Ok(123))));
/// assert_eq!(parser(" \n456"), Ok(("", Ok(456))));
/// assert_eq!(parser(" \nabc  "), Ok(("abc  ", i32::from_str(""))));
/// assert_eq!(parser("abc"), Ok(("abc", i32::from_str(""))));
/// ```
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
#[allow(dead_code)]
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
