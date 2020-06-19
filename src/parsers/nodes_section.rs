use std::collections::HashMap;

use nom::error::ParseError;
use nom::multi::count;
use nom::IResult;

use crate::error::{context, error, MapMshError, MshParserError, MshParserErrorKind};
use crate::mshfile::{MshFloatT, MshHeader, MshIntT, MshUsizeT, Node, NodeBlock, Nodes};
use crate::parsers::general_parsers::{count_indexed, verify_or};
use crate::parsers::num_parsers;

struct NodeSectionHeader<U: MshUsizeT> {
    num_entity_blocks: usize,
    num_nodes: U,
    min_node_tag: U,
    max_node_tag: U,
}

pub(crate) fn parse_node_section<'a, 'b: 'a>(
    header: &'a MshHeader,
) -> impl Fn(&'b [u8]) -> IResult<&'b [u8], Nodes<u64, i32, f64>, MshParserError<&'b [u8]>> {
    let header = header.clone();
    move |input| {
        let size_t_parser = num_parsers::uint_parser::<u64>(header.size_t_size, header.endianness);
        let int_parser = num_parsers::int_parser::<i32>(header.int_size, header.endianness);
        let double_parser = num_parsers::float_parser::<f64>(header.float_size, header.endianness);

        // Parse the section header
        let (input, node_section_header) = context("Node section header", |input| {
            parse_node_section_header(&size_t_parser, input)
        })(input)?;

        let NodeSectionHeader {
            num_entity_blocks,
            num_nodes,
            min_node_tag,
            max_node_tag,
        } = node_section_header;

        let sparse_tags = if max_node_tag - min_node_tag > num_nodes - 1 {
            true
        } else {
            false
        };

        // Parse the individual node entity blocks
        let (input, node_entity_blocks) = count_indexed(
            |index, input| {
                parse_node_entity(
                    &size_t_parser,
                    &int_parser,
                    double_parser,
                    sparse_tags,
                    input,
                )
                .with_context_from(input, || {
                    format!("node entity block ({} of {})", index + 1, num_entity_blocks)
                })
            },
            num_entity_blocks,
        )(input)?;

        Ok((
            input,
            Nodes {
                num_nodes,
                min_node_tag,
                max_node_tag,
                node_blocks: node_entity_blocks,
            },
        ))
    }
}

fn parse_node_section_header<'a, SizeTParser>(
    size_t_parser: SizeTParser,
    input: &'a [u8],
) -> IResult<&'a [u8], NodeSectionHeader<u64>, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], u64, MshParserError<&'a [u8]>>,
{
    let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

    let (input, num_entity_blocks) =
        context("number of node entity blocks", &to_usize_parser)(input)?;
    let (input, num_nodes) = context("total number of elements", &size_t_parser)(input)?;
    let (input, min_node_tag) = context(
        "min node tag",
        verify_or(
            &size_t_parser,
            |tag| *tag != 0,
            context(
                "Node tag 0 is reserved for internal use",
                error(MshParserErrorKind::InvalidTag),
            ),
        ),
    )(input)?;
    let (input, max_node_tag) = context(
        "max node tag",
        verify_or(
            &size_t_parser,
            |max_tag| *max_tag >= min_node_tag,
            context(
                "The maximum node tag has to be larger or equal to the minimum node tag",
                error(MshParserErrorKind::InvalidTag),
            ),
        ),
    )(input)?;

    Ok((
        input,
        NodeSectionHeader {
            num_entity_blocks,
            num_nodes,
            min_node_tag,
            max_node_tag,
        },
    ))
}

fn parse_node_entity<
    'a,
    U: MshUsizeT,
    I: MshIntT,
    F: MshFloatT,
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
) -> IResult<&'a [u8], NodeBlock<U, I, F>, E>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, E>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, E>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, E>,
{
    let (input, entity_dim) = int_parser(input)?;
    let (input, entity_tag) = int_parser(input)?;
    let (input, parametric) = int_parser(input)?;
    let (input, num_nodes_in_block) = size_t_parser(input)?;
    let num_nodes_in_block = num_nodes_in_block.to_usize().unwrap();

    let parametric = if parametric == I::zero() {
        false
    } else if parametric == I::one() {
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
        NodeBlock {
            entity_dim,
            entity_tag,
            parametric,
            node_tags,
            nodes,
            parametric_nodes: None,
        },
    ))
}
