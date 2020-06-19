#[macro_use]
mod utils;

use crate::utils::*;

#[test]
fn test_simple_ascii_file_error() {
    let circle_2d = read_test_mesh("circle_2d_error.msh");
    intended_error_output!(assert!(!msh_parses(&circle_2d)));
}

#[test]
fn test_invalid_section() {
    let msh = "\
$MeshFormat
4.1 0 8
$EndMeshFormat
$Comment
$EndComment
Hello

";
    intended_error_output!(assert!(!msh_parses(msh.as_bytes())));
}

#[test]
fn test_unsupported_msh_version_ascii() {
    let msh = "\
$MeshFormat
27.1 0 8
$EndMeshFormat

";
    intended_error_output!(assert!(!msh_parses(msh.as_bytes())));
}

#[test]
fn test_old_msh_version_bin() {
    let msh = read_test_mesh("old_msh_version.msh");
    intended_error_output!(assert!(!msh_parses(&msh)));
}
