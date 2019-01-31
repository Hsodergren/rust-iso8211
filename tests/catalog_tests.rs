use rust_s57::catalog::Catalog;
use std::fs::File;

type Result<T> = std::result::Result<T, failure::Error>;

fn print_error(err: &failure::Error) {
    println!("");
    for c in err.iter_chain() {
        println!("{}", c);
    }
    println!("");
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
    let mut last_record = None;
    for record in catalog {
        match record {
            Ok(r) => last_record = Some(r),
            Err(err) => {
                println!("Last successful parse was:");
                println!("{:?}", last_record.unwrap());
                return Err(err.into());
            }
        }
    }
    Ok(())
}
