use std::str;

use nom::character::complete::digit1;
use nom::combinator::map;
use nom::error::{ErrorKind, ParseError};
use nom::number::complete as numbers;
use nom::number::Endianness;
use nom::IResult;

use num::Integer;
use num_traits::{Float, NumCast, Signed, Unsigned};

use crate::parsers::{recognize_integer, ws};

pub fn uint_parser<'a, T: Unsigned + Integer + NumCast + str::FromStr, E: ParseError<&'a [u8]>>(
    source_size: usize,
    endianness: Option<Endianness>,
) -> impl Copy + Fn(&'a [u8]) -> IResult<&'a [u8], T, E> {
    if std::mem::size_of::<T>() < source_size {
        panic!("Input unsigned integer size of {} bytes is too large for target unsigned integer size of {} bytes", source_size, std::mem::size_of::<T>());
    }

    macro_rules! generate_parser {
        ($parser:expr) => {
            (|i| match $parser(i) {
                Ok((i, v)) => Ok((i, T::from(v).unwrap())),
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        };
    }

    match endianness {
        Some(Endianness::Little) => match source_size {
            1 => return generate_parser!(numbers::le_u8),
            2 => return generate_parser!(numbers::le_u16),
            4 => return generate_parser!(numbers::le_u32),
            8 => return generate_parser!(numbers::le_u64),
            16 => return generate_parser!(numbers::le_u128),
            _ => {
                unimplemented!(
                    "No parser for input unsigned integer size of {} bytes available",
                    source_size
                );
            }
        },
        Some(Endianness::Big) => match source_size {
            1 => return generate_parser!(numbers::be_u8),
            2 => return generate_parser!(numbers::be_u16),
            4 => return generate_parser!(numbers::be_u32),
            8 => return generate_parser!(numbers::be_u64),
            16 => return generate_parser!(numbers::be_u128),
            _ => {
                unimplemented!(
                    "No parser for input unsigned integer size of {} bytes available",
                    source_size
                );
            }
        },
        None => {
            (|i| match ws(map(digit1, |items| {
                str::FromStr::from_str(str::from_utf8(items).unwrap())
            }))(i)
            {
                Ok((i, v)) => match v {
                    Ok(v) => Ok((i, v)),
                    Err(_) => Err(nom::Err::Error(ParseError::from_error_kind(
                        i,
                        ErrorKind::ParseTo,
                    ))),
                },
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        }
    }
}

pub fn int_parser<'a, T: Signed + Integer + NumCast + str::FromStr, E: ParseError<&'a [u8]>>(
    source_size: usize,
    endianness: Option<Endianness>,
) -> impl Copy + Fn(&'a [u8]) -> IResult<&'a [u8], T, E> {
    if std::mem::size_of::<T>() < source_size {
        panic!(
            "Input integer input of {} bytes is too large for target integer size of {} bytes",
            source_size,
            std::mem::size_of::<T>()
        );
    }

    macro_rules! generate_parser {
        ($parser:expr) => {
            (|i| match $parser(i) {
                Ok((i, v)) => Ok((i, T::from(v).unwrap())),
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        };
    }

    match endianness {
        Some(Endianness::Little) => match source_size {
            1 => return generate_parser!(numbers::le_i8),
            2 => return generate_parser!(numbers::le_i16),
            4 => return generate_parser!(numbers::le_i32),
            8 => return generate_parser!(numbers::le_i64),
            16 => return generate_parser!(numbers::le_i128),
            _ => {
                unimplemented!(
                    "No parser for input integer size of {} bytes available",
                    source_size
                );
            }
        },
        Some(Endianness::Big) => match source_size {
            1 => return generate_parser!(numbers::be_i8),
            2 => return generate_parser!(numbers::be_i16),
            4 => return generate_parser!(numbers::be_i32),
            8 => return generate_parser!(numbers::be_i64),
            16 => return generate_parser!(numbers::be_i128),
            _ => {
                unimplemented!(
                    "No parser for source integer size of {} bytes available",
                    source_size
                );
            }
        },
        None => {
            (|i| match ws(map(recognize_integer, |items| {
                str::FromStr::from_str(str::from_utf8(items).unwrap())
            }))(i)
            {
                Ok((i, v)) => match v {
                    Ok(v) => Ok((i, v)),
                    Err(_) => Err(nom::Err::Error(ParseError::from_error_kind(
                        i,
                        ErrorKind::ParseTo,
                    ))),
                },
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        }
    }
}

pub fn float_parser<'a, T: Float + NumCast, E: ParseError<&'a [u8]>>(
    source_size: usize,
    endianness: Option<Endianness>,
) -> impl Copy + Fn(&'a [u8]) -> IResult<&'a [u8], T, E> {
    if std::mem::size_of::<T>() < source_size {
        panic!(
            "Input float size of {} bytes is too large for target float size of {} bytes",
            source_size,
            std::mem::size_of::<T>()
        );
    }

    macro_rules! generate_parser {
        ($parser:expr) => {
            (|i| match $parser(i) {
                Ok((i, v)) => Ok((i, T::from(v).unwrap())),
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        };
    }

    match endianness {
        Some(Endianness::Little) => match source_size {
            4 => return generate_parser!(numbers::le_f32),
            8 => return generate_parser!(numbers::le_f64),
            _ => {
                unimplemented!(
                    "No parser for input float size of {} bytes available",
                    source_size
                );
            }
        },
        Some(Endianness::Big) => match source_size {
            4 => return generate_parser!(numbers::be_f32),
            8 => return generate_parser!(numbers::be_f64),
            _ => {
                unimplemented!(
                    "No parser for input float size of {} bytes available",
                    source_size
                );
            }
        },
        None => {
            (|i| match ws(numbers::double)(i) {
                Ok((i, v)) => Ok((i, T::from(v).unwrap())),
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        }
    }
}
