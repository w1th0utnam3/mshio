use std::str;

use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::number::complete::{be_i32, double, le_i32};
use nom::number::Endianness;
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;

pub mod parsers;

use parsers::{br, sp};

/// Debug helper to view u8 slice as utf8 str and print it
#[allow(dead_code)]
fn print_u8(text: &str, input: &[u8]) {
    println!("{}: '{}'", text, String::from_utf8_lossy(input));
}

#[derive(Debug)]
struct Header {
    version: f64,
    file_type: i32,
    data_size: i32,
    endianness: Option<Endianness>,
}

fn parse_header_content(input: &[u8]) -> IResult<&[u8], Header> {
    let from_u8 = |items| str::FromStr::from_str(str::from_utf8(items).unwrap());

    let (input, version) = double(input)?;

    if version != 4.1 {
        unimplemented!("Only MSH files version 4.1 are supported");
    }

    let (input, file_type) = preceded(sp, map(digit1, from_u8))(input)?;
    let (input, data_size) = delimited(sp, map(digit1, from_u8), br)(input)?;

    let file_type = file_type.unwrap();
    let data_size = data_size.unwrap();

    let endianness = if file_type == 1 {
        // Binary file
        let (_, i_be) = be_i32(input)?;
        let (_, i_le) = le_i32(input)?;

        println!("be: {}, le: {}", i_be, i_le);

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
        Header {
            version,
            file_type,
            data_size,
            endianness,
        },
    ))
}

#[derive(Debug)]
struct Entities {}

fn parse_entity_content(input: &[u8]) -> IResult<&[u8], Entities> {
    print_u8("entities:", input);
    Ok((input, Entities {}))
}

fn parse_file(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, header) = parsers::parse_delimited_block(
        terminated(tag("$MeshFormat"), br),
        terminated(tag("$EndMeshFormat"), br),
        parse_header_content,
    )(input)?;

    let (input, _entities) = parsers::parse_delimited_block(
        terminated(tag("$Entities"), br),
        terminated(tag("$EndEntities"), br),
        parse_entity_content,
    )(input)?;

    Ok((input, header))
}

pub fn parse(msh: &[u8]) {
    match parse_file(msh) {
        Ok((_, header)) => {
            println!("Successfully parsed:");
            println!("{:?}", header);
        }
        Err(err) => {
            println!("{:?}", err);
            panic!("")
        }
    }
}
