#![allow(unused_imports)] // TODO EU5: remove this when ready
use crate::datatype::{Arg, Args, Datatype, Eu5Datatype};
use crate::item::Item;

use Arg::*;
use Datatype::*;
use Eu5Datatype::*;

pub const GLOBAL_PROMOTES_EU5: &[(&str, Args, Datatype)] =
    include!("include/data_global_promotes.rs");
pub const GLOBAL_FUNCTIONS_EU5: &[(&str, Args, Datatype)] =
    include!("include/data_global_functions.rs");
pub const PROMOTES_EU5: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");
pub const FUNCTIONS_EU5: &[(&str, Datatype, Args, Datatype)] =
    include!("include/data_functions.rs");
