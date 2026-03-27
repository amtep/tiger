use crate::ck3::misc::CUSTOM_RELIGION_LOCAS;
use crate::datatype::{Arg, Args, Ck3Datatype, Datatype};
use crate::item::Item;

use Arg::*;
use Ck3Datatype::*;
use Datatype::*;

const PORTRAITS: &[&str] = &[
    "left_portrait",
    "center_portrait",
    "right_portrait",
    "lower_left_portrait",
    "lower_right_portrait",
];

const XYZ: &[&str] = &["x", "y", "z"];

// The include/ files are synced with the game's data_type_* output files with the munch-data-types
// tool in utils.

pub const GLOBAL_PROMOTES_CK3: &[(&str, Args, Datatype)] =
    include!("include/data_global_promotes.rs");
pub const GLOBAL_FUNCTIONS_CK3: &[(&str, Args, Datatype)] =
    include!("include/data_global_functions.rs");
pub const PROMOTES_CK3: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");
pub const FUNCTIONS_CK3: &[(&str, Datatype, Args, Datatype)] =
    include!("include/data_functions.rs");
