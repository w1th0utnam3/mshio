use std::str;

use nom::character::complete::digit1;
use nom::combinator::map;
use nom::error::{ErrorKind, ParseError};
use nom::number::complete as numbers;
use nom::number::Endianness;
use nom::sequence::{delimited, preceded};
use nom::IResult;

use crate::error::{create_error, error_strings};
use crate::mshfile::MshHeader;
use crate::parsers::{br, sp};

pub(crate) fn parse_header_section<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], MshHeader, E> {
    let from_u8 = |items| str::FromStr::from_str(str::from_utf8(items).unwrap());

    let (input, version) = numbers::double(input)?;

    if version != 4.1 {
        return create_error(error_strings::MSH_VERSION_UNSUPPORTED, ErrorKind::Tag)(input);
    }

    let (input, file_type) = preceded(sp, map(digit1, from_u8))(input)?;
    let (input, data_size) = delimited(sp, map(digit1, from_u8), br)(input)?;

    let file_type = file_type.unwrap();
    let data_size = data_size.unwrap();

    let endianness = if file_type == 1 {
        // Binary file
        let (_, i_be) = numbers::be_i32(input)?;
        let (_, i_le) = numbers::le_i32(input)?;

        if i_be == 1 {
            Some(Endianness::Big)
        } else if i_le == 1 {
            Some(Endianness::Little)
        } else {
            unimplemented!("Unable to detect endianness of binary file");
        }
    } else if file_type == 0 {
        // ASCII file
        None
    } else {
        unimplemented!("Unsupported file type (expected 0 for ASCII or 1 for binary)");
    };

    Ok((
        input,
        MshHeader {
            version,
            file_type,
            size_t_size: data_size as usize,
            int_size: 4,
            float_size: 8,
            endianness,
        },
    ))
}
