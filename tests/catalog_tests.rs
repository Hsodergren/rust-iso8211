use failure::Error;
use rust_s57::catalog::Catalog;

#[test]
fn test_parse_catalog() -> Result<(), Error> {
    let cf = std::fs::File::open("tests/CATALOG.031").unwrap();
    if let Err(e) = Catalog::new(cf) {
        let mut pretty = e.to_string();
        let mut prev = e.as_fail();
        while let Some(next) = prev.cause() {
            pretty.push_str(": ");
            pretty.push_str(&next.to_string());
            prev = next;
        }
        println!("{:?}", e.backtrace());
        assert!(false)
    }
    Ok(())
}
