use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::eu5::tables::modifs::lookup_modif;

bitflags! {
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKinds changes.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[rustfmt::skip]
    pub struct ModifKinds: u32 {
        const Character         = 1<<0;
        const Country           = 1<<1;
        // TODO: EU5 fill in table
    }
}

impl crate::modif::ModifKinds for ModifKinds {
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
        // TODO: EU5 add new modifkinds here
        write!(f, "{}", vec.join(", "))
    }
}
