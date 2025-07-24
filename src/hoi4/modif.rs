use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::{hoi4::tables::modifs::lookup_modif, modif};

bitflags! {
    // LAST UPDATED HOI4 1.16.4
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKinds changes.
    #[derive(Debug, Copy, Clone)]
    #[rustfmt::skip]
    pub struct ModifKinds: u32 {
        const Character            = 1<<0;
        const Country              = 1<<1;
        const State                = 1<<2;
        const Aggressive           = 1<<5;
        const Ai                   = 1<<6;
        const Air                  = 1<<7;
        const Army                 = 1<<8;
        const Autonomy             = 1<<9;
        const Defensive            = 1<<10;
        const GovernmentInExile    = 1<<11;
        const IntelligenceAgency   = 1<<12;
        const MilitaryAdvancements = 1<<13;
        const Naval                = 1<<14;
        const Peace                = 1<<15;
        const Politics             = 1<<16;
        const Scientist            = 1<<17;
        const UnitLeader           = 1<<18;
        const WarProduction        = 1<<19;
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
        if self.contains(ModifKinds::Aggressive) {
            vec.push("aggressive");
        }
        if self.contains(ModifKinds::Ai) {
            vec.push("ai");
        }
        if self.contains(ModifKinds::Air) {
            vec.push("air");
        }
        if self.contains(ModifKinds::Army) {
            vec.push("army");
        }
        if self.contains(ModifKinds::Autonomy) {
            vec.push("autonomy");
        }
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Country) {
            vec.push("country");
        }
        if self.contains(ModifKinds::Defensive) {
            vec.push("defensive");
        }
        if self.contains(ModifKinds::GovernmentInExile) {
            vec.push("government in exile");
        }
        if self.contains(ModifKinds::IntelligenceAgency) {
            vec.push("intelligence agency");
        }
        if self.contains(ModifKinds::MilitaryAdvancements) {
            vec.push("military advancements");
        }
        if self.contains(ModifKinds::Naval) {
            vec.push("naval");
        }
        if self.contains(ModifKinds::Peace) {
            vec.push("peace");
        }
        if self.contains(ModifKinds::Politics) {
            vec.push("politics");
        }
        if self.contains(ModifKinds::Scientist) {
            vec.push("scientist");
        }
        if self.contains(ModifKinds::State) {
            vec.push("state");
        }
        if self.contains(ModifKinds::UnitLeader) {
            vec.push("unit leader");
        }
        if self.contains(ModifKinds::WarProduction) {
            vec.push("war production");
        }
        write!(f, "{}", vec.join(", "))
    }
}
