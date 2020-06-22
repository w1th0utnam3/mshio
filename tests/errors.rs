#[macro_use]
mod utils;

use crate::utils::*;
use mshio::error::MshParserErrorKind;

/// Generates a test case that expects parsing of a MSH file given by string fails with the given error
macro_rules! simple_error_test {
    ($test_name:ident, $expected_error:expr, $mesh_string:expr) => {
        #[test]
        fn $test_name() {
            let msh = $mesh_string;

            let parsed_msh = mshio::parse_msh_bytes(msh.as_bytes());
            assert!(
                parsed_msh.is_err(),
                concat!(
                    "The test '",
                    stringify!($test_name),
                    "' expects that parsing the mesh results in an error"
                )
            );

            let error = parsed_msh.unwrap_err();
            intended_error_output!(test_invalid_element_type, print_error_report(&error));
            assert_eq!(
                error.first_msh_error(),
                Some($expected_error),
                concat!(
                    "The test '",
                    stringify!($test_name),
                    "' expects that parsing the mesh results in the error '",
                    stringify!($expected_error),
                    "'."
                )
            );
        }
    };
}

simple_error_test!(
    test_unsupported_msh_version_ascii,
    MshParserErrorKind::UnsupportedMshVersion,
    "\
$MeshFormat
27.1 0 8
$EndMeshFormat

"
);

#[test]
fn test_old_msh_version_bin() {
    let msh = read_test_mesh("old_msh_version.msh");
    intended_error_output!(test_old_msh_version_bin, assert!(!msh_parses(&msh)));
}

simple_error_test!(
    test_invalid_section,
    MshParserErrorKind::InvalidSectionHeader,
    "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Comment
$EndComment
Hello

"
);

simple_error_test!(
    test_invalid_element_type,
    MshParserErrorKind::UnknownElement,
    "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Elements
1 20 1 20
2 0 788 20
$EndElements\
"
);

simple_error_test!(
    test_wrong_element_amount,
    MshParserErrorKind::InvalidElementDefinition,
    "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Entities
0 0 1 0
0 -1 -1 0 1 1 0 0 0
$EndEntities
$Elements
1 20 1 20
2 0 3 20
1 1 9 13 12
$EndElements\
"
);

simple_error_test!(
    test_wrong_entity_amount,
    MshParserErrorKind::InvalidTag,
    "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Entities
0 0 2 0
0 -1 -1 0 1 1 0 0 0
$EndEntities\
"
);

simple_error_test!(
    test_wrong_bounding_tag_amount,
    MshParserErrorKind::InvalidTag,
    "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Entities
0 0 1 0
0 -1 -1 0 1 1 0 0 2
$EndEntities\
"
);
