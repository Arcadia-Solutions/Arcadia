use bip_bencode::BRefAccess;
use bip_bencode::{BDictAccess, BConvert, BencodeConvertError, BListAccess};

use error::{ParseError, ParseErrorKind, ParseResult};

/// Struct implemented the BencodeConvert trait for decoding the metainfo file.
struct MetainfoConverter;

impl BConvert for MetainfoConverter {
    type Error = ParseError;

    fn handle_error(&self, error: BencodeConvertError) -> ParseError {
        error.into()
    }
}

/// Global instance for our conversion struct.
const CONVERT: MetainfoConverter = MetainfoConverter;

/// Used as an error key to refer to the root bencode object.
pub const ROOT_ERROR_KEY: &'static [u8] = b"root";

/// Keys found within the root dictionary of a metainfo file.
pub const ANNOUNCE_LIST_KEY: &'static [u8] = b"announce-list";
pub const ANNOUNCE_URL_KEY:  &'static [u8] = b"announce";
pub const CREATION_DATE_KEY: &'static [u8] = b"creation date";
pub const COMMENT_KEY:       &'static [u8] = b"comment";
pub const CREATED_BY_KEY:    &'static [u8] = b"created by";
pub const ENCODING_KEY:      &'static [u8] = b"encoding";
pub const INFO_KEY:          &'static [u8] = b"info";

/// Keys found within the info dictionary of a metainfo file.
pub const PIECE_LENGTH_KEY: &'static [u8] = b"piece length";
pub const PIECES_KEY:       &'static [u8] = b"pieces";
pub const PRIVATE_KEY:      &'static [u8] = b"private";
pub const NAME_KEY:         &'static [u8] = b"name";
pub const FILES_KEY:        &'static [u8] = b"files";
pub const SOURCE_KEY:       &'static [u8] = b"source";

/// Keys found within the files dictionary of a metainfo file.
pub const LENGTH_KEY: &'static [u8] = b"length";
pub const MD5SUM_KEY: &'static [u8] = b"md5sum";
pub const PATH_KEY:   &'static [u8] = b"path";

/// Parses the root bencode as a dictionary.
pub fn parse_root_dict<B>(root_bencode: &B) -> ParseResult<&BDictAccess<B::BKey, B::BType>>
    where B: BRefAccess {
    CONVERT.convert_dict(root_bencode, ROOT_ERROR_KEY)
}

/// Parses the announce list from the root dictionary.
pub fn parse_announce_list<B>(root_dict: &BDictAccess<B::BKey, B>) -> Option<&BListAccess<B>>
    where B: BRefAccess<BType=B> {
    CONVERT.lookup_and_convert_list(root_dict, ANNOUNCE_LIST_KEY).ok()
}

/// Converts list of lists to vec of vecs
pub fn convert_announce_list<B>(list: &BListAccess<B>) -> Vec<Vec<String>>
    where B: BRefAccess<BType=B> {
    list.into_iter()
        .filter_map(|entry| entry.list())
        .map(|entry| {
            entry.into_iter()
                .filter_map(|bencode_str| bencode_str.str())
                .map(String::from)
                .collect()
        })
        .collect()
}

/// Parses the announce url from the root dictionary.
pub fn parse_announce_url<'a, B>(root_dict: &'a BDictAccess<B::BKey, B>) -> Option<&'a str>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_str(root_dict, ANNOUNCE_URL_KEY).ok()
}

/// Parses the creation date from the root dictionary.
pub fn parse_creation_date<B>(root_dict: &BDictAccess<B::BKey, B>) -> Option<i64>
    where B: BRefAccess {
    CONVERT.lookup_and_convert_int(root_dict, CREATION_DATE_KEY).ok()
}

/// Parses the comment from the root dictionary.
pub fn parse_comment<'a, B>(root_dict: &'a BDictAccess<B::BKey, B>) -> Option<&'a str>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_str(root_dict, COMMENT_KEY).ok()
}

/// Parses the created by from the root dictionary.
pub fn parse_created_by<'a, B>(root_dict: &'a BDictAccess<B::BKey, B>) -> Option<&'a str>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_str(root_dict, CREATED_BY_KEY).ok()
}

/// Parses the encoding from the root dictionary.
pub fn parse_encoding<'a, B>(root_dict: &'a BDictAccess<B::BKey, B>) -> Option<&'a str>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_str(root_dict, ENCODING_KEY).ok()
}

/// Parses the info dictionary from the root dictionary.
pub fn parse_info_bencode<'a, B>(root_dict: &'a BDictAccess<B::BKey, B>) -> ParseResult<&B>
    where B: BRefAccess {
    CONVERT.lookup(root_dict, INFO_KEY)
}

// ----------------------------------------------------------------------------//

/// Largest piece length accepted, torrent clients do not create bigger pieces.
const MAXIMUM_PIECE_LENGTH: i64 = 256 * 1024 * 1024;

/// Parses the piece length from the info dictionary.
///
/// The piece length sizes the buffers allocated when the torrent is rebuilt, so
/// an unbounded value would abort the process on an impossible allocation.
pub fn parse_piece_length<B>(info_dict: &BDictAccess<B::BKey, B>) -> ParseResult<u64>
    where B: BRefAccess {
    let piece_length = try!(CONVERT.lookup_and_convert_int(info_dict, PIECE_LENGTH_KEY));

    if piece_length <= 0 || piece_length > MAXIMUM_PIECE_LENGTH {
        return Err(ParseError::from_kind(ParseErrorKind::InvalidData {
            details: format!("piece length {} is out of range", piece_length),
        }));
    }

    Ok(piece_length as u64)
}

/// Parses the pieces from the info dictionary.
pub fn parse_pieces<'a, B>(info_dict: &'a BDictAccess<B::BKey, B>) -> ParseResult<&'a [u8]>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_bytes(info_dict, PIECES_KEY)
}

/// Parses the private flag from the info dictionary.
pub fn parse_private<B>(info_dict: &BDictAccess<B::BKey, B>) -> Option<bool>
    where B: BRefAccess {
    CONVERT.lookup_and_convert_int(info_dict, PRIVATE_KEY).ok().map(|p| p == 1)
}

/// Parses the name from the info dictionary.
pub fn parse_name<'a, B>(info_dict: &'a BDictAccess<B::BKey, B>) -> ParseResult<&'a str>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_str(info_dict, NAME_KEY)
}

/// Parses the files list from the info dictionary.
pub fn parse_files_list<B>(info_dict: &BDictAccess<B::BKey, B>) -> ParseResult<&BListAccess<B>>
    where B: BRefAccess<BType=B> {
    CONVERT.lookup_and_convert_list(info_dict, FILES_KEY)
}

// ----------------------------------------------------------------------------//

/// Parses the file dictionary from the file bencode.
pub fn parse_file_dict<B>(file_bencode: &B) -> ParseResult<&BDictAccess<B::BKey, B::BType>>
    where B: BRefAccess {
    CONVERT.convert_dict(file_bencode, FILES_KEY)
}

/// Parses the length from the info or file dictionary.
pub fn parse_length<B>(info_or_file_dict: &BDictAccess<B::BKey, B>) -> ParseResult<u64>
    where B: BRefAccess {
    CONVERT.lookup_and_convert_int(info_or_file_dict, LENGTH_KEY).map(|len| len as u64)
}

/// Parses the md5sum from the info or file dictionary.
pub fn parse_md5sum<'a, B>(info_or_file_dict: &'a BDictAccess<B::BKey, B>) -> Option<&'a [u8]>
    where B: BRefAccess + 'a {
    CONVERT.lookup_and_convert_bytes(info_or_file_dict, MD5SUM_KEY).ok()
}

/// Parses the path list from the file dictionary.
pub fn parse_path_list<B>(file_dict: &BDictAccess<B::BKey, B>) -> ParseResult<&BListAccess<B>>
    where B: BRefAccess<BType=B> {
    CONVERT.lookup_and_convert_list(file_dict, PATH_KEY)
}

/// Parses the path string from the path bencode.
pub fn parse_path_str<B>(path_bencode: &B) -> ParseResult<&str>
    where B: BRefAccess {
    CONVERT.convert_str(path_bencode, PATH_KEY)
}
