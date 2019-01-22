use rust_s57::catalog::Catalog;
use std::fs::File;

type Result<T> = std::result::Result<T, failure::Error>;

fn print_error(err: &failure::Error) {
    for c in err.iter_chain() {
        println!("{}", c);
    }
}

#[test]
fn test_parse_catalog() {
    if let Err(e) = try_main() {
        println!("{}", e.backtrace());
        print_error(&e);
        assert!(false)
    }
}

fn try_main() -> Result<()> {
    let cf = File::open("tests/CATALOG.031").unwrap();
    let catalog = Catalog::new(cf)?;
    for record in catalog {
        println!("{:?}", record?);
    }
    Ok(())
}
