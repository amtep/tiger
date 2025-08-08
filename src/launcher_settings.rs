//! Loader for the `launcher-settings.json` file.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::files::{FileEntry, FileKind};
use crate::game::Game;
use crate::parse::json::parse_json_file;

/// Looks up the game's version in the launcher settings.
pub fn get_version_from_launcher(game_dir: &Path) -> Result<String> {
    let launcher_pathname = if Game::is_hoi4() {
        game_dir.join("launcher-settings.json")
    } else {
        game_dir.join("launcher/launcher-settings.json")
    };
    let launcher_entry = FileEntry::new_untracked(
        launcher_pathname.clone(),
        FileKind::Vanilla,
        launcher_pathname.clone(),
    );
    let block = parse_json_file(&launcher_entry).with_context(|| {
        format!("Could not parse launcher file {}", launcher_pathname.display())
    })?;
    if let Some(topblock) = block.iter_blocks().next() {
        if let Some(version) = topblock.get_field_value("rawVersion") {
            return Ok(version.as_str().to_owned());
        }
    }
    bail!("Version not found in {}", launcher_pathname.display())
}
