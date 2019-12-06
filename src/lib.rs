use std::str;

use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::error::ParseError;
use nom::multi::count;
use nom::number::complete as numbers;
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
    size_t_size: usize,
    int_size: usize,
    endianness: Option<Endianness>,
}

fn parse_header_content(input: &[u8]) -> IResult<&[u8], Header> {
    let from_u8 = |items| str::FromStr::from_str(str::from_utf8(items).unwrap());

    let (input, version) = numbers::double(input)?;

    if version != 4.1 {
        unimplemented!("Only MSH files version 4.1 are supported");
    }

    let (input, file_type) = preceded(sp, map(digit1, from_u8))(input)?;
    let (input, data_size) = delimited(sp, map(digit1, from_u8), br)(input)?;

    let file_type = file_type.unwrap();
    let data_size = data_size.unwrap();

    let endianness = if file_type == 1 {
        // Binary file
        let (_, i_be) = numbers::be_i32(input)?;
        let (_, i_le) = numbers::le_i32(input)?;

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
            size_t_size: data_size as usize,
            int_size: 4,
            endianness,
        },
    ))
}

#[derive(Debug)]
struct Point {}

#[derive(Debug)]
struct Curve {}

#[derive(Debug)]
struct Surface<IntT: num::Signed, FloatT: num::Float> {
    tag: IntT,
    min_x: FloatT,
    min_y: FloatT,
    min_z: FloatT,
    max_x: FloatT,
    max_y: FloatT,
    max_z: FloatT,
    physical_tags: Vec<IntT>,
    curve_tags: Vec<IntT>,
}

fn parse_surface<
    'a,
    SizeT: num::Unsigned + num::ToPrimitive,
    IntT: Clone + num::Signed,
    FloatT: num::Float,
    SizeTParser,
    IntParser,
    FloatParser,
    E: ParseError<&'a [u8]>,
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    input: &'a [u8],
) -> IResult<&'a [u8], Surface<IntT, FloatT>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], SizeT, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], IntT, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], FloatT, E>,
{
    let (input, surface_tag) = int_parser(input)?;

    let (input, min_x) = double_parser(input)?;
    let (input, min_y) = double_parser(input)?;
    let (input, min_z) = double_parser(input)?;
    let (input, max_x) = double_parser(input)?;
    let (input, max_y) = double_parser(input)?;
    let (input, max_z) = double_parser(input)?;

    let (input, num_physical_tags) = size_t_parser(input)?;
    let mut physical_tags = vec![IntT::zero(); num_physical_tags.to_usize().unwrap()];
    for j in 0..num_physical_tags.to_usize().unwrap() {
        physical_tags[j] = int_parser(input)?.1;
    }

    let (input, num_bounding_curves) = size_t_parser(input)?;
    let mut curve_tags = vec![IntT::zero(); num_bounding_curves.to_usize().unwrap()];
    for j in 0..num_bounding_curves.to_usize().unwrap() {
        curve_tags[j] = int_parser(input)?.1;
    }

    Ok((
        input,
        Surface {
            tag: surface_tag,
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
            physical_tags,
            curve_tags,
        },
    ))
}

#[derive(Debug)]
struct Volume {}

#[derive(Debug)]
struct Entities<IntT: num::Signed, FloatT: num::Float> {
    points: Vec<Point>,
    curves: Vec<Curve>,
    surfaces: Vec<Surface<IntT, FloatT>>,
    volumes: Vec<Volume>,
}

fn parse_entity_content<'a>(header: &Header, input: &'a [u8]) -> IResult<&'a [u8], Entities<i32, f64>> {
    print_u8("entities:", input);

    let size_t_parser =
        get_unsigned_integer_parser::<usize, _>(header.size_t_size, header.endianness);
    let (input, num_points) = size_t_parser(input)?;
    let (input, num_curves) = size_t_parser(input)?;
    let (input, num_surfaces) = size_t_parser(input)?;
    let (input, num_volumes) = size_t_parser(input)?;

    let int_parser = get_integer_parser::<i32, _>(header.int_size, header.endianness);
    let double_parser = get_float_parser::<f64, _>(8, header.endianness);

    for _ in 0..num_points {
        unimplemented!("Point entity reading not implemented")
    }

    for _ in 0..num_curves {
        unimplemented!("Curve entity reading not implemented")
    }

    let (input, surfaces) = count(
        |i| parse_surface(size_t_parser, int_parser, double_parser, i),
        num_surfaces,
    )(input)?;

    println!("Surfaces: {:?}", surfaces);

    for _ in 0..num_volumes {
        unimplemented!("Volume entity reading not implemented")
    }

    println!(
        "numPoints: {}, numCurves: {}, numSurfaces: {}, numVolumes: {}",
        num_points, num_curves, num_surfaces, num_volumes
    );

    Ok((
        input,
        Entities {
            points: Vec::new(),
            curves: Vec::new(),
            surfaces,
            volumes: Vec::new(),
        },
    ))
}

fn parse_file(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, header) = parsers::parse_delimited_block(
        terminated(tag("$MeshFormat"), br),
        terminated(tag("$EndMeshFormat"), br),
        parse_header_content,
    )(input)?;

    // TODO: Support arbitrary order and repetition of blocks, support unrecognized headers
    // To allow this, headers have to be recognized

    let (input, _entities) = parsers::parse_delimited_block(
        terminated(tag("$Entities"), br),
        terminated(tag("$EndEntities"), br),
        |i| parse_entity_content(&header, i),
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

fn get_unsigned_integer_parser<'a, T: num::Unsigned + num::Integer + num::NumCast, E: ParseError<&'a [u8]>>(
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
            unimplemented!("ASCII encoding not implemented");
        }
    }
}

fn get_integer_parser<'a, T: num::Signed + num::Integer + num::NumCast, E: ParseError<&'a [u8]>>(
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
            unimplemented!("ASCII encoding not implemented");
        }
    }
}

fn get_float_parser<'a, T: num::Float + num::NumCast, E: ParseError<&'a [u8]>>(
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
            unimplemented!("ASCII encoding not implemented");
        }
    }
}
