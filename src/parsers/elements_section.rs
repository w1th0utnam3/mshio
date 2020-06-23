use std::collections::HashMap;

use nom::IResult;
use num::traits::FromPrimitive;

use crate::error::{
    always_error, context, make_error, MapMshError, MshParserError, MshParserErrorKind,
};
use crate::mshfile::{Element, ElementBlock, ElementType, Elements, MshHeader, MshIntT, MshUsizeT};
use crate::parsers::num_parsers;
use crate::parsers::{count_indexed, verify_or};

struct ElementSectionHeader<U: MshUsizeT> {
    num_entity_blocks: usize,
    num_elements: U,
    min_element_tag: U,
    max_element_tag: U,
}

pub(crate) fn parse_element_section<'a, 'b: 'a>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Elements<u64, i32>, MshParserError<&'b [u8]>> {
    let header = header.clone();
    move |input| {
        let int_parser = num_parsers::int_parser::<i32>(header.int_size, header.endianness);
        let size_t_parser = num_parsers::uint_parser::<u64>(header.size_t_size, header.endianness);

        // Parse the section header
        let (input, element_section_header) = context("element section header", |input| {
            parse_element_section_header(&size_t_parser, input)
        })(input)?;

        let ElementSectionHeader {
            num_entity_blocks,
            num_elements,
            min_element_tag,
            max_element_tag,
        } = element_section_header;

        let sparse_tags = if max_element_tag - min_element_tag > num_elements - 1 {
            true
        } else {
            false
        };

        // Parse the individual element entity blocks
        let (input, element_entity_blocks) = count_indexed(
            |index, input| {
                parse_element_entity(&size_t_parser, &int_parser, sparse_tags, input)
                    .with_context_from(input, || {
                        format!(
                            "element entity block ({} of {})",
                            index + 1,
                            num_entity_blocks
                        )
                    })
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

fn parse_element_section_header<'a, SizeTParser>(
    size_t_parser: SizeTParser,
    input: &'a [u8],
) -> IResult<&'a [u8], ElementSectionHeader<u64>, MshParserError<&'a [u8]>>
where
    SizeTParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], u64, MshParserError<&'b [u8]>>,
{
    let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

    let (input, num_entity_blocks) =
        context("number of element entity blocks", &to_usize_parser)(input)?;
    let (input, num_elements) = context("total number of elements", &size_t_parser)(input)?;
    let (input, min_element_tag) = context(
        "min element tag",
        verify_or(
            &size_t_parser,
            |&tag| tag != 0,
            context(
                "Element tag 0 is reserved for internal use",
                always_error(MshParserErrorKind::InvalidTag),
            ),
        ),
    )(input)?;
    let (input, max_element_tag) = context(
        "max element tag",
        verify_or(
            &size_t_parser,
            |&max_tag| max_tag >= min_element_tag,
            context(
                "The maximum element tag has to be larger or equal to the minimum element tag",
                always_error(MshParserErrorKind::InvalidTag),
            ),
        ),
    )(input)?;

    Ok((
        input,
        ElementSectionHeader {
            num_entity_blocks,
            num_elements,
            min_element_tag,
            max_element_tag,
        },
    ))
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
    SizeTParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], U, MshParserError<&'b [u8]>>,
    IntParser: for<'b> Fn(&'b [u8]) -> IResult<&'b [u8], I, MshParserError<&'b [u8]>>,
{
    let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

    let (input, entity_dim) = context("entity dimension", &int_parser)(input)?;
    let (input, entity_tag) = context("entity tag", &int_parser)(input)?;
    let (input, element_type) =
        context("element type", move |i| parse_element_type(&int_parser, i))(input)?;
    let (input_new, num_elements_in_block) =
        context("number of elements in element block", to_usize_parser)(input)?;

    // Try to get the number of nodes per element
    let num_nodes_per_element = element_type.nodes().map_err(|_| {
        make_error(input, MshParserErrorKind::Unimplemented).with_context(
            input,
            "An element type encountered in the MSH file does not have a known number of nodes.",
        )
    })?;

    // Parse every element definition
    let (input, elements) = count_indexed(
        |index, input| {
            parse_element(&size_t_parser, num_nodes_per_element, input)
                .with_error(input, MshParserErrorKind::InvalidElementDefinition)
                .with_context_from(input, || {
                    format!(
                        "element definition ({} of {})",
                        index + 1,
                        num_elements_in_block
                    )
                })
        },
        num_elements_in_block,
    )(input_new)?;

    // Extract the element tags -> index mapping if tags are sparse
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
    let element_type_raw = element_type_raw
        .to_i32()
        .ok_or_else(|| make_error(input, MshParserErrorKind::UnknownElement))?;

    // Try to construct a element type variant from the i32 value
    let element_type = ElementType::from_i32(element_type_raw).ok_or_else(|| {
        make_error(input, MshParserErrorKind::UnknownElement)
            .with_context_from(input, || format!("value {}", element_type_raw))
    })?;

    Ok((input_new, element_type))
}

fn parse_element<'a, U, SizeTParser>(
    size_t_parser: SizeTParser,
    num_nodes_per_element: usize,
    input: &'a [u8],
) -> IResult<&'a [u8], Element<U>, MshParserError<&'a [u8]>>
where
    U: MshUsizeT,
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
{
    let (input, element_tag) = size_t_parser(input)?;

    let mut input = input;
    let mut node_tags = Vec::with_capacity(num_nodes_per_element);
    for _ in 0..num_nodes_per_element {
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
}
