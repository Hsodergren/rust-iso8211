use crate::catalog::{Result, UNIT_SEPARATOR};
use crate::error::ErrorKind;
use failure::ResultExt;
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
    Integer(Option<i64>),
    String(String),
    Float(Option<f64>),
}

impl ParseData {
    pub(crate) fn from_str(s: &str) -> Result<(usize, ParseData)> {
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
            None => Err(ErrorKind::UnParsableFormatControl(String::from(s)).into()),
        }
    }

    pub(crate) fn parse<R: BufRead>(&self, mut rdr: R) -> Result<Data> {
        let (d, t) = match &self {
            ParseData::Fixed(t, size) => {
                let mut data = vec![0; *size];
                rdr.read_exact(&mut data)
                    .with_context(|err| ErrorKind::IOError(err.kind()))?;
                (data, t)
            }
            ParseData::Variable(t) => {
                let mut data = Vec::new();
                rdr.read_until(UNIT_SEPARATOR, &mut data)
                    .with_context(|err| ErrorKind::IOError(err.kind()))?;
                (Vec::from(&data[..data.len() - 1]), t)
            }
        };
        let d = std::str::from_utf8(&d).with_context(|&err| ErrorKind::UtfError(err))?;
        match t {
            ParseType::String => Ok(Data::String(d.to_string())),
            ParseType::Integer => {
                if d.is_empty() {
                    Ok(Data::Integer(None))
                } else {
                    Ok(Data::Integer(Some(d.parse().with_context(
                        |err: &std::num::ParseIntError| {
                            ErrorKind::ParseIntError(err.clone(), d.to_string())
                        },
                    )?)))
                }
            }
            ParseType::Float => {
                if d.is_empty() {
                    Ok(Data::Float(None))
                } else {
                    Ok(Data::Float(Some(d.parse().with_context(
                        |err: &std::num::ParseFloatError| {
                            ErrorKind::ParseFloatError(err.clone(), d.to_string())
                        },
                    )?)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsedata() {
        assert_eq!(
            ParseData::from_str("A(3)").unwrap(),
            (1, ParseData::Fixed(ParseType::String, 3))
        );
        assert_eq!(
            ParseData::from_str("I(10)").unwrap(),
            (1, ParseData::Fixed(ParseType::Integer, 10))
        );
        assert_eq!(
            ParseData::from_str("R(5)").unwrap(),
            (1, ParseData::Fixed(ParseType::Float, 5))
        );
        assert_eq!(
            ParseData::from_str("5R").unwrap(),
            (5, ParseData::Variable(ParseType::Float))
        );
        assert_eq!(
            ParseData::from_str("10I").unwrap(),
            (10, ParseData::Variable(ParseType::Integer))
        );
        assert_eq!(
            ParseData::from_str("1A").unwrap(),
            (1, ParseData::Variable(ParseType::String))
        );
        assert_eq!(
            ParseData::from_str("2A(3)").unwrap(),
            (2, ParseData::Fixed(ParseType::String, 3))
        );
        assert_eq!(
            ParseData::from_str("10I(10)").unwrap(),
            (10, ParseData::Fixed(ParseType::Integer, 10))
        );
        assert_eq!(
            ParseData::from_str("1R(5)").unwrap(),
            (1, ParseData::Fixed(ParseType::Float, 5))
        );
    }

    #[test]
    fn read_data() {
        assert_eq!(
            ParseData::Fixed(ParseType::Integer, 5)
                .parse(Cursor::new("00001".as_bytes()))
                .unwrap(),
            Data::Integer(Some(1))
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
            Data::Float(Some(0.005))
        );
        assert_eq!(
            ParseData::Variable(ParseType::Integer)
                .parse(Cursor::new(&[
                    '0' as u8,
                    '0' as u8,
                    '0' as u8,
                    '0' as u8,
                    '1' as u8,
                    UNIT_SEPARATOR,
                ]))
                .unwrap(),
            Data::Integer(Some(1))
        );
        assert_eq!(
            ParseData::Variable(ParseType::String)
                .parse(Cursor::new(&[
                    'H' as u8,
                    'e' as u8,
                    'j' as u8,
                    's' as u8,
                    'a' as u8,
                    UNIT_SEPARATOR,
                ]))
                .unwrap(),
            Data::String(String::from("Hejsa"))
        );
        assert_eq!(
            ParseData::Variable(ParseType::Float)
                .parse(Cursor::new(&[
                    '0' as u8,
                    '.' as u8,
                    '0' as u8,
                    '0' as u8,
                    '5' as u8,
                    UNIT_SEPARATOR,
                ]))
                .unwrap(),
            Data::Float(Some(0.005))
        );
    }
}
