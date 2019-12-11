use std::collections::HashMap;
use std::hash::Hash;

use nom::error::ParseError;
use nom::multi::count;
use nom::IResult;

use num::{Integer, Signed, Unsigned};

use crate::mshfile::{Element, ElementEntity, Elements, MshHeader};
use crate::parsers::num_parsers;

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
        22 => 12,
        23 => 15,
        24 => 15,
        25 => 21,
        26 => 4,
        27 => 5,
        28 => 6,
        29 => 20,
        _ => unimplemented!("Unsupported element type '{}'", element_type),
    }
}

pub(crate) fn parse_element_section<'a, E: ParseError<&'a [u8]>>(
    header: &MshHeader,
    input: &'a [u8],
) -> IResult<&'a [u8], Elements<usize, i32>, E> {
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
