use std::io::Read;
use std::str::{from_utf8, Utf8Error};

const RECORD_SEPARATOR: &str = "\u{1E}";
const UNIT_SEPARATOR: &str = "\u{1F}";

#[derive(Debug, PartialEq)]
struct Leader {
    rl: u32,        // Record Lenght
    il: char,       // Interchange Level
    li: char,       // Leader Identifier
    cei: char,      // In Line Code Extension Indicator
    vn: char,       // Verison number
    ai: char,       // Application Indicator
    fcl: [char; 2], // Field Control Length
    ba: u32,        // Base Address Of Field Area
    csi: [char; 3], // Extended Character Set Indicator
    // Values of Entry Map
    flf: usize, // Size Of Field Length Field
    fpf: usize, // Size Of Field Position Field
    rsv: char,  // Reserved
    ftf: usize, // Size Of Field Tag Field
}

#[derive(Debug, PartialEq)]
struct DirectoryEntry {
    id: String,  // The Id of the field
    length: u32, // The length of the field in bytes
    offset: u32, // The offset in bytes form the start of the record
}

#[derive(Debug, PartialEq)]
enum DataStructureCode {
    SDI, // Single Data Item
    LS,  // Linear Structure
    MDS, // Multi-Dimensional structure
}
impl DataStructureCode {
    fn new(value: u8) -> Result<DataStructureCode> {
        match value {
            0 => Ok(DataStructureCode::SDI),
            1 => Ok(DataStructureCode::LS),
            2 => Ok(DataStructureCode::MDS),
            _ => Err(E::BadDataStructureCode()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum DataTypeCode {
    CS,  // Character String
    IP,  // Implicit Point
    EP,  // Explicit Point (Real)
    BF,  // Binary Form
    MDT, // Mixed Data Types
}
impl DataTypeCode {
    fn new(value: u8) -> Result<DataTypeCode> {
        match value {
            0 => Ok(DataTypeCode::CS),
            1 => Ok(DataTypeCode::IP),
            2 => Ok(DataTypeCode::EP),
            5 => Ok(DataTypeCode::BF),
            6 => Ok(DataTypeCode::MDT),
            _ => Err(E::BadDataTypeCode()),
        }
    }
}

// Truncated Escape Sequence
#[derive(Debug, PartialEq)]
enum TruncEscSeq {
    LE0, //Lexical Level 0
    LE1, //Lexical Level 1
    LE2, //Lexical Level 2
}
impl TruncEscSeq {
    fn new(value: String) -> Result<TruncEscSeq> {
        match value.as_ref() {
            "   " => Ok(TruncEscSeq::LE0),
            "-A " => Ok(TruncEscSeq::LE1),
            "%/@" => Ok(TruncEscSeq::LE2),
            _ => Err(E::BadTruncEscSeq()),
        }
    }
}

#[derive(Debug, PartialEq)]
struct FileControlField {
    dsc: DataStructureCode, // Data structure code
    dtc: DataTypeCode,
}

pub type Result<T> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum E {
    UtfError(std::str::Utf8Error),
    ParseError(std::string::ParseError),
    ParseIntError(std::num::ParseIntError),
    BadDirectoryData(),
    BadDataStructureCode(),
    BadDataTypeCode(),
    BadTruncEscSeq(),
}

impl From<std::str::Utf8Error> for E {
    fn from(e: Utf8Error) -> E {
        E::UtfError(e)
    }
}
impl From<std::string::ParseError> for E {
    fn from(e: std::string::ParseError) -> E {
        E::ParseError(e)
    }
}
impl From<std::num::ParseIntError> for E {
    fn from(e: std::num::ParseIntError) -> E {
        E::ParseIntError(e)
    }
}

fn parse_leader(byte: &[u8]) -> Result<Leader> {
    let rl = std::str::from_utf8(&byte[..5])?.parse()?;
    let il = byte[5] as char;
    let li = byte[6] as char;
    let cei = byte[7] as char;
    let vn = byte[8] as char;
    let ai = byte[9] as char;
    let fcl = [byte[10] as char, byte[11] as char];
    let ba = std::str::from_utf8(&byte[12..17])?.parse()?;
    let csi = [byte[17] as char, byte[18] as char, byte[19] as char];
    let flf = std::str::from_utf8(&byte[20..21])?.parse()?;
    let fpf = std::str::from_utf8(&byte[21..22])?.parse()?;
    let rsv = byte[22] as char;
    let ftf = from_utf8(&byte[23..24])?.parse()?;
    Ok(Leader {
        rl,
        il,
        li,
        cei,
        vn,
        ai,
        fcl,
        ba,
        csi,
        flf,
        fpf,
        rsv,
        ftf,
    })
}

// TODO: Change this function to use exact_chunk when it is stable
fn parse_directory(byte: &[u8], leader: Leader) -> Result<Vec<DirectoryEntry>> {
    let chunksize = (leader.ftf + leader.flf + leader.fpf) as usize;
    let dir_iter = byte.chunks(chunksize);
    let mut directories: Vec<DirectoryEntry> = Vec::new();
    for d in dir_iter {
        if d.len() != chunksize {
            return Err(E::BadDirectoryData());
        }
        let cont: String = from_utf8(&d[..])?.parse()?;
        println!("{}", cont);
        let id = from_utf8(&d[..leader.ftf])?.parse()?;
        let length = from_utf8(&d[leader.ftf..leader.ftf + leader.flf])?.parse()?;
        let offset = from_utf8(&d[leader.ftf + leader.flf..])?.parse()?;

        directories.push(DirectoryEntry { id, length, offset });
    }

    Ok(directories)
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_test_leader() -> Leader {
        Leader {
            rl: 241,
            il: '3',
            li: 'L',
            cei: 'E',
            vn: '1',
            ai: ' ',
            fcl: ['0', '9'],
            ba: 58,
            csi: [' ', '!', ' '],
            flf: 3,
            fpf: 4,
            rsv: '0',
            ftf: 4,
        }
    }

    fn get_test_direcory() -> Vec<DirectoryEntry> {
        vec![
            DirectoryEntry {
                id: "0000".to_string(),
                length: 19,
                offset: 0,
            },
            DirectoryEntry {
                id: "0001".to_string(),
                length: 44,
                offset: 19,
            },
            DirectoryEntry {
                id: "CATD".to_string(),
                length: 120,
                offset: 63,
            },
        ]
    }

    #[test]
    fn test_parse_leader() {
        let leader = "002413LE1 0900058 ! 3404".as_bytes();
        let expected = get_test_leader();
        let actual = parse_leader(leader).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_directory() {
        let leader = get_test_leader();
        let directory = "0000019000000010440019CATD1200063".as_bytes();
        let expected = get_test_direcory();
        let actual = parse_directory(directory, leader).unwrap();
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_parse_file_control_field() {
        let file_control_field = format!(
            "{}{}{}{}",
            "0000;&   ", UNIT_SEPARATOR, "0001CATD", RECORD_SEPARATOR
        );
        assert_eq!()
    }
}
