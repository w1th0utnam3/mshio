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
//!     let msh_bytes = fs::read("tests/sphere_coarse.msh")?;
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
//! the file format definition. For example the `MeshData` associated to a `MshFile` may contain an
//! optional [`Elements`](mshfile/struct.Elements.html) section. This `Elements` section can contain
//! an arbitray number of [`ElementBlock`](mshfile/struct.ElementBlock.html) instances, where each
//! `ElementBlock` only contains elements of the same type and dimension.
//!
//! Currently, only the following sections of MSH files are actually parsed: `Entities`, `Nodes`,
//! `Elements`. All other sections are silently ignored, if they follow the pattern of being
//! delimited by `$SectionName` and `$EndSectionName`.
//!
//! Although the `MshFile` struct and all related structs are generic over their float and integer
//! types, the `parse_msh_bytes` function enforces the usage of `f64`, `i32` and `usize` types as
//! we did not encounter MSH files with different types and cannot test it. The MSH format
//! documentation does not specify the size of the float and integer types.
//! Narrowing conversions should be performed manually by the user after parsing the file.
//!
//! Note that the `usize` type is used to index nodes and elements. If the system's `usize` type
//! is too small to hold the `size_t` type defined in the header of the MSH file, the parser
//! will return an error. This can be the case if a mesh written on a 64-bit machine is loaded on a
//! 32-bit machine. This might be fixed in a later release to allow to read such meshes as long
//! as the total number of elements/nodes in a block fits into `usize` (otherwise they cannot be
//! stored in a `Vec` anyway).
//!

use std::convert::{TryFrom, TryInto};
use std::str;

use nom::bytes::complete::tag;
use nom::character::complete::{alpha0, char};
use nom::combinator::peek;
use nom::error::{context, ErrorKind, ParseError, VerboseError};
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;

/// Error handling components of the parser
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

use error::{create_error, error_strings};
use parsers::{br, take_sp};
use parsers::{
    parse_element_section, parse_entity_section, parse_header_section, parse_node_section,
};

// TODO: Implement parser for physical groups
// TODO: Replace panics and unimplemented! calls with Err
// TODO: Add more context calls for all levels of parsers
// TODO: Review the passing of primitive parser functions as generic parameters (don't support Copy)

// TODO: Add proper enum variants for custom error
// TODO: Global static strings for error context
// TODO: Map static string error context back to error enum variants

// TODO: Allow parsing usize=u64 indexed meshes on usize=u32 machines and only return an error
//  if there are actually too many elements/nodes (because then Vec cannot hold them all)

/// Debug helper to view u8 slice as utf8 str and print it
#[allow(dead_code)]
fn print_u8(text: &str, input: &[u8]) {
    println!("{}: '{}'", text, String::from_utf8_lossy(input));
}

/// Try to parse a MshFile from a slice of bytes
///
/// The input  can be the content of an ASCII or binary encoded MSH file of file format version 4.1.
impl<'a> TryFrom<&'a [u8]> for MshFile<usize, i32, f64> {
    type Error = MshParserError<&'a [u8]>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match private_parse_msh_bytes::<VerboseError<_>>(value) {
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
) -> Result<MshFile<usize, i32, f64>, MshParserError<&'a [u8]>> {
    input.try_into()
}

fn private_parse_msh_bytes<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], MshFile<usize, i32, f64>, E> {
    let (input, header) = context(
        "MSH file header section",
        parsers::parse_delimited_block(
            terminated(tag("$MeshFormat"), br),
            terminated(tag("$EndMeshFormat"), br),
            context("MSH file header content", parse_header_section),
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
                |i| context("Entity section content", parse_entity_section(&header))(i),
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
                |i| context("Node section content", parse_node_section(&header))(i),
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
                |i| context("Element section content", parse_element_section(&header))(i),
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
            return create_error(error_strings::SECTION_HEADER_INVALID, ErrorKind::Tag)(input);
        }
    }

    let entities = match entity_sections.len() {
        1 => Some(entity_sections.remove(0)),
        0 => None,
        _ => unimplemented!("More than one entity section found"),
    };

    let nodes = match node_sections.len() {
        1 => Some(node_sections.remove(0)),
        0 => None,
        _ => unimplemented!("More than one node section found"),
    };

    let elements = match element_sections.len() {
        1 => Some(element_sections.remove(0)),
        0 => None,
        _ => unimplemented!("More than one element section found"),
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
