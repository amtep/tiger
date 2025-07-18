mod auto;
mod gamedir;
mod tiger;
mod update;
mod version;

/// String constants associated with the game being verified.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct GameConsts {
    /// Full name
    pub name: &'static str,
    /// Shortened name
    pub name_short: &'static str,
    /// Latest supported version
    pub version: &'static str,
    /// steam ID
    pub app_id: u32,
    /// A file that should be present if this is the correct game directory
    pub signature_file: &'static str,
    /// The directory under the Paradox Interactive directory for local files
    pub paradox_dir: &'static str,
}

pub use auto::run as auto;
pub use tiger::run as tiger;
