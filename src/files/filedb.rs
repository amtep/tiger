use std::{path::PathBuf, sync::Arc};

use crate::{files::FileEntry, FileKind, TigerHashMap};

#[derive(Debug, Default)]
pub struct FileDb {
    /// The base game and mod files in arbitrary order.
    pub(super) files: TigerHashMap<PathBuf, Arc<FileEntry>>,
}

impl FileDb {
    /// Gets a reference to a file entry.
    /// If the fileset has not been finalized, a new entry will be created and returned if needed.
    pub(crate) fn get_or_create_entry(
        &mut self,
        path: PathBuf,
        kind: FileKind,
        fullpath: PathBuf,
    ) -> &Arc<FileEntry> {
        self.files.entry(fullpath.clone()).or_insert_with(|| FileEntry::new(path, kind, fullpath))
    }
}
