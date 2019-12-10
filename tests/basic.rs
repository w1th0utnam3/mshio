#[test]
fn does_nothing() {
    assert!(true);
}

#[test]
fn simple_bin_file() {
    let circle_2d_bin = include_bytes!("circle_2d_bin.msh");
    assert!(mshio::parses(circle_2d_bin));
}

#[test]
fn simple_ascii_file() {
    let circle_2d = include_str!("circle_2d.msh");
    assert!(mshio::parses(circle_2d.as_bytes()));
}

#[test]
fn compare_simple_ascii_bin() {
    let circle_2d_bin_raw = include_bytes!("circle_2d_bin.msh");
    let circle_2d_raw = include_str!("circle_2d.msh");

    let (_, msh_bin) = mshio::parse_bytes(circle_2d_bin_raw).unwrap();
    let (_, msh_ascii) = mshio::parse_bytes(circle_2d_raw.as_bytes()).unwrap();

    // Headers differ, but data should be the same
    assert_eq!(msh_bin.data, msh_ascii.data);
}

/*
#[test]
fn simple_test() {
    let msh = "\
$MeshFormat
4.1 0 8
$EndMeshFormat
Hello
$EndMeshFormat
";
    mshio::parses(msh.as_bytes());

    assert!(true);
}
*/
