//#[test]
fn does_nothing() {
    assert!(true);
}

#[test]
fn bin_file() {
    let circle_2d_bin = include_bytes!("circle_2d_bin.msh");
    mshio::parse(circle_2d_bin);

    assert!(true);
}

//#[test]
fn simple_test() {
    let msh = "\
$MeshFormat
4.1 0 8
$EndMeshFormat
Hello
$EndMeshFormat
";
    //mshio::parse(&msh);

    assert!(true);
}

//#[test]
fn load_file() {
    let circle_2d = include_str!("circle_2d.msh");
    //mshio::parse(circle_2d);

    assert!(true);
}
