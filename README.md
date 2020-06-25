# mshio

[![On crates.io](https://img.shields.io/crates/v/mshio)](https://crates.io/crates/mshio)
[![On docs.rs](https://docs.rs/mshio/badge.svg)](https://docs.rs/mshio/)
[![Build Status](https://github.com/w1th0utnam3/mshio/workflows/CI/badge.svg)](https://github.com/w1th0utnam3/mshio/actions)

Parser library for the Gmsh MSH file format (version 4.1)

The library supports parsing ASCII and binary encoded MSH files adhering to the MSH file format
version 4.1 as specified in the [Gmsh documention](http://gmsh.info/doc/texinfo/gmsh.html#MSH-file-format).

```rust
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Try to read and parse a MSH file
    let msh_bytes = fs::read("tests/data/sphere_coarse.msh")?;
    let parser_result = mshio::parse_msh_bytes(msh_bytes.as_slice());

    // Note that the a parser error cannot be propagated directly using the ?-operator, as it
    // contains a reference into the u8 slice where the error occurred.
    let msh = parser_result.map_err(|e| format!("Error while parsing:\n{}", e))?;
    assert_eq!(msh.total_element_count(), 891);

    Ok(())
}
```

If parsing was successful, the [`parse_msh_bytes`](https://docs.rs/mshio/latest/mshio/fn.parse_msh_bytes.html) function returns a
[`MshFile`](https://docs.rs/mshio/latest/mshio/mshfile/struct.MshFile.html) instance. The structure of `MshFile` closely mirrors
the MSH format specification. For example the `MeshData` associated to a `MshFile` may contain an
optional [`Elements`](https://docs.rs/mshio/latest/mshio/mshfile/struct.Elements.html) section. This `Elements` section can contain
an arbitray number of [`ElementBlock`](https://docs.rs/mshio/latest/mshio/mshfile/struct.ElementBlock.html) instances, where each
`ElementBlock` only contains elements of the same type and dimension.

Currently, only the following sections of MSH files are actually parsed: `Entities`, `Nodes`,
`Elements`. All other sections are silently ignored, if they follow the pattern of being
delimited by `$SectionName` and `$EndSectionName` (in accordance to the MSH format specification).

Note that the actual values are not checked for consistency beyond what is defined in the MSH format specification.
This means, that a parsed element may refer to node indices that are not present in the node section (if the MSH file already contains
such an inconsistency). In the future, utility functions may be added to check this.

Although the `MshFile` struct and all related structs are generic over their value types,
the `parse_msh_bytes` function enforces the usage of `u64`, `i32` and `f64` as output value types 
corresponding to the MSH input value types `size_t`, `int` and `double`
(of course `size_t` values will still be parsed as having the size specified in the file header).
We did not encounter MSH files using different types (e.g. 64 bit integers or 32 bit floats) and therefore cannot test it. 
In addition, the MSH format specification does not specify the size of the float and integer types.
If the user desires narrowing conversions, they should be performed manually after parsing the file.

Note that when loading collections of elements/nodes and other entities, the parser checks if
the number of these objects can be represented in the system's `usize` type. If this is not the
case it returns an error as they cannot be stored in a `Vec` in this case.

**What works already?**
 - Parsing of ASCII and binary (big/little endian) MSH files.
 - Parsing of the `Entities`, `Nodes`, `Elements` sections.
 - Supports all element types with fixed numbers of nodes that are currently supported by Gmsh.

**Issues**
 - The library contains some remaining unnecessary `unimplemented!`/`.expect` calls that should be replaced by errors.
   But these are very few and almost all error causes result in actual `Err` return values instead.
 - Unsupported sections are silently ignored. In the future, the `MshData` should store a list containing names of the ignored sections for convenience/debugging.
 - The MSH format specification allows to have multiple sections of the same type. Currently, parsing such a MSH file results in an error. 
   Joining them would require more logic in the parser, but it might also be ok to just store all sections of the same type in a `Vec`.
   We did decide on a solution as we do not have real world example files to test this with.
 - The library needs more internal code documentation in some parts and the parsers can probably be simplified.
 - More test cases for specific error cases are needed.

**Future work**
 - Writing of MSH files is currently not supported and is very low on our priority list, as we do not have a use case. Feel free contribute!
 - Parsing of other MSH sections. Again, we do not have a use case for this at the moment.
 - Utility functions that check for inconsistencies or that help to convert data into more common mesh representations.
