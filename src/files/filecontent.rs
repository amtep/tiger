use std::{
    borrow::Cow,
    fs::{read, read_to_string},
    slice::from_raw_parts,
    sync::OnceLock,
};

use encoding_rs::WINDOWS_1252;

use crate::{
    files::FileEntry,
    report::{err, ErrorKey},
};

pub const BOM_UTF8_BYTES: [u8; 3] = *b"\xef\xbb\xbf";
pub const BOM_CHAR: char = '\u{feff}';

#[derive(Debug)]
pub struct FileContent {
    /// Full content of the file. It must never be moved or modified.
    // full: Cow<'static, str>,
    full: &'static str,
    /// The file content split into lines. Cached to avoid doing that work again.
    // Stored as raw pointers to make the self reference work
    lines: OnceLock<Box<[(*const u8, usize)]>>,
}

impl FileContent {
    pub fn new(mut s: String) -> Self {
        s.shrink_to_fit();
        //Self { full: Cow::from(s), lines: OnceLock::new() }
        Self { full: String::leak(s), lines: OnceLock::new() }
    }

    pub fn new_static(s: &'static str) -> Self {
        //Self { full: Cow::from(s), lines: OnceLock::new() }
        Self { full: s, lines: OnceLock::new() }
    }

    pub fn read(entry: &FileEntry) -> Option<Self> {
        let contents = match read_to_string(entry.fullpath()) {
            Ok(contents) => contents,
            Err(e) => {
                err(ErrorKey::ReadError)
                    .msg("could not read file")
                    .info(format!("{e:#}"))
                    .abbreviated(entry)
                    .push();
                return None;
            }
        };
        Some(FileContent::new(contents))
    }

    pub fn read_detect_encoding(entry: &FileEntry) -> Option<Self> {
        let bytes = match read(entry.fullpath()) {
            Ok(bytes) => bytes,
            Err(e) => {
                err(ErrorKey::ReadError)
                    .msg("could not read file")
                    .info(format!("{e:#}"))
                    .abbreviated(entry)
                    .push();
                return None;
            }
        };
        if bytes.starts_with(&BOM_UTF8_BYTES) {
            let contents = match String::from_utf8(bytes) {
                Ok(utf8) => utf8,
                Err(e) => {
                    err(ErrorKey::ReadError)
                        .msg("could not decode UTF-8 file")
                        .info(format!("{e:#}"))
                        .abbreviated(entry)
                        .push();
                    return None;
                }
            };
            Some(FileContent::new(contents))
        } else {
            let contents =
                match WINDOWS_1252.decode_without_bom_handling_and_without_replacement(&bytes) {
                    Some(Cow::Owned(utf8)) => utf8,
                    Some(Cow::Borrowed(_)) => unsafe {
                        // If the result is borrowed, we can transform the original byte vec
                        // and avoid the data copy we'd otherwise get doirg Cow::to_owned()
                        // Safety: the byte vec is already confirmed to be valid uft8.
                        String::from_utf8_unchecked(bytes)
                    },
                    None => {
                        err(ErrorKey::Encoding)
                            .msg("could not decode WINDOWS-1252 file")
                            .abbreviated(entry)
                            .push();
                        return None;
                    }
                };
            Some(FileContent::new(contents))
        }
    }

    pub fn full(&self) -> &'static str {
        self.full
    }

    pub fn has_bom(&self) -> bool {
        self.full.starts_with(BOM_CHAR)
    }

    pub fn nobom(&self) -> &'static str {
        self.full.strip_prefix(BOM_CHAR).unwrap_or(self.full)
    }

    fn lines_get_or_init(&self) -> &[(*const u8, usize)] {
        self.lines
            .get_or_init(|| self.nobom().lines().map(|r| (r.as_ptr(), r.len())).collect())
            .as_ref()
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines_get_or_init().iter().map(|(p, len)| unsafe {
            // Safety: p and len are constructed from the owned or static string in `full`.
            // It is never modified or moved, and guaranteed to be valid utf8.
            std::str::from_utf8_unchecked(from_raw_parts(*p, *len))
        })
    }

    pub fn line(&self, line_num: usize) -> Option<&str> {
        if line_num == 0 {
            None
        } else {
            self.lines_get_or_init().get(line_num - 1).map(|(p, len)| unsafe {
                // Safety: p and len are constructed from the owned or static string in `full`.
                // It is never modified or moved, and guaranteed to be valid utf8.
                std::str::from_utf8_unchecked(from_raw_parts(*p, *len))
            })
        }
    }
}

// Safety: The raw pointers are never exposed, so threads can't do anything funny with them
unsafe impl Send for FileContent {}
unsafe impl Sync for FileContent {}
