use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::bail;
use anyhow::Result;
use itertools::Itertools as _;
use walkdir::WalkDir;

use super::filedb::FileDb;
use super::fileset::FilePaths;
use super::fileset::LoadedMod;
use super::FileKind;
use crate::add_loaded_mod_root;
use crate::block::Block;
use crate::everything::FilesError;
use crate::files::FileEntry;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::add_loaded_dlc_root;
use crate::util::fix_slashes_for_target_platform;
use crate::Fileset;
use crate::Game;
use crate::Loc;
#[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
use crate::ModFile;
#[cfg(feature = "vic3")]
use crate::ModMetadata;
use crate::TigerHashMap;
use crate::TigerHashSet;
use crate::Token;

#[derive(Debug)]
pub struct FilesetBuilder {
    paths: FilePaths,
    db: FileDb,
}

impl FilesetBuilder {
    pub(crate) fn new(vanilla_dir: Option<&Path>) -> Self {
        Self {
            paths: FilePaths {
                vanilla_root: if Game::is_jomini() {
                    vanilla_dir.map(|dir| dir.join("game"))
                } else {
                    vanilla_dir.map(ToOwned::to_owned)
                },
                #[cfg(feature = "jomini")]
                clausewitz_root: vanilla_dir.map(|dir| dir.join("clausewitz")),
                #[cfg(feature = "jomini")]
                jomini_root: vanilla_dir.map(|dir| dir.join("jomini")),
            },
            db: FileDb::default(),
        }
    }

    #[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
    pub fn with_modfile(mut self, mut modpath: PathBuf) -> Result<FilesetBuilderWithMod> {
        if modpath.is_dir() {
            modpath.push("descriptor.mod");
        }
        let modfile_entry = self.db.get_or_create_entry(modpath.clone(), FileKind::Mod, modpath);
        let modfile = ModFile::read(modfile_entry)?;

        let modpath = modfile.modpath();
        if modpath.exists() {
            eprintln!("Using mod directory: {}", modpath.display());
        } else {
            eprintln!("Looking for mod in {}", modpath.display());
            bail!("Cannot find mod directory. Please make sure the .mod file is correct.");
        }

        let replace_paths = modfile.replace_paths();

        Ok(FilesetBuilderWithMod {
            paths: self.paths,
            db: self.db,
            the_mod: LoadedMod::new_main_mod(modfile.modpath(), replace_paths),
        })
    }

    #[cfg(feature = "vic3")]
    pub fn with_metadata(mut self, mod_root: PathBuf) -> Result<FilesetBuilderWithMod> {
        let metadata = ModMetadata::read(&mut self.db, mod_root)?;

        eprintln!("Using mod directory: {}", metadata.modpath().display());

        let replace_paths = metadata.replace_paths();

        Ok(FilesetBuilderWithMod {
            paths: self.paths,
            db: self.db,
            the_mod: LoadedMod::new_main_mod(metadata.modpath().to_path_buf(), replace_paths),
        })
    }
}

#[derive(Debug)]
pub struct FilesetBuilderWithMod {
    paths: FilePaths,
    db: FileDb,
    pub(crate) the_mod: LoadedMod,
}

impl FilesetBuilderWithMod {
    pub fn config(
        mut self,
        config_file: PathBuf,
        #[allow(unused_variables)] workshop_dir: Option<&Path>,
        #[allow(unused_variables)] paradox_dir: Option<&Path>,
    ) -> Result<FilesetBuilderWithConfig> {
        let config_entry =
            self.db.get_or_create_entry(config_file.clone(), FileKind::Mod, config_file);
        let config = if config_entry.is_file() {
            PdxFile::read_optional_bom(config_entry, &ParserMemory::default())
                .ok_or(FilesError::ConfigUnreadable { path: config_entry.path().to_path_buf() })?
        } else {
            Block::new(Loc::from(config_entry))
        };

        let mut loaded_mods = vec![];

        let config_path = config.loc.fullpath();
        for block in config.get_field_blocks("load_mod") {
            let mod_idx;
            if let Ok(idx) = u8::try_from(loaded_mods.len()) {
                mod_idx = idx;
            } else {
                bail!("too many loaded mods, cannot process more");
            }

            let default_label = || format!("MOD{mod_idx}");
            let label =
                block.get_field_value("label").map_or_else(default_label, ToString::to_string);

            if Game::is_ck3() || Game::is_imperator() || Game::is_hoi4() {
                #[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
                if let Some(path) = get_modfile(&label, config_path, block, paradox_dir) {
                    let kind = FileKind::LoadedMod(mod_idx);
                    let modfile_entry = self.db.get_or_create_entry(path.clone(), kind, path);
                    let modfile = ModFile::read(modfile_entry)?;
                    eprintln!(
                        "Loading secondary mod {label} from: {}{}",
                        modfile.modpath().display(),
                        modfile
                            .display_name()
                            .map_or_else(String::new, |name| format!(" \"{name}\"")),
                    );
                    let loaded_mod = LoadedMod::new(
                        kind,
                        label.clone(),
                        modfile.modpath().clone(),
                        modfile.replace_paths(),
                    );
                    add_loaded_mod_root(label);
                    loaded_mods.push(loaded_mod);
                } else {
                    bail!("could not load secondary mod from config; missing valid `modfile` or `workshop_id` field");
                }
            } else if Game::is_vic3() {
                #[cfg(feature = "vic3")]
                if let Some(pathdir) = get_mod(&label, config_path, block, workshop_dir) {
                    match ModMetadata::read(&mut self.db, pathdir.clone()) {
                        Ok(metadata) => {
                            eprintln!(
                                "Loading secondary mod {label} from: {}{}",
                                pathdir.display(),
                                metadata
                                    .display_name()
                                    .map_or_else(String::new, |name| format!(" \"{name}\"")),
                            );
                            let kind = FileKind::LoadedMod(mod_idx);
                            let loaded_mod = LoadedMod::new(
                                kind,
                                label.clone(),
                                pathdir,
                                metadata.replace_paths(),
                            );
                            add_loaded_mod_root(label);
                            loaded_mods.push(loaded_mod);
                        }
                        Err(e) => {
                            eprintln!(
                                "could not load secondary mod {label} from: {}",
                                pathdir.display()
                            );
                            eprintln!("  because: {e}");
                        }
                    }
                } else {
                    bail!("could not load secondary mod from config; missing valid `mod` or `workshop_id` field");
                }
            }
        }
        Ok(FilesetBuilderWithConfig {
            paths: self.paths,
            db: self.db,
            config,
            the_mod: self.the_mod,
            loaded_mods,
            loaded_dlcs: vec![],
        })
    }
}

#[derive(Debug)]
pub struct FilesetBuilderWithConfig {
    paths: FilePaths,
    db: FileDb,
    config: Block,
    the_mod: LoadedMod,
    loaded_mods: Vec<LoadedMod>,
    loaded_dlcs: Vec<LoadedMod>,
}

impl FilesetBuilderWithConfig {
    fn should_replace(&self, path: &Path, kind: FileKind) -> bool {
        if kind == FileKind::Mod {
            return false;
        }
        if kind < FileKind::Mod && self.the_mod.should_replace(path) {
            return true;
        }
        for loaded_mod in &self.loaded_mods {
            if kind < loaded_mod.kind() && loaded_mod.should_replace(path) {
                return true;
            }
        }
        false
    }

    fn scan(&mut self, path: &Path, kind: FileKind) -> Result<(), walkdir::Error> {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.depth() == 0 || !entry.file_type().is_file() {
                continue;
            }
            // unwrap is safe here because WalkDir gives us paths with this prefix.
            let inner_path = entry.path().strip_prefix(path).unwrap();
            if inner_path.starts_with(".git") {
                continue;
            }
            let inner_dir = inner_path.parent().unwrap_or_else(|| Path::new(""));
            if self.should_replace(inner_dir, kind) {
                continue;
            }
            self.db.files.entry(entry.path().to_path_buf()).or_insert_with(|| {
                FileEntry::new(inner_path.to_path_buf(), kind, entry.path().to_path_buf())
            });
        }
        Ok(())
    }

    pub fn scan_all(&mut self) -> Result<(), FilesError> {
        #[cfg(feature = "jomini")]
        if let Some(clausewitz_root) = self.paths.clausewitz_root.clone() {
            self.scan(&clausewitz_root.clone(), FileKind::Clausewitz).map_err(|e| {
                FilesError::VanillaUnreadable { path: clausewitz_root.clone(), source: e }
            })?;
        }
        #[cfg(feature = "jomini")]
        if let Some(jomini_root) = &self.paths.jomini_root.clone() {
            self.scan(&jomini_root.clone(), FileKind::Jomini).map_err(|e| {
                FilesError::VanillaUnreadable { path: jomini_root.clone(), source: e }
            })?;
        }
        if let Some(vanilla_root) = &self.paths.vanilla_root.clone() {
            self.scan(&vanilla_root.clone(), FileKind::Vanilla).map_err(|e| {
                FilesError::VanillaUnreadable { path: vanilla_root.clone(), source: e }
            })?;
            #[cfg(feature = "hoi4")]
            if Game::is_hoi4() {
                self.load_dlcs(&vanilla_root.join("integrated_dlc"))?;
            }
            self.load_dlcs(&vanilla_root.join("dlc"))?;
        }
        // loaded_mods is cloned here for the borrow checker
        for loaded_mod in &self.loaded_mods.clone() {
            self.scan(loaded_mod.root(), loaded_mod.kind()).map_err(|e| {
                FilesError::ModUnreadable { path: loaded_mod.root().to_path_buf(), source: e }
            })?;
        }
        #[allow(clippy::unnecessary_to_owned)] // borrow checker requires to_path_buf here
        self.scan(&self.the_mod.root().to_path_buf(), FileKind::Mod).map_err(|e| {
            FilesError::ModUnreadable { path: self.the_mod.root().to_path_buf(), source: e }
        })?;
        Ok(())
    }

    /// # Panics
    /// Will panic if more than 256 DLCs are installed
    pub fn load_dlcs(&mut self, dlc_root: &Path) -> Result<(), FilesError> {
        for entry in WalkDir::new(dlc_root).max_depth(1).sort_by_file_name().into_iter().flatten() {
            if entry.depth() == 1 && entry.file_type().is_dir() {
                let label = entry.file_name().to_string_lossy().to_string();
                let idx =
                    u8::try_from(self.loaded_dlcs.len()).expect("more than 256 DLCs installed");
                let dlc = LoadedMod::new(
                    FileKind::Dlc(idx),
                    label.clone(),
                    entry.path().to_path_buf(),
                    Vec::new(),
                );
                self.scan(dlc.root(), dlc.kind()).map_err(|e| FilesError::VanillaUnreadable {
                    path: dlc.root().to_path_buf(),
                    source: e,
                })?;
                self.loaded_dlcs.push(dlc);
                add_loaded_dlc_root(label);
            }
        }
        Ok(())
    }

    pub fn finalize(self) -> Fileset {
        // This sorts by pathname but where pathnames are equal it places `Mod` entries after `Vanilla` entries
        // and `LoadedMod` entries between them in order
        let sorted = self.db.files.values().sorted();

        let mut ordered_files: Vec<Arc<FileEntry>> = vec![];

        // When there are identical paths, only keep the last entry of them.
        for entry in sorted {
            if let Some(prev) = ordered_files.last_mut() {
                if entry.path() == prev.path() {
                    *prev = Arc::clone(entry);
                } else {
                    ordered_files.push(Arc::clone(entry));
                }
            } else {
                ordered_files.push(Arc::clone(entry));
            }
        }

        let mut filename_tokens = vec![];
        let mut filenames = TigerHashMap::default();
        for entry in &ordered_files {
            entry.get_or_init_path_idx();
            let token = Token::new(&entry.filename().to_string_lossy(), entry.into());
            filename_tokens.push(token);
            filenames.insert(entry.path().to_path_buf(), Arc::clone(entry));
        }

        Fileset {
            paths: self.paths,
            the_mod: self.the_mod,
            loaded_mods: self.loaded_mods,
            loaded_dlcs: self.loaded_dlcs,
            config: self.config,
            db: self.db,
            ordered_files,
            filename_tokens,
            filenames,
            directories: RwLock::new(TigerHashSet::default()),
            used: RwLock::new(TigerHashSet::default()),
        }
    }
}

#[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
fn get_modfile(
    label: &String,
    config_path: &Path,
    block: &Block,
    paradox_dir: Option<&Path>,
) -> Option<PathBuf> {
    let mut path: Option<PathBuf> = None;
    if let Some(modfile) = block.get_field_value("modfile") {
        let modfile_path = fix_slashes_for_target_platform(
            config_path
                .parent()
                .unwrap() // SAFETY: known to be for a file in a directory
                .join(modfile.as_str()),
        );
        if modfile_path.exists() {
            path = Some(modfile_path);
        } else {
            eprintln!("Could not find mod {label} at: {}", modfile_path.display());
        }
    }
    if path.is_none() {
        if let Some(workshop_id) = block.get_field_value("workshop_id") {
            match paradox_dir {
                Some(p) => {
                    path = Some(fix_slashes_for_target_platform(
                        p.join(format!("mod/ugc_{workshop_id}.mod")),
                    ));
                }
                None => eprintln!("workshop_id defined, but could not find paradox directory"),
            }
        }
    }
    path
}

#[cfg(feature = "vic3")]
fn get_mod(
    label: &String,
    config_path: &Path,
    block: &Block,
    workshop_dir: Option<&Path>,
) -> Option<PathBuf> {
    let mut path: Option<PathBuf> = None;
    if let Some(modfile) = block.get_field_value("mod") {
        let mod_path = fix_slashes_for_target_platform(
            config_path
                .parent()
                .unwrap() // SAFETY: known to be for a file in a directory
                .join(modfile.as_str()),
        );
        if mod_path.exists() {
            path = Some(mod_path);
        } else {
            eprintln!("Could not find mod {label} at: {}", mod_path.display());
        }
    }
    if path.is_none() {
        if let Some(workshop_id) = block.get_field_value("workshop_id") {
            match workshop_dir {
                Some(w) => {
                    path = Some(fix_slashes_for_target_platform(w.join(workshop_id.as_str())));
                }
                None => eprintln!("workshop_id defined, but could not find workshop"),
            }
        }
    }
    path
}
