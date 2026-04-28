//! Error types for parsing and writing EDN.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("EDN parse error at byte {pos}: {kind}")]
    Parse { pos: usize, kind: ErrorKind },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedEof,
    UnexpectedByte(u8),
    InvalidEscape(u8),
    InvalidUnicode(String),
    InvalidNumber(String),
    InvalidKeyword(String),
    InvalidSymbol(String),
    InvalidTag(String),
    InvalidChar(String),
    InvalidInst(String),
    InvalidUuid(String),
    UnclosedString,
    UnclosedList,
    UnclosedVector,
    UnclosedMap,
    UnclosedSet,
    OddMapElements,
    Utf8(String),
    TagWithoutElement(String),
    UserTagMissingNamespace(String),
    Other(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorKind::*;
        match self {
            UnexpectedEof => f.write_str("unexpected end of input"),
            UnexpectedByte(b) => write!(f, "unexpected byte 0x{:02x}", b),
            InvalidEscape(b) => write!(f, "invalid string escape \\{}", *b as char),
            InvalidUnicode(s) => write!(f, "invalid unicode escape: {}", s),
            InvalidNumber(s) => write!(f, "invalid number: {}", s),
            InvalidKeyword(s) => write!(f, "invalid keyword: {}", s),
            InvalidSymbol(s) => write!(f, "invalid symbol: {}", s),
            InvalidTag(s) => write!(f, "invalid tag: {}", s),
            InvalidChar(s) => write!(f, "invalid character literal: {}", s),
            InvalidInst(s) => write!(f, "invalid #inst: {}", s),
            InvalidUuid(s) => write!(f, "invalid #uuid: {}", s),
            UnclosedString => f.write_str("unclosed string"),
            UnclosedList => f.write_str("unclosed list"),
            UnclosedVector => f.write_str("unclosed vector"),
            UnclosedMap => f.write_str("unclosed map"),
            UnclosedSet => f.write_str("unclosed set"),
            OddMapElements => f.write_str("map literal must have an even number of forms"),
            Utf8(s) => write!(f, "invalid UTF-8: {}", s),
            TagWithoutElement(s) => write!(f, "tag {} has no following element", s),
            UserTagMissingNamespace(s) => {
                write!(f, "user tag #{} must have a namespace prefix", s)
            }
            Other(s) => f.write_str(s),
        }
    }
}

impl Error {
    pub(crate) fn at(pos: usize, kind: ErrorKind) -> Self {
        Error::Parse { pos, kind }
    }
}
