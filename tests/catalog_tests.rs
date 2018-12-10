extern crate rust_s57;

use rust_s57::catalog::Catalog;

#[test]
fn test_parse_catalog() {
    let cf = std::fs::File::open("tests/CATALOG.031").unwrap();
    assert!(Catalog::new(cf).is_ok());
}
