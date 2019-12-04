use std::str;

use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{digit1};
use nom::combinator::map;
use nom::error::ParseError;
use nom::number::complete::{be_i32, double, le_i32};
use nom::number::Endianness;
use nom::sequence::delimited;
use nom::{AsChar, IResult, InputTakeAtPosition};

#[derive(Debug)]
struct Header {
    version: f64,
    file_type: i32,
    data_size: i32,
    endianness: Option<Endianness>,
}

// Consumes spaces
fn sp<I, E: ParseError<I>>(i: I) -> IResult<I, I, E>
where
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar,
{
    let whitespaces = " \t\r\n";
    take_while(move |c: <I as InputTakeAtPosition>::Item| whitespaces.contains(c.as_char()))(i)
}

// Matches a parser possibly surrounded by spaces
fn ws<I, O, E: ParseError<I>, F>(parser: F) -> impl Fn(I) -> IResult<I, O, E>
where
    F: Fn(I) -> IResult<I, O, E>,
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar,
{
    let delim = delimited(sp, parser, sp);
    move |input: I| delim(input)
}

fn parse_header_content(input: &[u8]) -> IResult<&[u8], Header> {
    let from_u8 = |items| str::FromStr::from_str(str::from_utf8(items).unwrap());

    let (input, version) = ws(double)(input)?;
    let (input, file_type) = ws(map(digit1, from_u8))(input)?;
    let (input, data_size) = ws(map(digit1, from_u8))(input)?;

    let file_type = file_type.unwrap();
    let data_size = data_size.unwrap();

    println!(
        "remaining header: '{:?}'",
        std::str::from_utf8(input).unwrap()
    );

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
        unimplemented!("Unsupported file type");
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

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, _tag) = ws(tag("$MeshFormat"))(input)?;
    let (input, header_content) = take_until("$EndMeshFormat")(input)?;
    let (input, _tag) = ws(tag("$EndMeshFormat"))(input)?;

    println!(
        "header_content: '{:?}'",
        std::str::from_utf8(header_content).unwrap()
    );
    let (_, header) = parse_header_content(header_content)?;

    Ok((input, header))
}

pub fn parse(msh: &[u8]) {
    println!("{:?}", msh);
    match parse_header(msh) {
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
