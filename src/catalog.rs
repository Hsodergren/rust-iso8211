use crate::data_parser::{Data, ParseData, ParseType};
use std::io::Read;
use std::str::{from_utf8, Utf8Error};

const RECORD_SEPARATOR: u8 = 0x1e;
const UNIT_SEPARATOR: u8 = 0x1f;

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
    fn new(value: &str) -> Result<TruncEscSeq> {
        match value {
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
    fic: FieldControls,
    name: String,
    foc: Vec<(String, ParseData)>,
}

pub type Result<T> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum E {
    BadDataStructureCode(),
    BadDataTypeCode(),
    BadDirectoryData(),
    BadTruncEscSeq(),
    EmptyFormatControls,
    InvalidHeader,
    IOError(std::io::Error),
    ParseError(std::string::ParseError),
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    UnParsable(String),
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
impl From<std::num::ParseFloatError> for E {
    fn from(e: std::num::ParseFloatError) -> E {
        E::ParseFloatError(e)
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
    let tes = TruncEscSeq::new(from_utf8(&byte[6..])?)?;

    Ok(FieldControls {
        dsc,
        dtc,
        aux,
        prt,
        tes,
    })
}

fn parse_array_descriptors(byte: &[u8]) -> Result<Vec<String>> {
    if byte.is_empty() {
        Ok(vec![String::from("DRID")])
    } else {
        Ok(from_utf8(&byte[..])?
            .split('!')
            .map(String::from)
            .collect::<Vec<String>>())
    }
}

fn parse_format_controls(byte: &[u8]) -> Result<Vec<ParseData>> {
    if byte.len() < 2 {
        Err(E::EmptyFormatControls)
    } else {
        // Remove surrounding parenthesies and create ParseDatas
        Ok(from_utf8(&byte[1..byte.len() - 1])?
            .split(',')
            .map(|fc| ParseData::new(fc))
            .collect::<Result<Vec<(usize, ParseData)>>>()?
            .into_iter()
            .flat_map(|pd| std::iter::repeat(pd.1).take(pd.0))
            .collect())
    }
}

fn parse_ddfs(byte: &[u8], dirs: &[DirectoryEntry]) -> Result<Vec<DDFEntry>> {
    // We should absolutely handle the file control field... later... but for now we skip it.
    dirs.iter()
        .skip(1)
        .map(|dir| {
            let s = dir.offset;
            //  take -1 to remove the record separator from the slice
            let e = dir.offset + dir.length - 1;
            parse_ddf(&byte[s..e])
        })
        .collect()
}

fn parse_ddf(byte: &[u8]) -> Result<DDFEntry> {
    let parts = byte.split(|&b| b == UNIT_SEPARATOR).collect::<Vec<&[u8]>>();
    let (fic_bytes, name_bytes) = parts.get(0).ok_or(E::InvalidHeader)?.split_at(9);
    let fic = parse_field_controls(fic_bytes)?;
    let name: String = from_utf8(name_bytes)?.parse()?;
    let array_desc = parse_array_descriptors(parts.get(1).ok_or(E::InvalidHeader)?)?;
    let data_parser = parse_format_controls(parts.get(2).ok_or(E::InvalidHeader)?)?;
    if array_desc.len() == data_parser.len() {
        let foc = array_desc
            .into_iter()
            .zip(data_parser.into_iter())
            .collect();
        Ok(DDFEntry { fic, name, foc })
    } else {
        Err(E::InvalidHeader)
    }
}

#[derive(Debug)]
struct DDR {
    leader: Leader,
    dirs: Vec<DirectoryEntry>,
    // file_control_field,
    data_descriptive_fields: Vec<DDFEntry>,
}

#[derive(Debug)]
pub struct Catalog<R: Read> {
    ddr: DDR, // Data Descriptive Record
    rdr: R,   // reader to ask for Data Records
}

impl<R: Read> Catalog<R> {
    pub fn new(mut rdr: R) -> Result<Catalog<R>> {
        // Read the length of the DDR, stored in the first 5 bytes
        let mut ddr_bytes = [0; 5];
        rdr.read_exact(&mut ddr_bytes)?;

        // Read the rest of the DDR
        let ddr_length: usize = from_utf8(&ddr_bytes)?.parse()?;
        let mut ddr_data = vec![0; ddr_length - 5];
        rdr.read_exact(&mut ddr_data)?;

        //Concatenate to make complete ddr data
        let mut ddr_bytes = ddr_bytes.to_vec();
        ddr_bytes.append(&mut ddr_data);
        let ddr = parse_ddr(&ddr_bytes)?;
        Ok(Catalog { ddr, rdr })
    }
}

fn parse_ddr(ddr_bytes: &[u8]) -> Result<DDR> {
    let leader = parse_leader(&ddr_bytes[..24])?;
    let field_area_idx = match ddr_bytes.iter().position(|&b| b == RECORD_SEPARATOR) {
        Some(index) => index,
        None => return Err(E::BadDirectoryData()),
    };
    let dirs = parse_directory(&ddr_bytes[24..field_area_idx], &leader)?;
    let data_descriptive_fields = parse_ddfs(&ddr_bytes[field_area_idx + 1..], &dirs)?;

    Ok(DDR {
        leader,
        dirs,
        data_descriptive_fields,
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
    fn test_parse_format_controls_with_empty() {
        let array_descriptor = &[0u8; 0];
        assert!(parse_format_controls(array_descriptor).is_err())
    }

    #[test]
    fn test_parse_format_controls() {
        let format_controls = "(A(2),2I(10),2R)".as_bytes();
        let expected = get_test_format_controls();
        let actual = parse_format_controls(format_controls).unwrap();
        assert_eq!(actual, expected);
    }
}
