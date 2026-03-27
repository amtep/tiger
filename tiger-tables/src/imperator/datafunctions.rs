use crate::datatype::{Arg, Args, Datatype, ImperatorDatatype};
use crate::item::Item;

use Arg::*;
use Datatype::*;
use ImperatorDatatype::*;

pub const GLOBAL_PROMOTES_IMPERATOR: &[(&str, Args, Datatype)] =
    include!("include/data_global_promotes.rs");
pub const GLOBAL_FUNCTIONS_IMPERATOR: &[(&str, Args, Datatype)] =
    include!("include/data_global_functions.rs");
pub const PROMOTES_IMPERATOR: &[(&str, Datatype, Args, Datatype)] =
    include!("include/data_promotes.rs");
pub const FUNCTIONS_IMPERATOR: &[(&str, Datatype, Args, Datatype)] =
    include!("include/data_functions.rs");
