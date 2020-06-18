use std::collections::HashMap;

use nom::multi::count;
use nom::IResult;
use num_traits::FromPrimitive;

use crate::error::{error, MshParserCompatibleError, MshParserError, MshParserErrorKind};
use crate::mshfile::{Element, ElementBlock, ElementType, Elements, MshHeader, MshIntT, MshUsizeT};
use crate::parsers::num_parsers;

pub(crate) fn parse_element_section<'a, 'b: 'a>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Elements<usize, i32>, MshParserError<&'b [u8]>> {
    let header = header.clone();
    move |input| {
        let size_t_parser =
            num_parsers::uint_parser::<usize, _>(header.size_t_size, header.endianness);

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
                element_blocks: element_entity_blocks,
            },
        ))
    }
}

fn parse_element_type<'a, I, IntParser, E>(
    int_parser: IntParser,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementType, MshParserError<&'a [u8]>>
where
    I: MshIntT,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    E: MshParserCompatibleError<&'a [u8]>,
    nom::Err<MshParserError<&'a [u8]>>: From<nom::Err<E>>,
{
    let (input_new, element_type_raw) = int_parser(input)?;
    let element_type_raw = element_type_raw.to_i32().ok_or_else(|| {
        MshParserError::from_error_kind(input, MshParserErrorKind::ElementUnknown).into_nom_err()
    })?;

    match ElementType::from_i32(element_type_raw) {
        Some(element_type) => Ok((input_new, element_type)),
        None => error(MshParserErrorKind::ElementUnknown)(input),
    }
}

fn parse_element_entity<'a, U, I, SizeTParser, IntParser, E>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementBlock<U, I>, MshParserError<&'a [u8]>>
where
    U: MshUsizeT,
    I: MshIntT,
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    E: MshParserCompatibleError<&'a [u8]>,
    nom::Err<MshParserError<&'a [u8]>>: From<nom::Err<E>>,
{
    let (input, entity_dim) = int_parser(input)?;
    let (input, entity_tag) = int_parser(input)?;
    let (input, element_type) = parse_element_type(int_parser, input)?;
    let (input_new, num_elements_in_block) = size_t_parser(input)?;

    // Try to convert number of elements to usize
    let num_elements_in_block = num_elements_in_block.to_usize().ok_or_else(|| {
        MshParserError::from_error_kind(input.clone(), MshParserErrorKind::TooManyEntities)
            .with_context(
                input.clone(),
                format!(
                    "The current block contains {:?} entities",
                    num_elements_in_block
                ),
            )
            .into_nom_err()
    })?;

    // Try to get the number of nodes per element
    let num_nodes = match element_type.nodes() {
        Ok(v) => v,
        Err(_) => {
            return error(MshParserErrorKind::ElementNumNodesUnknown)(input);
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

    let (input, elements) = count(parse_element, num_elements_in_block)(input_new)?;

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
