pub const DLC_FEATURES_EU5: &[&str] = &[];

/// A list of music provided by DLCs, for people who don't have them
pub const DLC_MUSIC: &[&str] = &[];

pub const COMMON_DIRS: &[&str] = &[
    // TODO: EU5 fill in table
    // Note that parent directories should not be listed except for the COMMON_SUBDIRS_OK ones,
    // where only the parent directory should be listed.
    "common/insults",
];

// TODO: EU5 verify
pub const COMMON_SUBDIRS_OK: &[&str] = &["common/defines", "common/history"];
