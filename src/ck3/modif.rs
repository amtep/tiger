use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::{ck3::tables::modifs::lookup_modif, modif};

bitflags! {
    // LAST UPDATED CK3 1.15.0
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKindsCk3 changes.
    #[derive(Debug, Copy, Clone)]
    #[rustfmt::skip]
    pub struct ModifKinds: u32 {
        const Character  = 1<<0;
        const Province   = 1<<3;
        const County     = 1<<5;
        const Terrain    = 1<<6;
        const Culture    = 1<<7;
        const Scheme     = 1<<8;
        const TravelPlan = 1<<9;
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
        if self.contains(ModifKinds::Province) {
            vec.push("province");
        }
        if self.contains(ModifKinds::County) {
            vec.push("county");
        }
        if self.contains(ModifKinds::Terrain) {
            vec.push("terrain");
        }
        if self.contains(ModifKinds::Culture) {
            vec.push("culture");
        }
        if self.contains(ModifKinds::Scheme) {
            vec.push("scheme");
        }
        if self.contains(ModifKinds::TravelPlan) {
            vec.push("travel plan");
        }
        write!(f, "{}", vec.join(", "))
    }
}
