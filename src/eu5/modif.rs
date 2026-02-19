use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::eu5::tables::modifs::lookup_modif;

bitflags! {
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKinds changes.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[rustfmt::skip]
    pub struct ModifKinds: u32 {
        const None = 1<<0;
        const Character = 1<<1;
        const Country = 1<<2;
        const Dynasty = 1<<3;
        const InternationalOrganization = 1<<4;
        const Location = 1<<5;
        const Mercenary = 1<<6;
        const Province = 1<<7;
        const Rebel = 1<<8;
        const Religion = 1<<9;
        const Unit = 1<<10;
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
        if self.contains(ModifKinds::None) {
            vec.push("None");
        }
        if self.contains(ModifKinds::Character) {
            vec.push("Character");
        }
        if self.contains(ModifKinds::Country) {
            vec.push("Country");
        }
        if self.contains(ModifKinds::Dynasty) {
            vec.push("Dynasty");
        }
        if self.contains(ModifKinds::InternationalOrganization) {
            vec.push("InternationalOrganization");
        }
        if self.contains(ModifKinds::Location) {
            vec.push("Location");
        }
        if self.contains(ModifKinds::Mercenary) {
            vec.push("Mercenary");
        }
        if self.contains(ModifKinds::Province) {
            vec.push("Province");
        }
        if self.contains(ModifKinds::Rebel) {
            vec.push("Rebel");
        }
        if self.contains(ModifKinds::Religion) {
            vec.push("Religion");
        }
        if self.contains(ModifKinds::Unit) {
            vec.push("Unit");
        }
        write!(f, "{}", vec.join(", "))
    }
}
