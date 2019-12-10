use std::collections::HashMap;
use std::hash::Hash;
use std::str;

use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::error::{ParseError, VerboseError};
use nom::multi::count;
use nom::number::complete as numbers;
use nom::number::Endianness;
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;

use num::{Float, Integer, Signed, Unsigned};

pub mod general_parsers;
pub mod mesh_data;
mod num_parsers;

pub use general_parsers as parsers;
use general_parsers::{br, sp};
use mesh_data::{
    Element, ElementEntity, Elements, Entities, MshData, MshFile, MshHeader, Node, NodeEntity,
    Nodes, Surface,
};

// TODO: Replace panics and unimplemented! calls with Err

/// Debug helper to view u8 slice as utf8 str and print it
#[allow(dead_code)]
fn print_u8(text: &str, input: &[u8]) {
    println!("{}: '{}'", text, String::from_utf8_lossy(input));
}

fn parse_header_section<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], MshHeader, E> {
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
            endianness,
        },
    ))
}

fn parse_surface<
    'a,
    SizeT: Unsigned + Integer + num::ToPrimitive,
    IntT: Clone + Signed + Integer,
    FloatT: Float,
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
    let num_physical_tags = num_physical_tags.to_usize().unwrap();

    let mut physical_tags = vec![IntT::zero(); num_physical_tags];
    for j in 0..num_physical_tags {
        physical_tags[j] = int_parser(input)?.1;
    }

    let (input, num_bounding_curves) = size_t_parser(input)?;
    let num_bounding_curves = num_bounding_curves.to_usize().unwrap();

    let mut curve_tags = vec![IntT::zero(); num_bounding_curves];
    for j in 0..num_bounding_curves {
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

fn parse_entity_section<'a>(
    header: &MshHeader,
    input: &'a [u8],
) -> IResult<&'a [u8], Entities<i32, f64>, VerboseError<&'a [u8]>> {
    let size_t_parser = num_parsers::uint_parser::<usize, _>(header.size_t_size, header.endianness);
    let (input, num_points) = size_t_parser(input)?;
    let (input, num_curves) = size_t_parser(input)?;
    let (input, num_surfaces) = size_t_parser(input)?;
    let (input, num_volumes) = size_t_parser(input)?;

    let int_parser = num_parsers::int_parser::<i32, _>(header.int_size, header.endianness);
    let double_parser = num_parsers::float_parser::<f64, _>(8, header.endianness);

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

    for _ in 0..num_volumes {
        unimplemented!("Volume entity reading not implemented")
    }

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

fn parse_node_entity<
    'a,
    SizeT: Unsigned + Integer + num::ToPrimitive + Hash,
    IntT: Signed + Integer + num::ToPrimitive,
    FloatT: Float,
    SizeTParser,
    IntParser,
    FloatParser,
    E: ParseError<&'a [u8]>,
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], NodeEntity<SizeT, IntT, FloatT>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], SizeT, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], IntT, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], FloatT, E>,
{
    let (input, entity_dim) = int_parser(input)?;
    let (input, entity_tag) = int_parser(input)?;
    let (input, parametric) = int_parser(input)?;
    let (input, num_nodes_in_block) = size_t_parser(input)?;
    let num_nodes_in_block = num_nodes_in_block.to_usize().unwrap();

    let parametric = if parametric == IntT::zero() {
        false
    } else if parametric == IntT::one() {
        true
    } else {
        panic!("Unsupported value for node block attribute 'parametric' (only 0 and 1 supported)")
    };

    if parametric {
        unimplemented!("Parametric nodes are not supported yet");
    }

    let parse_node_tag = |input| {
        let (input, node_tag) = size_t_parser(input)?;
        Ok((input, node_tag))
    };

    let (input, node_tags) = if sparse_tags {
        let (input, node_tags) = count(parse_node_tag, num_nodes_in_block)(input)?;
        (
            input,
            Some(
                node_tags
                    .into_iter()
                    .enumerate()
                    .map(|(i, tag)| (tag, i))
                    .collect::<HashMap<_, _>>(),
            ),
        )
    } else {
        let (input, _) = count(parse_node_tag, num_nodes_in_block)(input)?;
        (input, None)
    };

    let parse_node = |input| {
        let (input, x) = double_parser(input)?;
        let (input, y) = double_parser(input)?;
        let (input, z) = double_parser(input)?;

        Ok((input, Node { x, y, z }))
    };

    let (input, nodes) = count(parse_node, num_nodes_in_block as usize)(input)?;

    Ok((
        input,
        NodeEntity {
            entity_dim,
            entity_tag,
            parametric,
            node_tags,
            nodes,
            parametric_nodes: None,
        },
    ))
}

fn parse_node_section<'a>(
    header: &MshHeader,
    input: &'a [u8],
) -> IResult<&'a [u8], Nodes<usize, i32, f64>, VerboseError<&'a [u8]>> {
    let size_t_parser = num_parsers::uint_parser::<usize, _>(header.size_t_size, header.endianness);

    let (input, num_entity_blocks) = size_t_parser(input)?;
    let (input, num_nodes) = size_t_parser(input)?;
    let (input, min_node_tag) = size_t_parser(input)?;
    let (input, max_node_tag) = size_t_parser(input)?;

    let int_parser = num_parsers::int_parser::<i32, _>(header.int_size, header.endianness);
    let double_parser = num_parsers::float_parser::<f64, _>(8, header.endianness);

    let sparse_tags = if min_node_tag == 0 {
        panic!("Node tag 0 is reserved for internal use");
    } else if max_node_tag - min_node_tag > num_nodes - 1 {
        true
    } else {
        false
    };

    let (input, node_entity_blocks) = count(
        |i| parse_node_entity(size_t_parser, int_parser, double_parser, sparse_tags, i),
        num_entity_blocks,
    )(input)?;

    Ok((
        input,
        Nodes {
            min_node_tag: min_node_tag,
            max_node_tag: max_node_tag,
            node_entities: node_entity_blocks,
        },
    ))
}

/// Returns the number of nodes per element type defined by GMSH
///
/// See http://gmsh.info/doc/texinfo/gmsh.html#MSH-file-format
/// and https://gitlab.onelab.info/gmsh/gmsh/blob/master/Common/GmshDefines.h
fn nodes_per_element(element_type: i32) -> usize {
    match element_type {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 4,
        5 => 8,
        6 => 6,
        7 => 5,
        _ => unimplemented!("Unsupported element type '{}'", element_type),
    }
}

fn parse_element_entity<
    'a,
    SizeT: Unsigned + Integer + num::ToPrimitive + Hash + Copy,
    IntT: Signed + Integer + num::ToPrimitive,
    SizeTParser,
    IntParser,
    E: ParseError<&'a [u8]>,
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementEntity<SizeT, IntT>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], SizeT, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], IntT, E>,
{
    let (input, entity_dim) = int_parser(input)?;
    let (input, entity_tag) = int_parser(input)?;
    let (input, element_type) = int_parser(input)?;
    let (input, num_elements_in_block) = size_t_parser(input)?;
    let num_elements_in_block = num_elements_in_block.to_usize().unwrap();

    let num_nodes = nodes_per_element(element_type.to_i32().unwrap());

    let parse_element = |input| {
        let (input, element_tag) = size_t_parser(input)?;

        let mut input = input;
        let mut node_tags = Vec::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            let (input_, node_tag) = size_t_parser(input)?;
            node_tags.push(node_tag);
            input = input_;
        }

        Ok((
            input,
            Element {
                element_tag,
                nodes: node_tags,
            },
        ))
    };

    let (input, elements) = count(parse_element, num_elements_in_block)(input)?;

    let element_tags = if sparse_tags {
        Some(
            elements
                .iter()
                .enumerate()
                .map(|(i, ele)| (ele.element_tag, i))
                .collect::<HashMap<_, _>>(),
        )
    } else {
        None
    };

    Ok((
        input,
        ElementEntity {
            entity_dim,
            entity_tag,
            element_type,
            element_tags,
            elements,
        },
    ))
}

fn parse_element_section<'a>(
    header: &MshHeader,
    input: &'a [u8],
) -> IResult<&'a [u8], Elements<usize, i32>, VerboseError<&'a [u8]>> {
    let size_t_parser = num_parsers::uint_parser::<usize, _>(header.size_t_size, header.endianness);

    let (input, num_entity_blocks) = size_t_parser(input)?;
    let (input, num_elements) = size_t_parser(input)?;
    let (input, min_element_tag) = size_t_parser(input)?;
    let (input, max_element_tag) = size_t_parser(input)?;

    let int_parser = num_parsers::int_parser::<i32, _>(header.int_size, header.endianness);

    let sparse_tags = if min_element_tag == 0 {
        panic!("Element tag 0 is reserved for internal use");
    } else if max_element_tag - min_element_tag > num_elements - 1 {
        true
    } else {
        false
    };

    let (input, element_entity_blocks) = count(
        |i| parse_element_entity(size_t_parser, int_parser, sparse_tags, i),
        num_entity_blocks,
    )(input)?;

    Ok((
        input,
        Elements {
            min_node_tag: min_element_tag,
            max_node_tag: max_element_tag,
            element_entities: element_entity_blocks,
        },
    ))
}

pub fn parse_bytes<'a>(
    input: &'a [u8],
) -> IResult<&'a [u8], MshFile<usize, i32, f64>, VerboseError<&'a [u8]>> {
    let (input, header) = parsers::parse_delimited_block(
        terminated(tag("$MeshFormat"), br),
        terminated(tag("$EndMeshFormat"), br),
        parse_header_section,
    )(input)?;

    // TODO: Support arbitrary order and repetition of blocks, support unrecognized headers
    // To allow this, headers have to be recognized

    let (input, entities) = parsers::parse_delimited_block(
        terminated(tag("$Entities"), br),
        terminated(tag("$EndEntities"), br),
        |i| parse_entity_section(&header, i),
    )(input)?;

    let (input, nodes) = parsers::parse_delimited_block(
        terminated(tag("$Nodes"), br),
        terminated(tag("$EndNodes"), br),
        |i| parse_node_section(&header, i),
    )(input)?;

    let (input, elements) = parsers::parse_delimited_block(
        terminated(tag("$Elements"), br),
        terminated(tag("$EndElements"), br),
        |i| parse_element_section(&header, i),
    )(input)?;

    Ok((
        input,
        MshFile {
            header,
            data: MshData {
                entities,
                nodes,
                elements,
            },
        },
    ))
}

pub fn parses(msh: &[u8]) -> bool {
    match parse_bytes(msh) {
        Ok((_, mesh)) => {
            println!("Successfully parsed:");
            println!("{:?}", mesh);
            true
        }
        Err(err) => {
            println!("{:?}", err);
            false
        }
    }
}
