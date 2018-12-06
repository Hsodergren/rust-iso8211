use catalog::E;
use regex::Regex;
use std::io::Read;

lazy_static! {
    // Maybe a number folowed by A,I,R followed by maybe a parenthesied number
    // See tests
    static ref FIELD_REGEX: Regex = Regex::new(r"^(\d+)?([AIR])(\(\d*\))?").unwrap();
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ParseData {
    Fixed(ParseType, usize),
    Variable(ParseType),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ParseType {
    Integer,
    String,
    Float,
}

#[derive(Debug, PartialEq)]
pub enum Data {
    Integer(i64),
    String(String),
    Float(f64),
}

impl ParseData {
    pub(crate) fn new(s: &str) -> Result<(usize, ParseData), E> {
        match FIELD_REGEX.captures(s) {
            Some(cap) => {
                let num = cap.get(1).map_or(1, |c| c.as_str().parse().unwrap());
                let typ = match cap.get(2).unwrap().as_str() {
                    "A" => ParseType::String,
                    "I" => ParseType::Integer,
                    "R" => ParseType::Float,
                    _ => unreachable!(),
                };
                let pd = cap.get(3).map_or(ParseData::Variable(typ.clone()), |c| {
                    let c = &c.as_str();
                    ParseData::Fixed(typ, c[1..c.len() - 1].parse().unwrap())
                });
                Ok((num, pd))
            }
            None => Err(E::UnParsable(String::from(s))),
        }
    }

    pub(crate) fn parse<R: Read>(rdr: R) -> Result<Data, E> {
        Ok(Data::Integer(5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsedata() {
        assert_eq!(
            ParseData::new("A(3)").unwrap(),
            (1, ParseData::Fixed(ParseType::String, 3))
        );
        assert_eq!(
            ParseData::new("I(10)").unwrap(),
            (1, ParseData::Fixed(ParseType::Integer, 10))
        );
        assert_eq!(
            ParseData::new("R(5)").unwrap(),
            (1, ParseData::Fixed(ParseType::Float, 5))
        );
        assert_eq!(
            ParseData::new("5R").unwrap(),
            (5, ParseData::Variable(ParseType::Float))
        );
        assert_eq!(
            ParseData::new("10I").unwrap(),
            (10, ParseData::Variable(ParseType::Integer))
        );
        assert_eq!(
            ParseData::new("1A").unwrap(),
            (1, ParseData::Variable(ParseType::String))
        );
        assert_eq!(
            ParseData::new("2A(3)").unwrap(),
            (2, ParseData::Fixed(ParseType::String, 3))
        );
        assert_eq!(
            ParseData::new("10I(10)").unwrap(),
            (10, ParseData::Fixed(ParseType::Integer, 10))
        );
        assert_eq!(
            ParseData::new("1R(5)").unwrap(),
            (1, ParseData::Fixed(ParseType::Float, 5))
        );
    }
}
