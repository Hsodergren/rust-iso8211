use data_parser::{Data, ParseData, ParseType};
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
    id: String,    // The Id of the field
    length: usize, // The length of the field in bytes
    offset: usize, // The offset in bytes form the start of the record
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
            "%/A" => Ok(TruncEscSeq::LE2),
            _ => Err(E::BadTruncEscSeq()),
        }
    }
}

#[derive(Debug, PartialEq)]
struct FileControlField {
    dsc: DataStructureCode, // Data structure code
    dtc: DataTypeCode,
}

#[derive(Debug, PartialEq)]
struct FieldControls {
    dsc: DataStructureCode,
    dtc: DataTypeCode,
    aux: String, // Auxilliary controls
    prt: String, // Printable graphics
    tes: TruncEscSeq,
}

// Data Descriptive Field Entry
#[derive(Debug, PartialEq)]
struct DDFEntry {
    fc: FieldControls,
}

pub type Result<T> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum E {
    BadDataStructureCode(),
    BadDataTypeCode(),
    BadDirectoryData(),
    BadTruncEscSeq(),
    IOError(std::io::Error),
    ParseError(std::string::ParseError),
    ParseIntError(std::num::ParseIntError),
    UtfError(std::str::Utf8Error),
}

impl From<std::io::Error> for E {
    fn from(e: std::io::Error) -> E {
        E::IOError(e)
    }
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
fn parse_directory(byte: &[u8], leader: &Leader) -> Result<Vec<DirectoryEntry>> {
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

fn parse_field_controls(byte: &[u8]) -> Result<FieldControls> {
    let dsc = DataStructureCode::new(from_utf8(&byte[0..1])?.parse()?)?;
    let dtc = DataTypeCode::new(from_utf8(&byte[1..2])?.parse()?)?;
    let aux = from_utf8(&byte[2..4])?.to_string();
    let prt = from_utf8(&byte[4..6])?.to_string();
    let tes = TruncEscSeq::new(from_utf8(&byte[6..])?.to_string())?;

    Ok(FieldControls {
        dsc,
        dtc,
        aux,
        prt,
        tes,
    })
}

fn parse_array_descriptors(byte: &[u8]) -> Result<Vec<String>> {
    Ok(from_utf8(&byte[..])?
        .split("!")
        .map(|s| String::from(s))
        .collect::<Vec<String>>())
}

fn parse_format_controls(byte: &[u8]) -> Result<Vec<ParseData>> {
    // Remove surrounding parenthesies and create ParseDatas
    Ok(from_utf8(&byte[1..byte.len() - 1])?
        .split(",")
        .flat_map(|fc| {
            let (n, d) = ParseData::new(fc).unwrap();
            std::iter::repeat(d).take(n)
        })
        .collect())
}

fn parse_ddf(byte: &[u8]) -> Result<DDFEntry> {
    let mut cursor = std::io::Cursor::new(byte);
    let mut fc_buffer = [0; 10];
    cursor.read_exact(&mut fc_buffer);
    let fc = parse_field_controls(&fc_buffer)?;
    Ok(DDFEntry { fc })
}

struct DDR {
    leader: Leader,
    directory: Vec<DirectoryEntry>,
    // file_control_field,
    data_descriptive_field: Vec<DDFEntry>,
}

pub struct Catalog<R: Read> {
    ddr: DDR, // Data Descriptive Record
    rdr: R,   // reader to ask for Data Records
}

impl<R: Read> Catalog<R> {
    pub fn new(mut cat_rdr: R) -> Result<Catalog<R>> {
        // Read the length of the DDR, stored in the first 5 bytes
        let mut ddr_bytes = [0; 5];
        cat_rdr.read(&mut ddr_bytes)?;

        // Read the rest of the DDR
        let ddr_length: usize = from_utf8(&ddr_bytes)?.parse()?;
        let mut ddr_data = vec![0; ddr_length - 5];
        cat_rdr.read_exact(&mut ddr_data)?;

        //Concatenate to make complete ddr data
        let mut ddr_bytes = ddr_bytes.to_vec();
        ddr_bytes.append(&mut ddr_data);
        let ddr = parse_ddr(&ddr_bytes)?;
        Ok(Catalog {
            ddr: ddr,
            rdr: cat_rdr,
        })
    }
}

fn parse_ddr(ddr_bytes: &Vec<u8>) -> Result<DDR> {
    let leader = parse_leader(&ddr_bytes[..24])?;
    let directory = parse_directory(&ddr_bytes[24..], &leader)?;
    let data_descriptive_field = Vec::new();

    Ok(DDR {
        leader,
        directory,
        data_descriptive_field,
    })
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

    fn get_test_directory() -> Vec<DirectoryEntry> {
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

    fn get_test_field_controls() -> FieldControls {
        FieldControls {
            dsc: DataStructureCode::LS,
            dtc: DataTypeCode::MDT,
            aux: "00".to_string(),
            prt: ";&".to_string(),
            tes: TruncEscSeq::LE1,
        }
    }

    fn get_test_format_controls() -> Vec<ParseData> {
        vec![
            ParseData::Fixed(ParseType::String, 2),
            ParseData::Fixed(ParseType::Integer, 10),
            ParseData::Fixed(ParseType::Integer, 10),
            ParseData::Variable(ParseType::Float),
            ParseData::Variable(ParseType::Float),
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
        let expected = get_test_directory();
        let actual = parse_directory(directory, &leader).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_field_controls() {
        let field_controls = "1600;&-A ".as_bytes();
        let expected = get_test_field_controls();
        let actual = parse_field_controls(field_controls).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_array_descriptor() {
        let array_descriptor =
            "RCNM!RCID!FILE!LFIL!VOLM!IMPL!SLAT!WLON!NLAT!ELON!CRCS!COMT".as_bytes();
        let expected = vec![
            "RCNM", "RCID", "FILE", "LFIL", "VOLM", "IMPL", "SLAT", "WLON", "NLAT", "ELON", "CRCS",
            "COMT",
        ];
        let actual = parse_array_descriptors(array_descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_format_controls() {
        let format_controls = "(A(2),2I(10),2R)".as_bytes();
        let expected = get_test_format_controls();
        let actual = parse_format_controls(format_controls).unwrap();
        assert_eq!(actual, expected);
    }
}
