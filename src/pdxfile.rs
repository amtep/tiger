//! Helper functions for loading pdx script files in various character encodings.
//!
//! The main entry point is [`PdxFile`].

use crate::block::Block;
use crate::files::FileEntry;
use crate::parse::pdxfile::parse_pdx_file;
#[cfg(feature = "ck3")]
use crate::parse::pdxfile::{parse_reader_export, PdxfileMemory};
use crate::parse::ParserMemory;
#[cfg(feature = "hoi4")]
use crate::report::err;
use crate::report::{warn, ErrorKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdxEncoding {
    Utf8Bom,
    #[cfg(feature = "jomini")]
    Utf8OptionalBom,
    #[cfg(feature = "ck3")]
    Detect,
    #[cfg(feature = "hoi4")]
    Utf8NoBom,
}

pub struct PdxFile {}

impl PdxFile {
    /// Parse a UTF-8 file that should start with a BOM (Byte Order Marker).
    pub fn read(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.contents()?.has_bom() {
            let msg = "Expected UTF-8 BOM encoding";
            warn(ErrorKey::Encoding).msg(msg).abbreviated(entry).push();
        }
        parse_pdx_file(entry, parser)
    }

    /// Parse a UTF-8 file that may must start with a BOM (Byte Order Marker).
    #[cfg(feature = "hoi4")]
    pub fn read_no_bom(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if entry.contents()?.has_bom() {
            let msg = "Expected UTF-8 encoding without BOM";
            err(ErrorKey::Encoding).msg(msg).abbreviated(entry).push();
        }
        parse_pdx_file(entry, parser)
    }

    /// Parse a UTF-8 file that may optionally start with a BOM (Byte Order Marker).
    pub fn read_optional_bom(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        parse_pdx_file(entry, parser)
    }

    /// Parse a file that may be in UTF-8 with BOM encoding, or Windows-1252 encoding.
    #[cfg(feature = "ck3")]
    pub fn read_detect_encoding(entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        entry.contents_detect_encoding()?;
        parse_pdx_file(entry, parser)
    }

    pub fn read_encoded(
        entry: &FileEntry,
        encoding: PdxEncoding,
        parser: &ParserMemory,
    ) -> Option<Block> {
        match encoding {
            PdxEncoding::Utf8Bom => Self::read(entry, parser),
            #[cfg(feature = "jomini")]
            PdxEncoding::Utf8OptionalBom => Self::read_optional_bom(entry, parser),
            #[cfg(feature = "ck3")]
            PdxEncoding::Detect => Self::read_detect_encoding(entry, parser),
            #[cfg(feature = "hoi4")]
            PdxEncoding::Utf8NoBom => Self::read_no_bom(entry, parser),
        }
    }

    #[cfg(feature = "ck3")]
    pub fn reader_export(entry: &FileEntry, memory: &mut PdxfileMemory) {
        if let Some(contents) = entry.contents() {
            if !contents.has_bom() {
                let msg = "Expected UTF-8 BOM encoding";
                warn(ErrorKey::Encoding).msg(msg).abbreviated(entry).push();
            }
            parse_reader_export(entry, memory);
        }
    }
}
