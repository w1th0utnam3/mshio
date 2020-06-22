# mshio

[![Build Status](https://github.com/w1th0utnam3/mshio/workflows/Build%20and%20run%20tests/badge.svg)](https://github.com/w1th0utnam3/mshio/actions)

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

If parsing was successful, the [`parse_msh_bytes`](fn.parse_msh_bytes.html) function returns a
[`MshFile`](mshfile/struct.MshFile.html) instance. The structure of `MshFile` closely mirrors
the file format definition. For example the `MeshData` associated to a `MshFile` may contain an
optional [`Elements`](mshfile/struct.Elements.html) section. This `Elements` section can contain
an arbitray number of [`ElementBlock`](mshfile/struct.ElementBlock.html) instances, where each
`ElementBlock` only contains elements of the same type and dimension.

Currently, only the following sections of MSH files are actually parsed: `Entities`, `Nodes`,
`Elements`. All other sections are silently ignored, if they follow the pattern of being
delimited by `$SectionName` and `$EndSectionName`.

Note that the mesh definition is not checked for consistency. This means, that a parsed element
may refer to node indices that are not present in the node section (if the MSH file contains
such an inconsistency). In the future, utitliy functions may be added to check this.

Although the `MshFile` struct and all related structs are generic over their float and integer
types, the `parse_msh_bytes` function enforces the usage of `f64`, `i32` and `u64` types as
we did not encounter MSH files with different types and cannot test it. The MSH format
documentation does not specify the size of the float and integer types.
Narrowing conversions should be performed manually by the user after parsing the file.

Note that when loading collections of elements/nodes and other entities, the parser checks if
the number of these objects can be represented in the system's `usize` type. If this is not the
case it returns an error as they cannot be stored in a `Vec` in this case.

**What is already implemented**
 - Parsing of ASCII and binary (big/little endian) MSH files.
 - Parsing of the `Entities`, `Nodes`, `Elements` sections.
 - Supports all element types currently supported by Gmsh.

**Issues**
 - The library contains some unnecessary `unimplemented!`/`.expect` calls that should be replaced by errors.
 - Unsupported sections are silently ignored. In the future, the `MshData` should store the names of the ignored sections for debugging.
 - The MSH format allows to have multiple sections of the same type, currently this results in an error. Joining them would require more logic, but it might also be ok to just store all sections of the same type in a `Vec`. Joining could be performed afterwards by utility functions. We do not have real world example files to test this with.
