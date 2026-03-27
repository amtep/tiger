use crate::datatype::{Arg, Args, Datatype, Vic3Datatype};
use crate::item::Item;

use Arg::*;
use Datatype::*;
use Vic3Datatype::*;

// The include/ files are converted from the game's data_type_* output files.
pub const GLOBAL_PROMOTES_VIC3: &[(&str, Args, Datatype)] =
    include!("include/data_global_promotes.rs");
pub const GLOBAL_FUNCTIONS_VIC3: &[(&str, Args, Datatype)] =
    include!("include/data_global_functions.rs");
pub const PROMOTES_VIC3: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");
pub const FUNCTIONS_VIC3: &[(&str, Datatype, Args, Datatype)] =
    include!("include/data_functions.rs");
