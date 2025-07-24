use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::{modif, vic3::tables::modifs::lookup_modif};

bitflags! {
    // LAST UPDATED VIC3 1.7.0
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKinds changes.
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[rustfmt::skip]
    pub struct ModifKinds: u32 {
        const Character         = 1<<0;
        const Country           = 1<<1;
        const State             = 1<<2;
        const Unit              = 1<<4;
        const Battle            = 1<<5;
        const Building          = 1<<6;
        const InterestGroup     = 1<<7;
        const Market            = 1<<8;
        const PoliticalMovement = 1<<9;
        const Tariff            = 1<<10;
        const Tax               = 1<<11;
        const Goods             = 1<<12;
        const MilitaryFormation = 1<<13;
        const PowerBloc         = 1<<14;
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
        if self.contains(ModifKinds::Battle) {
            vec.push("battle");
        }
        if self.contains(ModifKinds::Building) {
            vec.push("building");
        }
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Country) {
            vec.push("country");
        }
        if self.contains(ModifKinds::InterestGroup) {
            vec.push("interest group");
        }
        if self.contains(ModifKinds::Market) {
            vec.push("market");
        }
        if self.contains(ModifKinds::PoliticalMovement) {
            vec.push("political movement");
        }
        if self.contains(ModifKinds::State) {
            vec.push("state");
        }
        if self.contains(ModifKinds::Tariff) {
            vec.push("tariff");
        }
        if self.contains(ModifKinds::Tax) {
            vec.push("tax");
        }
        if self.contains(ModifKinds::Unit) {
            vec.push("unit");
        }
        if self.contains(ModifKinds::Goods) {
            vec.push("goods");
        }
        if self.contains(ModifKinds::MilitaryFormation) {
            vec.push("military formation");
        }
        if self.contains(ModifKinds::PowerBloc) {
            vec.push("power bloc");
        }
        write!(f, "{}", vec.join(", "))
    }
}
