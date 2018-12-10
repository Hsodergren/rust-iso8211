use crate::catalog::E;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::prelude::*;
use std::io::Cursor;

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

    pub(crate) fn parse<R: BufRead>(&self, mut rdr: R) -> Result<Data, E> {
        let (d, t) = match &self {
            ParseData::Fixed(t, size) => {
                let mut data = vec![0; *size];
                rdr.read_exact(&mut data)?;
                (data, t)
            }
            ParseData::Variable(t) => {
                let mut data = Vec::new();
                rdr.read_until(0x1f, &mut data)?;
                (Vec::from(&data[..data.len() - 1]), t)
            }
        };
        let d = std::str::from_utf8(&d)?;
        match t {
            ParseType::String => Ok(Data::String(d.parse()?)),
            ParseType::Integer => Ok(Data::Integer(d.parse()?)),
            ParseType::Float => Ok(Data::Float(d.parse()?)),
        }
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

    #[test]
    fn read_data() {
        assert_eq!(
            ParseData::Fixed(ParseType::Integer, 5)
                .parse(Cursor::new("00001".as_bytes()))
                .unwrap(),
            Data::Integer(1)
        );
        assert_eq!(
            ParseData::Fixed(ParseType::String, 5)
                .parse(Cursor::new("Hejsa".as_bytes()))
                .unwrap(),
            Data::String(String::from("Hejsa"))
        );
        assert_eq!(
            ParseData::Fixed(ParseType::Float, 5)
                .parse(Cursor::new("0.005".as_bytes()))
                .unwrap(),
            Data::Float(0.005)
        );
        assert_eq!(
            ParseData::Variable(ParseType::Integer)
                .parse(Cursor::new(&[
                    '0' as u8, '0' as u8, '0' as u8, '0' as u8, '1' as u8, 0x1f
                ]))
                .unwrap(),
            Data::Integer(1)
        );
        assert_eq!(
            ParseData::Variable(ParseType::String)
                .parse(Cursor::new(&[
                    'H' as u8, 'e' as u8, 'j' as u8, 's' as u8, 'a' as u8, 0x1f
                ]))
                .unwrap(),
            Data::String(String::from("Hejsa"))
        );
        assert_eq!(
            ParseData::Variable(ParseType::Float)
                .parse(Cursor::new(&[
                    '0' as u8, '.' as u8, '0' as u8, '0' as u8, '5' as u8, 0x1f
                ]))
                .unwrap(),
            Data::Float(0.005)
        );
    }
}
