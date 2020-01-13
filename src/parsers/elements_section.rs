use std::collections::HashMap;

use nom::error::{ErrorKind, ParseError};
use nom::multi::count;
use nom::IResult;
use num_traits::FromPrimitive;

use crate::error::{error_strings, with_context};
use crate::mshfile::{Element, ElementBlock, ElementType, Elements, MshHeader, MshIntT, MshUsizeT};
use crate::parsers::num_parsers;

pub(crate) fn parse_element_section<'a, E>(
    header: &MshHeader,
    input: &'a [u8],
) -> IResult<&'a [u8], Elements<usize, i32>, E>
where
    E: ParseError<&'a [u8]>,
{
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
            num_elements,
            min_element_tag,
            max_element_tag,
            element_entities: element_entity_blocks,
        },
    ))
}

fn parse_element_type<'a, I, IntParser, E>(
    int_parser: IntParser,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementType, E>
where
    I: MshIntT,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    E: ParseError<&'a [u8]>,
{
    let (input, element_type_raw) = int_parser(input)?;
    let element_type_raw = element_type_raw
        .to_i32()
        .expect("Element type does not fit into i32");

    match ElementType::from_i32(element_type_raw) {
        Some(element_type) => Ok((input, element_type)),
        None => with_context(error_strings::ELEMENT_UNKNOWN, ErrorKind::Tag)(input),
    }
}

fn parse_element_entity<'a, U, I, SizeTParser, IntParser, E>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementBlock<U, I>, E>
where
    U: MshUsizeT,
    I: MshIntT,
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    E: ParseError<&'a [u8]>,
{
    let (input, entity_dim) = int_parser(input)?;
    let (input, entity_tag) = int_parser(input)?;
    let (input, element_type) = parse_element_type(int_parser, input)?;
    let (input, num_elements_in_block) = size_t_parser(input)?;

    // Convert number of elements to usize for convenience
    let num_elements_in_block = num_elements_in_block
        .to_usize()
        .expect("Number of elements in block do not fit in usize");

    // Try to get the number of nodes of the elements
    let num_nodes = match element_type.nodes() {
        Ok(v) => v,
        Err(_) => {
            return with_context(error_strings::ELEMENT_NUM_NODES_UNKNOWN, ErrorKind::Tag)(input);
        }
    };

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
                .map(|(i, ele)| (ele.element_tag.clone(), i))
                .collect::<HashMap<_, _>>(),
        )
    } else {
        None
    };

    Ok((
        input,
        ElementBlock {
            entity_dim,
            entity_tag,
            element_type,
            element_tags,
            elements,
        },
    ))
}
