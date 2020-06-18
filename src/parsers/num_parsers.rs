use std::str;

use nom::character::complete::digit1;
use nom::combinator::map;
#[allow(unused)]
use nom::error::VerboseError;
use nom::error::{ErrorKind, ParseError};
use nom::number::complete as numbers;
use nom::number::Endianness;
use nom::IResult;

use num::Integer;
use num_traits::{Float, NumCast, Signed, Unsigned};

use crate::error::{error_strings, nom_error};
use crate::parsers::{recognize_integer, ws};

pub fn uint_parser<'a, T: Unsigned + Integer + NumCast + str::FromStr, E: ParseError<&'a [u8]>>(
    source_size: usize,
    endianness: Option<Endianness>,
) -> impl Copy + Fn(&'a [u8]) -> IResult<&'a [u8], T, E> {
    /*
    if std::mem::size_of::<T>() < source_size {
        panic!("Input unsigned integer size of {} bytes is too large for target unsigned integer size of {} bytes", source_size, std::mem::size_of::<T>());
    }
    */

    macro_rules! generate_parser {
        ($parser:expr) => {
            (|i| match $parser(i) {
                Ok((i, v)) => {
                    if let Some(v) = T::from(v) {
                        Ok(((i, v)))
                    } else {
                        nom_error(error_strings::UINT_PARSING_ERROR, ErrorKind::ParseTo)(i)
                    }
                }
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
                str::FromStr::from_str(str::from_utf8(items).expect("Cannot parse UTF8 to digits"))
            }))(i)
            {
                Ok((i, v)) => match v {
                    Ok(v) => Ok((i, v)),
                    Err(_) => nom_error(error_strings::UINT_PARSING_ERROR, ErrorKind::ParseTo)(i),
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
    /*
    if std::mem::size_of::<T>() < source_size {
        panic!(
            "Input integer input of {} bytes is too large for target integer size of {} bytes",
            source_size,
            std::mem::size_of::<T>()
        );
    }
    */

    macro_rules! generate_parser {
        ($parser:expr) => {
            (|i| match $parser(i) {
                Ok((i, v)) => {
                    if let Some(v) = T::from(v) {
                        Ok(((i, v)))
                    } else {
                        nom_error(error_strings::INT_PARSING_ERROR, ErrorKind::ParseTo)(i)
                    }
                }
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
                str::FromStr::from_str(str::from_utf8(items).expect("Cannot parse UTF8 to integer"))
            }))(i)
            {
                Ok((i, v)) => match v {
                    Ok(v) => Ok((i, v)),
                    Err(_) => nom_error(error_strings::INT_PARSING_ERROR, ErrorKind::ParseTo)(i),
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
    /*
    if std::mem::size_of::<T>() < source_size {
        panic!(
            "Input float size of {} bytes is too large for target float size of {} bytes",
            source_size,
            std::mem::size_of::<T>()
        );
    }
    */

    macro_rules! generate_parser {
        ($parser:expr) => {
            (|i| match $parser(i) {
                Ok((i, v)) => {
                    if let Some(v) = T::from(v) {
                        Ok(((i, v)))
                    } else {
                        nom_error(error_strings::FLOAT_PARSING_ERROR, ErrorKind::ParseTo)(i)
                    }
                }
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
                Ok((i, v)) => {
                    if let Some(v) = T::from(v) {
                        Ok((i, v))
                    } else {
                        nom_error(error_strings::FLOAT_PARSING_ERROR, ErrorKind::ParseTo)(i)
                    }
                }
                Err(e) => Err(e),
            }) as fn(&'a [u8]) -> IResult<&'a [u8], T, E>
        }
    }
}

// Generates a test that checks if parsing of a large value into a smaller type is handled correctly
macro_rules! generate_num_parser_oversized_values_test {
    ($test_name:ident, $parser_name:ident, $large_type:ident, $small_type:ident) => {
        #[test]
        fn $test_name() {
            // Construct a value that is too large for the smaller type
            let big_value: $large_type = <$large_type as NumCast>::from(2.0).unwrap()
                * <$large_type as NumCast>::from($small_type::MAX).unwrap();
            // Ensure that the value is too large for the smaller type
            assert_eq!(<$small_type as NumCast>::from(big_value), None);

            // Construct a value that fits into the smaller type
            let small_value: $large_type =
                <$large_type as NumCast>::from($small_type::MAX).unwrap();
            // Ensure round trip works
            assert_eq!(
                <$large_type as NumCast>::from(
                    <$small_type as NumCast>::from(small_value).unwrap()
                )
                .unwrap(),
                small_value
            );

            // Generate inputs for parsing
            let big_value_str: String = big_value.to_string();
            let big_value_be = big_value.to_be_bytes();
            let big_value_le = big_value.to_le_bytes();

            let small_value_str: String = small_value.to_string();
            let small_value_be = small_value.to_be_bytes();
            let small_value_le = small_value.to_le_bytes();

            macro_rules! generate_endianness_tests {
                ($endianness:expr, $big_input:expr, $small_input:expr) => {
                    // Ensure: large value input -> large type: works
                    {
                        let parser = $parser_name::<$large_type, VerboseError<_>>(
                            std::mem::size_of::<$large_type>(),
                            $endianness,
                        );
                        let result = parser($big_input);
                        assert!(result.is_ok());
                        assert_eq!(result.unwrap().1, big_value);
                    }

                    // Ensure: large value input -> smaller type: error
                    {
                        let parser = $parser_name::<$small_type, VerboseError<_>>(
                            std::mem::size_of::<$large_type>(),
                            $endianness,
                        );
                        let result = parser($big_input);
                        assert!(result.is_err());
                    }

                    // Ensure: small value input -> smaller type: works
                    {
                        let parser = $parser_name::<$small_type, VerboseError<_>>(
                            std::mem::size_of::<$large_type>(),
                            $endianness,
                        );
                        let result = parser($small_input);
                        assert!(result.is_ok());
                        assert_eq!(
                            <$large_type as NumCast>::from(result.unwrap().1).unwrap(),
                            small_value
                        );
                    }
                };
            }

            generate_endianness_tests!(None, big_value_str.as_bytes(), small_value_str.as_bytes());
            generate_endianness_tests!(Some(Endianness::Big), &big_value_be, &small_value_be);
            generate_endianness_tests!(Some(Endianness::Little), &big_value_le, &small_value_le);
        }
    };
}

generate_num_parser_oversized_values_test!(
    test_uint_parser_oversized_values,
    uint_parser,
    u64,
    u32
);
generate_num_parser_oversized_values_test!(test_int_parser_oversized_values, int_parser, i64, i32);
generate_num_parser_oversized_values_test!(
    test_float_parser_oversized_values,
    float_parser,
    f64,
    f32
);
