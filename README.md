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
    let msh_bytes = fs::read("tests/sphere_coarse.msh")?;
    let parser_result = mshio::parse_msh_bytes(msh_bytes.as_slice());
    // Note that the a parser error cannot be propagated directly using the ?-operator, as it
    // contains a reference into the u8 slice where the error occurred.
    let msh = parser_result.map_err(|e| format!("Error while parsing:\n{}", e))?;
    assert_eq!(msh.total_element_count(), 891);
    Ok(())
}
```

If parsing was successful, the `parse_msh_bytes` function returns a
`MshFile` instance. The structure of the `MshFile` struct closely mirrors
the file format definition. For example the `MeshData` associated to a `MshFile` may contain an
optional `Elements` section. This `Elements` section can contain
an arbitray number of `ElementBlock` instances, where each
`ElementBlock` only contains elements of the same type and dimension.

Currently, only the following sections of MSH files are actually parsed: `Entities`, `Nodes`,
`Elements`. All other sections are silently ignored, if they follow the pattern of being
delimited by `$SectionName` and `$EndSectionName`.

Although the `MshFile` struct and all related structs are generic over their float and integer
types, the `parse_msh_bytes` function enforces the usage of `f64`, `i32` and `usize` types as
we did not encounter MSH files with different types and cannot test it. The MSH format
documentation does not specify the size of the float and integer types.
Narrowing conversions should be performed manually by the user after parsing the file.

Note that the `usize` type is used to index nodes and elements. If the system's `usize` type
is too small to hold the `size_t` type defined in the header of the MSH file, the parser
will return an error. This can be the case if a mesh written on a 64-bit machine is loaded on a
32-bit machine. This might be fixed in a later release to allow to read such meshes as long
as the total number of elements/nodes in a block fits into `usize` (otherwise they cannot be
stored in a `Vec` anyway).

**What is already implemented**
 - Parsing of ASCII and binary (big/little endian) MSH files.
 - Parsing of the `Entities`, `Nodes`, `Elements` sections.
 - Supports all element types currently supported by Gmsh.

**Issues**
 - The errors returned by the parser only wrap the raw `nom` parser errors, in the future there should be more specific error values.
 - A mesh indexed using `sizeof(size_t)==8` (64 bit) unsigned integers cannot be loaded on `sizeof(size_t)==4` (32 bit) machines.
 - The library contains some unnecessary `panic!`/`unimplemented!` calls that should be turned into errors.
 - Unsupported sections are silently ignored, maybe the `MshData` should store the names of the ignored sections for debugging.
 - The MSH format allows to have multiple sections of the same type, currently this results in a panic. Joining them would require more logic, but it might also be ok to just store all sections of the same type in a `Vec`. Joining could be performed afterwards by utility functions.
