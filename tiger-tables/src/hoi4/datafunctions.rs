use crate::datatype::{Args, Datatype, Hoi4Datatype};

use Datatype::*;
use Hoi4Datatype::*;

pub const GLOBAL_PROMOTES_HOI4: &[(&str, Args, Datatype)] =
    include!("include/data_global_promotes.rs");
// Hoi4 does not have these
pub const GLOBAL_FUNCTIONS_HOI4: &[(&str, Args, Datatype)] = &[];
pub const PROMOTES_HOI4: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");
pub const FUNCTIONS_HOI4: &[(&str, Datatype, Args, Datatype)] =
    include!("include/data_functions.rs");
