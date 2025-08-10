//! Track all the files (vanilla and mods) that are relevant to the current validation.

use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::string::ToString;
use std::sync::{Arc, OnceLock, RwLock};

use anyhow::Result;
use rayon::prelude::*;

use super::filecontent::FileContent;
use crate::block::Block;
use crate::everything::Everything;
use crate::files::fileset_builder::FilesetBuilder;
use crate::files::FileDb;
use crate::game::Game;
use crate::helpers::TigerHashSet;
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pathtable::{PathTable, PathTableIndex};
use crate::report::{err, fatal, report, store_source_file, ErrorKey, Severity};
use crate::token::Token;
use crate::TigerHashMap;

/// Note that ordering of these enum values matters.
/// Files later in the order will override files of the same name before them,
/// and the warnings about duplicates take that into account.
// TODO: verify the relative order of `Clausewitz` and `Jomini`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileKind {
    /// `Internal` is for parsing tiger's own data. The user should not see warnings from this.
    Internal,
    /// `Clausewitz` and `Jomini` are directories bundled with the base game.
    Clausewitz,
    Jomini,
    /// The base game files.
    Vanilla,
    /// Downloadable content present on the user's system.
    Dlc(u8),
    /// Other mods loaded as directed by the config file. 0-based indexing.
    LoadedMod(u8),
    /// The mod under scrutiny. Usually, warnings are not emitted unless they touch `Mod` files.
    Mod,
}

impl FileKind {
    pub fn counts_as_vanilla(&self) -> bool {
        match self {
            FileKind::Clausewitz | FileKind::Jomini | FileKind::Vanilla | FileKind::Dlc(_) => true,
            FileKind::Internal | FileKind::LoadedMod(_) | FileKind::Mod => false,
        }
    }
}

#[derive(Debug)]
pub struct FileEntry {
    /// Pathname components below the mod directory or the vanilla game dir
    /// Must not be empty.
    path: PathBuf,
    /// Whether it's a vanilla or mod file
    kind: FileKind,
    /// Index into the `PathTable`. Used to initialize `Loc`, which doesn't carry a copy of the pathbuf.
    /// A `FileEntry` might not have this index, because `FileEntry` needs to be usable before the (ordered)
    /// path table is created.
    idx: OnceLock<PathTableIndex>,
    /// The full filesystem path of this entry. Not used for ordering or equality.
    fullpath: PathBuf,
    /// The files contents. Not used for ordering or equality.
    contents: OnceLock<Option<FileContent>>,
}

impl FileEntry {
    // We make sure every file entry is wrapped in an Arc so that everyone has access
    // to late initialized fields
    pub(super) fn new(path: PathBuf, kind: FileKind, fullpath: PathBuf) -> Arc<Self> {
        debug_assert!(path.file_name().is_some());
        let entry = Arc::new(Self {
            path,
            kind,
            idx: OnceLock::new(),
            fullpath,
            contents: OnceLock::new(),
        });
        store_source_file(Arc::clone(&entry));
        entry
    }

    pub fn new_internal(description: PathBuf, contents: &'static str) -> Self {
        Self {
            path: description.clone(),
            kind: FileKind::Internal,
            idx: OnceLock::from(PathTableIndex::UNTRACKED),
            fullpath: description,
            contents: OnceLock::from(Some(FileContent::new_static(contents))),
        }
    }

    /// Creates a new file entry which will not have errors logged.
    pub fn new_untracked(path: PathBuf, kind: FileKind, fullpath: PathBuf) -> Self {
        debug_assert!(path.file_name().is_some());
        Self {
            path,
            kind,
            idx: OnceLock::from(PathTableIndex::UNTRACKED),
            fullpath,
            contents: OnceLock::new(),
        }
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn fullpath(&self) -> &Path {
        &self.fullpath
    }

    /// Convenience function
    /// Won't panic because `FileEntry` with empty filename is not allowed.
    #[allow(clippy::missing_panics_doc)]
    pub fn filename(&self) -> &OsStr {
        self.path.file_name().unwrap()
    }

    /// Prefer `path_idx` when possible.
    /// Using this funtion before the file set is finalized will
    /// result in an unsorted path table
    pub(crate) fn get_or_init_path_idx(&self) -> PathTableIndex {
        *self.idx.get_or_init(|| PathTable::store(self.path.clone(), self.fullpath.clone()))
    }

    pub fn path_idx(&self) -> Option<PathTableIndex> {
        self.idx.get().copied()
    }

    /// Tests if this entry exists as a file on disk
    pub fn is_file(&self) -> bool {
        self.fullpath().is_file()
    }

    pub fn contents(&self) -> Option<&FileContent> {
        self.contents.get_or_init(|| FileContent::read(self)).as_ref()
    }

    pub fn contents_detect_encoding(&self) -> Option<&FileContent> {
        self.contents.get_or_init(|| FileContent::read_detect_encoding(self)).as_ref()
    }
}

impl Display for FileEntry {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.path.display())
    }
}

impl PartialEq for FileEntry {
    fn eq(&self, other: &Self) -> bool {
        (self.idx.get().is_some() == other.idx.get().is_some() && self.idx == other.idx)
            || (self.path == other.path && self.kind == other.kind)
    }
}

impl Eq for FileEntry {}

impl Hash for FileEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.kind.hash(state);
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare idx if available (for speed), otherwise compare the paths.
        let path_ord = if let (Some(self_idx), Some(other_idx)) = (self.idx.get(), other.idx.get())
        {
            self_idx.cmp(other_idx)
        } else {
            self.path.cmp(&other.path)
        };

        // For same paths, the later [`FileKind`] wins.
        if path_ord == Ordering::Equal {
            self.kind.cmp(&other.kind)
        } else {
            path_ord
        }
    }
}

/// A trait for a submodule that can process files.
pub trait FileHandler<T: Send>: Sync + Send {
    /// The `FileHandler` can read settings it needs from the ck3-tiger config.
    fn config(&mut self, _config: &Block) {}

    /// Which files this handler is interested in.
    /// This is a directory prefix of files it wants to handle,
    /// relative to the mod or vanilla root.
    fn subpath(&self) -> PathBuf;

    /// This is called for each matching file, in arbitrary order.
    /// If a `T` is returned, it will be passed to `handle_file` later.
    /// Since `load_file` is executed multi-threaded while `handle_file`
    /// is single-threaded, try to do the heavy work in this function.
    fn load_file(&self, entry: &Arc<FileEntry>, parser: &ParserMemory) -> Option<T>;

    /// This is called for each matching file in turn, in lexical order.
    /// That's the order in which the CK3 game engine loads them too.
    fn handle_file(&mut self, entry: &Arc<FileEntry>, loaded: T);

    /// This is called after all files have been handled.
    /// The `FileHandler` can generate indexes, perform full-data checks, etc.
    fn finalize(&mut self) {}
}

#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub(super) struct FilePaths {
    /// The CK3 game directory.
    pub(super) vanilla_root: Option<PathBuf>,

    /// Extra CK3 directory loaded before vanilla.
    #[cfg(feature = "jomini")]
    pub(super) clausewitz_root: Option<PathBuf>,

    /// Extra CK3 directory loaded before vanilla.
    #[cfg(feature = "jomini")]
    pub(super) jomini_root: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct LoadedMod {
    /// The `FileKind` to use for file entries from this mod.
    kind: FileKind,

    /// The tag used for this mod in error messages.
    #[allow(dead_code)]
    label: String,

    /// The location of this mod in the filesystem.
    root: PathBuf,

    /// A list of directories that should not be read from vanilla or previous mods.
    replace_paths: Vec<PathBuf>,
}

impl LoadedMod {
    pub(super) fn new_main_mod(root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Self { kind: FileKind::Mod, label: "MOD".to_string(), root, replace_paths }
    }

    pub(super) fn new(
        kind: FileKind,
        label: String,
        root: PathBuf,
        replace_paths: Vec<PathBuf>,
    ) -> Self {
        Self { kind, label, root, replace_paths }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn should_replace(&self, path: &Path) -> bool {
        self.replace_paths.iter().any(|p| p == path)
    }
}

#[derive(Debug)]
pub struct Fileset {
    #[allow(dead_code)]
    pub(super) paths: FilePaths,

    /// The mod being analyzed.
    #[allow(dead_code)]
    pub(super) the_mod: LoadedMod,

    /// Other mods to be loaded before `mod`, in order.
    pub loaded_mods: Vec<LoadedMod>,

    /// DLC directories to be loaded after vanilla, in order.
    #[allow(dead_code)]
    pub(super) loaded_dlcs: Vec<LoadedMod>,

    /// The ck3-tiger config.
    pub(crate) config: Block,

    /// Collection of all known files.
    #[allow(dead_code)]
    pub(super) db: FileDb,

    /// The bose game and mod files in the order the game would load them.
    pub(super) ordered_files: Vec<Arc<FileEntry>>,

    /// Filename Tokens for the files in `ordered_files`.
    /// Used for [`Fileset::iter_keys()`].
    pub(super) filename_tokens: Vec<Token>,

    /// All filenames from `ordered_files`, for quick lookup.
    pub(super) filenames: TigerHashMap<PathBuf, Arc<FileEntry>>,

    /// All directories that have been looked up, for quick lookup.
    pub(super) directories: RwLock<TigerHashSet<PathBuf>>,

    /// Filenames that have been looked up during validation. Used to filter the --unused output.
    pub(super) used: RwLock<TigerHashSet<String>>,
}

impl Fileset {
    pub fn builder(vanilla_dir: Option<&Path>) -> FilesetBuilder {
        FilesetBuilder::new(vanilla_dir)
    }

    pub fn get_files_under<'a>(&'a self, subpath: &'a Path) -> &'a [Arc<FileEntry>] {
        let start = self.ordered_files.partition_point(|entry| entry.path < subpath);
        let end = start
            + self.ordered_files[start..].partition_point(|entry| entry.path.starts_with(subpath));
        &self.ordered_files[start..end]
    }

    pub fn filter_map_under<'a, F, T>(&'a self, subpath: &'a Path, f: F) -> Vec<T>
    where
        F: Fn(&'a Arc<FileEntry>) -> Option<T> + Sync + Send,
        T: Send,
    {
        self.get_files_under(subpath).par_iter().filter_map(f).collect()
    }

    pub fn handle<T: Send, H: FileHandler<T>>(&self, handler: &mut H, parser: &ParserMemory) {
        handler.config(&self.config);
        let subpath = handler.subpath();
        let entries = self.filter_map_under(&subpath, |entry| {
            handler.load_file(entry, parser).map(|loaded| (entry, loaded))
        });
        for (entry, loaded) in entries {
            handler.handle_file(entry, loaded);
        }
        handler.finalize();
    }

    /// # Panics
    pub fn mark_used(&self, file: &str) {
        let file = file.strip_prefix('/').unwrap_or(file);
        self.used.write().unwrap().insert(file.to_string());
    }

    pub fn entry(&self, key: &str) -> Option<&Arc<FileEntry>> {
        let key = key.strip_prefix('/').unwrap_or(key);
        let filepath = if Game::is_hoi4() && key.contains('\\') {
            PathBuf::from(key.replace('\\', "/"))
        } else {
            PathBuf::from(key)
        };
        self.filenames.get(&filepath)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.entry(key).is_some()
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.filename_tokens.iter()
    }

    /// # Panics
    pub fn entry_exists(&self, key: &str) -> bool {
        // file exists
        if self.exists(key) {
            return true;
        }

        // directory lookup - check if there are any files within the directory
        let dir = key.strip_prefix('/').unwrap_or(key);
        let dirpath = Path::new(dir);

        if self.directories.read().unwrap().contains(dirpath) {
            return true;
        }

        match self.ordered_files.binary_search_by_key(&dirpath, |fe| fe.path.as_path()) {
            // should be handled in `exists` already; something must be wrong
            Ok(_) => unreachable!(),
            Err(idx) => {
                // there exists a file in the given directory
                if self.ordered_files[idx].path.starts_with(dirpath) {
                    self.directories.write().unwrap().insert(dirpath.to_path_buf());
                    return true;
                }
            }
        }
        false
    }

    pub fn verify_entry_exists(&self, entry: &str, token: &Token, max_sev: Severity) {
        self.mark_used(&entry.replace("//", "/"));
        if !self.entry_exists(entry) {
            let msg = format!("file or directory {entry} does not exist");
            report(ErrorKey::MissingFile, Item::File.severity().at_most(max_sev))
                .msg(msg)
                .loc(token)
                .push();
        }
    }

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn verify_exists(&self, file: &Token) {
        self.mark_used(&file.as_str().replace("//", "/"));
        if !self.exists(file.as_str()) {
            let msg = "referenced file does not exist";
            report(ErrorKey::MissingFile, Item::File.severity()).msg(msg).loc(file).push();
        }
    }

    pub fn verify_exists_implied(&self, file: &str, t: &Token, max_sev: Severity) {
        self.mark_used(&file.replace("//", "/"));
        if !self.exists(file) {
            let msg = format!("file {file} does not exist");
            report(ErrorKey::MissingFile, Item::File.severity().at_most(max_sev))
                .msg(msg)
                .loc(t)
                .push();
        }
    }

    pub fn verify_exists_implied_crashes(&self, file: &str, t: &Token) {
        self.mark_used(&file.replace("//", "/"));
        if !self.exists(file) {
            let msg = format!("file {file} does not exist");
            fatal(ErrorKey::Crash).msg(msg).loc(t).push();
        }
    }

    /// # Panics
    pub fn validate(&self, _data: &Everything) {
        let common_dirs = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::tables::misc::COMMON_DIRS,
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::tables::misc::COMMON_DIRS,
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::tables::misc::COMMON_DIRS,
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => crate::hoi4::tables::misc::COMMON_DIRS,
        };
        let common_subdirs_ok = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::tables::misc::COMMON_SUBDIRS_OK,
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::tables::misc::COMMON_SUBDIRS_OK,
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::tables::misc::COMMON_SUBDIRS_OK,
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => crate::hoi4::tables::misc::COMMON_SUBDIRS_OK,
        };
        // Check the files in directories in common/ to make sure they are in known directories
        let mut warned: Vec<&Path> = Vec::new();
        'outer: for entry in &self.ordered_files {
            if !entry.path.to_string_lossy().ends_with(".txt") {
                continue;
            }
            if entry.path == PathBuf::from("common/achievement_groups.txt") {
                continue;
            }
            #[cfg(feature = "hoi4")]
            if Game::is_hoi4() {
                for valid in crate::hoi4::tables::misc::COMMON_FILES {
                    if <&str as AsRef<Path>>::as_ref(valid) == entry.path {
                        continue 'outer;
                    }
                }
            }
            let dirname = entry.path.parent().unwrap();
            if warned.contains(&dirname) {
                continue;
            }
            if !entry.path.starts_with("common") {
                // Check if the modder forgot the common/ part
                let joined = Path::new("common").join(&entry.path);
                for valid in common_dirs {
                    if joined.starts_with(valid) {
                        let msg = format!("file in unexpected directory {}", dirname.display());
                        let info = format!("did you mean common/{} ?", dirname.display());
                        err(ErrorKey::Filename).msg(msg).info(info).loc(&**entry).push();
                        warned.push(dirname);
                        continue 'outer;
                    }
                }
                continue;
            }

            for valid in common_subdirs_ok {
                if entry.path.starts_with(valid) {
                    continue 'outer;
                }
            }

            for valid in common_dirs {
                if <&str as AsRef<Path>>::as_ref(valid) == dirname {
                    continue 'outer;
                }
            }

            if entry.path.starts_with("common/scripted_values") {
                let msg = "file should be in common/script_values/";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if (Game::is_ck3() || Game::is_imperator())
                && entry.path.starts_with("common/on_actions")
            {
                let msg = "file should be in common/on_action/";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if (Game::is_vic3() || Game::is_hoi4())
                && entry.path.starts_with("common/on_action")
            {
                let msg = "file should be in common/on_actions/";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if Game::is_vic3() && entry.path.starts_with("common/modifiers") {
                let msg = "file should be in common/static_modifiers since 1.7";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if Game::is_ck3() && entry.path.starts_with("common/vassal_contracts") {
                let msg = "common/vassal_contracts was replaced with common/subject_contracts/contracts/ in 1.16";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else {
                let msg = format!("file in unexpected directory `{}`", dirname.display());
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            }
            warned.push(dirname);
        }
    }

    /// # Panics
    /// Will panic if a lock on self.used can not be obtained
    pub fn check_unused_dds(&self, _data: &Everything) {
        let mut vec = Vec::new();
        for entry in &self.ordered_files {
            let pathname = entry.path.to_string_lossy();
            if entry.path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dds"))
                && !entry.path.starts_with("gfx/interface/illustrations/loading_screens")
                && !self.used.read().unwrap().contains(pathname.as_ref())
            {
                vec.push(entry);
            }
        }
        for entry in vec {
            report(ErrorKey::UnusedFile, Severity::Untidy)
                .msg("Unused DDS files")
                .abbreviated(entry)
                .push();
        }
    }
}
