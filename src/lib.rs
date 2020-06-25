//! Parser for Gmsh mesh files using the MSH file format version 4.1
//!
//! The library supports parsing ASCII and binary encoded MSH files adhering to the MSH file format
//! version 4.1 as specified in the [Gmsh documention](http://gmsh.info/doc/texinfo/gmsh.html#MSH-file-format).
//!
//! ```
//! use std::error::Error;
//! use std::fs;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     // Try to read and parse a MSH file
//!     let msh_bytes = fs::read("tests/data/sphere_coarse.msh")?;
//!     let parser_result = mshio::parse_msh_bytes(msh_bytes.as_slice());
//!
//!     // Note that the a parser error cannot be propagated directly using the ?-operator, as it
//!     // contains a reference into the u8 slice where the error occurred.
//!     let msh = parser_result.map_err(|e| format!("Error while parsing:\n{}", e))?;
//!     assert_eq!(msh.total_element_count(), 891);
//!
//!     Ok(())
//! }
//! ```
//!
//! If parsing was successful, the [`parse_msh_bytes`](fn.parse_msh_bytes.html) function returns a
//! [`MshFile`](mshfile/struct.MshFile.html) instance. The structure of `MshFile` closely mirrors
//! the MSH format specification. For example the `MeshData` associated to a `MshFile` may contain an
//! optional [`Elements`](mshfile/struct.Elements.html) section. This `Elements` section can contain
//! an arbitray number of [`ElementBlock`](mshfile/struct.ElementBlock.html) instances, where each
//! `ElementBlock` only contains elements of the same type and dimension.
//!
//! Currently, only the following sections of MSH files are actually parsed: `Entities`, `Nodes`,
//! `Elements`. All other sections are silently ignored, if they follow the pattern of being
//! delimited by `$SectionName` and `$EndSectionName` (in accordance to the MSH format specification).
//!
//! Note that the actual values are not checked for consistency beyond what is defined in the MSH format specification.
//! This means, that a parsed element may refer to node indices that are not present in the node section (if the MSH file already contains
//! such an inconsistency). In the future, utility functions may be added to check this.
//!
//! Although the `MshFile` struct and all related structs are generic over their value types,
//! the `parse_msh_bytes` function enforces the usage of `u64`, `i32` and `f64` as output value types 
//! corresponding to the MSH input value types `size_t`, `int` and `double`
//! (of course `size_t` values will still be parsed as having the size specified in the file header).
//! We did not encounter MSH files using different types (e.g. 64 bit integers or 32 bit floats) and therefore cannot test it. 
//! In addition, the MSH format specification does not specify the size of the float and integer types.
//! If the user desires narrowing conversions, they should be performed manually after parsing the file.
//!
//! Note that when loading collections of elements/nodes and other entities, the parser checks if
//! the number of these objects can be represented in the system's `usize` type. If this is not the
//! case it returns an error as they cannot be stored in a `Vec` in this case.
//!

use std::convert::{TryFrom, TryInto};

use nom::bytes::complete::tag;
use nom::character::complete::{alpha0, char};
use nom::combinator::peek;
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;

/// Error handling components of the parser
#[allow(unused)]
pub mod error;
/// Contains all types that are used to represent the structure of parsed MSH files
///
/// The central type is [`MshFile`](struct.MshFile.html) which contains the whole structure of the
/// parsed mesh.
pub mod mshfile;
/// Parser utility functions used by this MSH parser (may be private in the future)
pub mod parsers;

/// Error type returned by the MSH parser if parsing fails without panic
pub use error::MshParserError;
/// Re-exports all types that are used to represent the structure of an MSH file
pub use mshfile::*;

use crate::error::{make_error, MapMshError, MshParserErrorKind};
use error::{always_error, context};
use parsers::{br, take_sp};
use parsers::{
    parse_element_section, parse_entity_section, parse_header_section, parse_node_section,
};

// TODO: Error instead of panic on num_parser construction if size of the data type is not supported
// TODO: Reconsider naming of the MshUsizeT etc. and num parser trait names, make them consistent
// TODO: Doc strings for the new num_parser trait interface

// TODO: Make section parsers generic over data types (i.e. don't mandate f64, u64, i32)
// TODO: Unify element and node section parsing
//  (e.g. a single section parser, then per section type one header and one content parser)
// TODO: Unify entity parsing (currently, point parsers and the curve/surface/volume parsers are separate)

// TODO: Implement parser for physical groups
// TODO: Log in the MeshData struct which unknown sections were ignored
// TODO: Add more .context() calls/more specialized errors
// TODO: Replace remaining unimplemented!/expect calls with errors

// TODO: Test the float values parsed from a binary MSH file
// TODO: Add tests of errors in node section
// TODO: Add tests of errors in entity section
// TODO: Add tests that try to parse a mesh with u64 indices to u32

/// Try to parse a MshFile from a slice of bytes
///
/// The input  can be the content of an ASCII or binary encoded MSH file of file format version 4.1.
impl<'a> TryFrom<&'a [u8]> for MshFile<u64, i32, f64> {
    type Error = MshParserError<&'a [u8]>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match private_parse_msh_bytes(value) {
            Ok((_, file)) => Ok(file),
            Err(e) => Err(e.into()),
        }
    }
}

/// Try to parse a [`MshFile`](mshfile/struct.MshFile.html) from a slice of bytes
///
/// The input can be the content of an ASCII or binary encoded MSH file of file format version 4.1.
pub fn parse_msh_bytes<'a>(
    input: &'a [u8],
) -> Result<MshFile<u64, i32, f64>, MshParserError<&'a [u8]>> {
    input.try_into()
}

fn private_parse_msh_bytes<'a>(
    input: &'a [u8],
) -> IResult<&'a [u8], MshFile<u64, i32, f64>, MshParserError<&'a [u8]>> {
    let (input, (header, parsers)) = context(
        "MSH file header section",
        parsers::parse_delimited_block(
            terminated(tag("$MeshFormat"), br),
            terminated(tag("$EndMeshFormat"), br),
            context("MSH format header content", parse_header_section),
        ),
    )(input)?;

    // Closure to detect a line with a section start tag
    let section_detected = |start_tag, input| {
        peek::<_, _, (), _>(delimited(take_sp, tag(start_tag), br))(input).is_ok()
    };

    // Macro to apply a parser to a section delimited by start and end tags
    macro_rules! parse_section {
        ($start_tag:expr, $end_tag:expr, $parser:expr, $input:expr) => {{
            delimited(
                delimited(take_sp, tag($start_tag), br),
                $parser,
                delimited(take_sp, tag($end_tag), take_sp),
            )($input)
        }};
    }

    let mut entity_sections = Vec::new();
    let mut node_sections = Vec::new();
    let mut element_sections = Vec::new();

    let mut input = input;

    // Loop over all sections of the mesh file
    while !parsers::eof::<_, ()>(input).is_ok() {
        // Check for entity section
        if section_detected("$Entities", input) {
            let (input_, entities) = parse_section!(
                "$Entities",
                "$EndEntities",
                |i| context("entity section", parse_entity_section(&parsers))(i),
                input
            )?;

            entity_sections.push(entities);
            input = input_;
        }
        // Check for node section
        else if section_detected("$Nodes", input) {
            let (input_, nodes) = parse_section!(
                "$Nodes",
                "$EndNodes",
                |i| context("node section", parse_node_section(&parsers))(i),
                input
            )?;

            node_sections.push(nodes);
            input = input_;
        }
        // Check for elements section
        else if section_detected("$Elements", input) {
            let (input_, elements) = parse_section!(
                "$Elements",
                "$EndElements",
                |i| context("element section", parse_element_section(&parsers))(i),
                input
            )?;

            element_sections.push(elements);
            input = input_;
        }
        // Check for unknown section (gets ignored)
        else if let Ok((input_, section_header)) =
            peek::<_, _, (), _>(preceded(take_sp, delimited(char('$'), alpha0, br)))(input)
        {
            let section_header = String::from_utf8_lossy(section_header);
            let section_start_tag = format!("${}", section_header);
            let section_end_tag = format!("$End{}", section_header);

            let (input_, _) = parsers::delimited_block(
                delimited(take_sp, tag(&section_start_tag[..]), br),
                delimited(take_sp, tag(&section_end_tag[..]), take_sp),
            )(input_)?;

            input = input_;
        }
        // Check for invalid lines
        else {
            return always_error(MshParserErrorKind::InvalidSectionHeader)(input);
        }
    }

    // TODO: Replace the unimplemented! calls with errors

    let entities = match entity_sections.len() {
        1 => Some(entity_sections.remove(0)),
        0 => None,
        _ => {
            return Err(make_error(input, MshParserErrorKind::Unimplemented)
                .with_context(input, "Multiple entity sections found in the MSH file, this cannot be handled at the moment."))
        }
    };

    let nodes = match node_sections.len() {
        1 => Some(node_sections.remove(0)),
        0 => None,
        _ => return Err(make_error(input, MshParserErrorKind::Unimplemented)
            .with_context(input, "Multiple node sections found in the MSH file, this cannot be handled at the moment.")),
    };

    let elements = match element_sections.len() {
        1 => Some(element_sections.remove(0)),
        0 => None,
        _ => return Err(make_error(input, MshParserErrorKind::Unimplemented)
            .with_context(input, "Multiple element sections found in the MSH file, this cannot be handled at the moment.")),
    };

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
