use std::collections::HashMap;

use nom::IResult;
use num::traits::FromPrimitive;

use crate::error::{context, error, MshParserError, MshParserErrorKind};
use crate::mshfile::{Element, ElementBlock, ElementType, Elements, MshHeader, MshIntT, MshUsizeT};
use crate::parsers::general_parsers::{count_indexed, verify_or};
use crate::parsers::num_parsers;

pub(crate) fn parse_element_section<'a, 'b: 'a>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Elements<u64, i32>, MshParserError<&'b [u8]>> {
    let header = header.clone();
    move |input| {
        let int_parser = num_parsers::int_parser::<i32>(header.int_size, header.endianness);
        let size_t_parser = num_parsers::uint_parser::<u64>(header.size_t_size, header.endianness);
        let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

        let (input, (num_entity_blocks, num_elements, min_element_tag, max_element_tag)) = context(
            "Element section header",
            |input| {
                let (input, num_entity_blocks) =
                    context("number of element entity blocks", &to_usize_parser)(input)?;
                let (input, num_elements) =
                    context("total number of elements", &size_t_parser)(input)?;
                let (input, min_element_tag) = context(
                    "min element tag",
                    verify_or(
                        &size_t_parser,
                        |tag| *tag != 0,
                        context(
                            "Element tag 0 is reserved for internal use",
                            error(MshParserErrorKind::InvalidTag),
                        ),
                    ),
                )(input)?;
                let (input, max_element_tag) = context(
                "max element tag",
                verify_or(
                    &size_t_parser,
                    |max_tag| *max_tag >= min_element_tag,
                    context(
                        "The maximum element tag has to be larger or equal to the minimum element tag",
                        error(MshParserErrorKind::InvalidTag),
                    ),
                ),
            )(input)?;

                Ok((
                    input,
                    (
                        num_entity_blocks,
                        num_elements,
                        min_element_tag,
                        max_element_tag,
                    ),
                ))
            },
        )(
            input
        )?;

        let sparse_tags = if max_element_tag - min_element_tag > num_elements - 1 {
            true
        } else {
            false
        };

        // Parse the individual element entity blocks
        let (input, element_entity_blocks) = count_indexed(
            |index, input| {
                context(
                    format!(
                        "element entity block ({} of {})",
                        index + 1,
                        num_entity_blocks
                    ),
                    |i| parse_element_entity(&size_t_parser, &int_parser, sparse_tags, i),
                )(input)
            },
            num_entity_blocks,
        )(input)?;

        // Return the element section content
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

fn parse_element_type<'a, I, IntParser>(
    int_parser: IntParser,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementType, MshParserError<&'a [u8]>>
where
    I: MshIntT,
    IntParser: Clone + Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
{
    // Read the raw integer representing the element type
    let (input_new, element_type_raw) = int_parser(input)?;

    // Try to convert it into i32 (because this is the underlying type of our enum)
    let element_type_raw = element_type_raw.to_i32().ok_or_else(|| {
        MshParserErrorKind::ElementUnknown
            .into_error(input)
            .into_nom_error()
    })?;

    // Try to construct a element type variant from the i32 value
    match ElementType::from_i32(element_type_raw) {
        Some(element_type) => Ok((input_new, element_type)),
        None => context(
            format!("value {}", element_type_raw),
            error(MshParserErrorKind::ElementUnknown),
        )(input),
    }
}

fn parse_element_entity<'a, U, I, SizeTParser, IntParser>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementBlock<U, I>, MshParserError<&'a [u8]>>
where
    U: MshUsizeT,
    I: MshIntT,
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
{
    let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

    let (input, entity_dim) = context("entity dimension", &int_parser)(input)?;
    let (input, entity_tag) = context("entity tag", &int_parser)(input)?;
    let (input, element_type) =
        context("element type", move |i| parse_element_type(&int_parser, i))(input)?;
    let (input_new, num_elements_in_block) =
        context("number of elements in element block", to_usize_parser)(input)?;

    // Try to get the number of nodes per element
    let num_nodes = match element_type.nodes() {
        Ok(v) => v,
        Err(_) => {
            // This can only happen if the .nodes() implementation is not in sync with the actual element types
            return error(MshParserErrorKind::ElementNumNodesUnknown)(input);
        }
    };

    // Closure to parse a single element definition in the block
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

    // Parse every element definition
    let (input, elements) = count_indexed(
        |index, input| {
            context(
                format!(
                    "element definition ({} of {})",
                    index + 1,
                    num_elements_in_block
                ),
                parse_element,
            )(input)
        },
        num_elements_in_block,
    )(input_new)?;

    // Extract the index -> element tags mapping if tags are sparse
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
