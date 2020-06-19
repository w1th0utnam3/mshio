use std::collections::HashMap;

use nom::multi::count;
use nom::IResult;

use crate::error::{
    always_error, context, error, make_error, MapMshError, MshParserError, MshParserErrorKind,
};
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
                    &double_parser,
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
                always_error(MshParserErrorKind::InvalidTag),
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
                always_error(MshParserErrorKind::InvalidTag),
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
>(
    size_t_parser: SizeTParser,
    int_parser: IntParser,
    double_parser: FloatParser,
    sparse_tags: bool,
    input: &'a [u8],
) -> IResult<&'a [u8], NodeBlock<U, I, F>, MshParserError<&'a [u8]>>
where
    SizeTParser: Fn(&'a [u8]) -> IResult<&'a [u8], U, MshParserError<&'a [u8]>>,
    IntParser: Fn(&'a [u8]) -> IResult<&'a [u8], I, MshParserError<&'a [u8]>>,
    FloatParser: Fn(&'a [u8]) -> IResult<&'a [u8], F, MshParserError<&'a [u8]>>,
{
    let to_usize_parser = num_parsers::usize_parser(&size_t_parser);

    let (input, entity_dim) = context("entity dimension", &int_parser)(input)?;
    let (input, entity_tag) = context("entity tag", &int_parser)(input)?;
    let (input, parametric) = context(
        "parametric flag",
        verify_or(
            &int_parser,
            |p| *p == I::zero() || *p == I::one(),
            context(
                "Unsupported value for node block attribute 'parametric' (only 0 and 1 supported)",
                always_error(MshParserErrorKind::InvalidParameter),
            ),
        ),
    )(input)?;
    let (input, num_nodes_in_block) =
        context("number of nodes in element block", &to_usize_parser)(input)?;

    let parametric = parametric != I::zero();
    if parametric {
        return Err(make_error(input, MshParserErrorKind::Unimplemented)
            .with_context(input, "Parametric nodes are not supported yet"));
    }

    // Closure that parses all node tags
    let parse_all_node_tags = |input| {
        context(
            "node tags",
            count(
                error(MshParserErrorKind::InvalidTag, &size_t_parser),
                num_nodes_in_block,
            ),
        )(input)
    };

    // Parse the node tags
    let (input, node_tags) = if sparse_tags {
        let (input, node_tags) = parse_all_node_tags(input)?;

        // Collect the node tags into a map: tag -> index
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
        // If the node tags are not sparse, we still have to read them to advance in the file
        let (input, _) = parse_all_node_tags(input)?;
        (input, None)
    };

    // Closure that parse a single node coordinate tuple
    let parse_node = |input| {
        let (input, x) = context("x coordinate", &double_parser)(input)?;
        let (input, y) = context("y coordinate", &double_parser)(input)?;
        let (input, z) = context("z coordinate", &double_parser)(input)?;

        Ok((input, Node { x, y, z }))
    };

    // Parse node coordinates
    let (input, nodes) = context(
        "node coordinates",
        count(
            error(MshParserErrorKind::InvalidNodeDefinition, parse_node),
            num_nodes_in_block,
        ),
    )(input)?;

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
