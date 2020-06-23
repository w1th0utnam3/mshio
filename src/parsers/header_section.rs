use std::str;

use nom::character::complete::digit1;
use nom::combinator::map;
use nom::number::complete as numbers;
use nom::number::Endianness;
use nom::sequence::{delimited, preceded};
use nom::IResult;

use crate::error::{
    always_error, context, make_error, MapMshError, MshParserError, MshParserErrorKind,
};
use crate::mshfile::{MshFloatT, MshHeader, MshIntT, MshUsizeT};
use crate::parsers::num_parser_traits::{ParsesFloat, ParsesInt, ParsesSizeT};
use crate::parsers::num_parsers;
use crate::parsers::{br, sp, verify_or};

pub(crate) fn parse_header_section<'a>(
    input: &'a [u8],
) -> IResult<
    &'a [u8],
    (
        MshHeader,
        impl ParsesSizeT<u64> + ParsesInt<i32> + ParsesFloat<f64>,
    ),
    MshParserError<&'a [u8]>,
> {
    // TODO: Replace this expect
    let from_u8 =
        |items| str::FromStr::from_str(str::from_utf8(items).expect("Cannot parse UTF8 to digits"));

    let (input, version) = verify_or(
        numbers::double,
        |&version| version == 4.1,
        always_error(MshParserErrorKind::UnsupportedMshVersion),
    )(input)?;

    let (input, file_type) = context(
        "file type flag",
        verify_or(
            preceded(sp, map(digit1, from_u8)),
            |file_type| *file_type == Ok(0) || *file_type == Ok(1),
            context(
                "Invalid file type (expected 0 for ASCII or 1 for binary)",
                always_error(MshParserErrorKind::InvalidFileHeader),
            ),
        ),
    )(input)?;
    // TODO: Check for all allowed data sizes
    let (input, data_size) = context("data size", delimited(sp, map(digit1, from_u8), br))(input)?;

    // TODO: Replace these unwraps
    let file_type = file_type.unwrap();
    let data_size = data_size.unwrap();

    let endianness = if file_type == 1 {
        // Binary file
        let (_, i_be) = context("endianness test value", numbers::be_i32)(input)?;
        let (_, i_le) = context("endianness test value", numbers::le_i32)(input)?;

        if i_be == 1 {
            Some(Endianness::Big)
        } else if i_le == 1 {
            Some(Endianness::Little)
        } else {
            return Err(make_error(input, MshParserErrorKind::InvalidFileHeader)
                .with_context(input, "Unable to detect endianness of binary file"));
        }
    } else {
        // ASCII file
        None
    };

    let header = MshHeader {
        version,
        file_type,
        size_t_size: data_size as usize,
        int_size: 4,
        float_size: 8,
        endianness,
    };
    let parsers = num_parsers_from_header(&header);

    Ok((input, (header, parsers)))
}

pub(crate) fn num_parsers_from_header<'a, U: MshUsizeT, I: MshIntT, F: MshFloatT>(
    header: &'a MshHeader,
) -> impl ParsesSizeT<U> + ParsesInt<I> + ParsesFloat<F> {
    let size_t_parser = num_parsers::uint_parser::<U>(header.size_t_size, header.endianness);
    let int_parser = num_parsers::int_parser::<I>(header.int_size, header.endianness);
    let double_parser = num_parsers::float_parser::<F>(header.float_size, header.endianness);

    num_parsers::NumParsers {
        size_t_parser,
        int_parser,
        float_parser: double_parser,
    }
}
