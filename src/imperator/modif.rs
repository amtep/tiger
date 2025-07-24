use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::{imperator::tables::modifs::lookup_modif, modif};

bitflags! {
    // LAST UPDATED IMPERATOR 2.0.4
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKindsImperator changes.
    #[derive(Debug, Copy, Clone)]
    #[rustfmt::skip] // table looks better with cfg on same line
    pub struct ModifKinds: u32 {
        const Character      = 1<<0;
        const Country        = 1<<1;
        const State          = 1<<2;
        const Province       = 1<<3;
        const Unit           = 1<<4;
        const Legion         = 1<<5;
        const CountryCulture = 1<<6;
    }
}

impl modif::ModifKinds for ModifKinds {
    fn lookup_modif(
        name: &crate::Token,
        data: &crate::Everything,
        warn: Option<crate::Severity>,
    ) -> Option<Self> {
        lookup_modif(name, data, warn)
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut vec = Vec::new();
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Country) {
            vec.push("country");
        }
        if self.contains(ModifKinds::Province) {
            vec.push("province");
        }
        if self.contains(ModifKinds::State) {
            vec.push("state");
        }
        if self.contains(ModifKinds::Unit) {
            vec.push("unit");
        }
        if self.contains(ModifKinds::Legion) {
            vec.push("legion");
        }
        if self.contains(ModifKinds::CountryCulture) {
            vec.push("country culture");
        }
        write!(f, "{}", vec.join(", "))
    }
}
