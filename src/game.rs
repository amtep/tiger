//! Dealing with which game we are validating

use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use bitflags::bitflags;

use crate::helpers::display_choices;

/// Records at runtime which game we are validating, in case there are multiple feature flags set.
static GAME: OnceLock<Game> = OnceLock::new();

/// Enum specifying which game we are validating.
///
/// This enum is meant to be optimized away entirely when there is only one feature flag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Game {
    #[cfg(feature = "ck3")]
    Ck3,
    #[cfg(feature = "vic3")]
    Vic3,
    #[cfg(feature = "imperator")]
    Imperator,
    #[cfg(feature = "eu5")]
    Eu5,
    #[cfg(feature = "hoi4")]
    Hoi4,
}

impl Game {
    /// Decide which game we are validating. Should be called as early as possible.
    /// Returns an error if called more than once.
    pub fn set(game: Game) -> Result<()> {
        GAME.set(game).map_err(|_| anyhow!("tried to set game type twice"))?;
        Ok(())
    }

    /// Return which game we are validating. Should only be called after [`Game::set`].
    ///
    /// ## Panics
    /// Will panic if called before [`Game::set`].
    #[allow(clippy::self_named_constructors)] // not a constructor
    #[allow(unreachable_code)]
    pub fn game() -> Game {
        #[cfg(all(
            feature = "ck3",
            not(feature = "vic3"),
            not(feature = "imperator"),
            not(feature = "eu5"),
            not(feature = "hoi4")
        ))]
        return Game::Ck3;
        #[cfg(all(
            feature = "vic3",
            not(feature = "ck3"),
            not(feature = "imperator"),
            not(feature = "eu5"),
            not(feature = "hoi4")
        ))]
        return Game::Vic3;
        #[cfg(all(
            feature = "imperator",
            not(feature = "ck3"),
            not(feature = "vic3"),
            not(feature = "eu5"),
            not(feature = "hoi4")
        ))]
        return Game::Imperator;
        #[cfg(all(
            feature = "hoi4",
            not(feature = "ck3"),
            not(feature = "vic3"),
            not(feature = "imperator"),
            not(feature = "eu5"),
        ))]
        return Game::Hoi4;
        #[cfg(all(
            feature = "eu5",
            not(feature = "ck3"),
            not(feature = "vic3"),
            not(feature = "imperator"),
            not(feature = "hoi4"),
        ))]
        return Game::Eu5;
        *GAME.get().expect("internal error: don't know which game we are validating")
    }

    /// Convenience function indicating whether we are validating Crusader Kings 3 mods.
    #[inline]
    pub(crate) fn is_ck3() -> bool {
        #[cfg(not(feature = "ck3"))]
        return false;
        #[cfg(all(
            feature = "ck3",
            not(feature = "vic3"),
            not(feature = "imperator"),
            not(feature = "eu5"),
            not(feature = "hoi4")
        ))]
        return true;
        #[cfg(all(
            feature = "ck3",
            any(feature = "vic3", feature = "imperator", feature = "eu5", feature = "hoi4")
        ))]
        return GAME.get() == Some(&Game::Ck3);
    }

    /// Convenience function indicating whether we are validating Victoria 3 mods.
    #[inline]
    pub(crate) fn is_vic3() -> bool {
        #[cfg(not(feature = "vic3"))]
        return false;
        #[cfg(all(
            feature = "vic3",
            not(feature = "ck3"),
            not(feature = "imperator"),
            not(feature = "eu5"),
            not(feature = "hoi4")
        ))]
        return true;
        #[cfg(all(
            feature = "vic3",
            any(feature = "ck3", feature = "imperator", feature = "eu5", feature = "hoi4")
        ))]
        return GAME.get() == Some(&Game::Vic3);
    }

    /// Convenience function indicating whether we are validating Imperator: Rome mods.
    #[inline]
    pub(crate) fn is_imperator() -> bool {
        #[cfg(not(feature = "imperator"))]
        return false;
        #[cfg(all(
            feature = "imperator",
            not(feature = "ck3"),
            not(feature = "vic3"),
            not(feature = "eu5"),
            not(feature = "hoi4")
        ))]
        return true;
        #[cfg(all(
            feature = "imperator",
            any(feature = "ck3", feature = "vic3", feature = "hoi4", feature = "eu5")
        ))]
        return GAME.get() == Some(&Game::Imperator);
    }

    /// Convenience function indicating whether we are validating Europa Universalis 5 mods.
    #[inline]
    pub(crate) fn is_eu5() -> bool {
        #[cfg(not(feature = "eu5"))]
        return false;
        #[cfg(all(
            feature = "eu5",
            not(feature = "ck3"),
            not(feature = "vic3"),
            not(feature = "imperator"),
            not(feature = "hoi4")
        ))]
        return true;
        #[cfg(all(
            feature = "eu5",
            any(feature = "ck3", feature = "vic3", feature = "hoi4", feature = "imperator")
        ))]
        return GAME.get() == Some(&Game::Eu5);
    }

    /// Convenience function indicating whether we are validating one of the four newer games
    /// which use the Jomini scripting engine.
    #[inline]
    pub(crate) fn is_jomini() -> bool {
        Game::is_ck3() || Game::is_vic3() || Game::is_imperator() || Game::is_eu5()
    }

    /// Convenience function indicating whether we are validating Hearts of Iron 4 mods.
    #[inline]
    pub(crate) fn is_hoi4() -> bool {
        #[cfg(not(feature = "hoi4"))]
        return false;
        #[cfg(all(
            feature = "hoi4",
            not(feature = "ck3"),
            not(feature = "vic3"),
            not(feature = "imperator"),
            not(feature = "eu5")
        ))]
        return true;
        #[cfg(all(
            feature = "hoi4",
            any(feature = "ck3", feature = "vic3", feature = "imperator", feature = "eu5")
        ))]
        return GAME.get() == Some(&Game::Hoi4);
    }
}

bitflags! {
    /// A set of bitflags to indicate for which game something is intended,
    /// independent of which game we are validating.
    ///
    /// This way, error messages about things being used in the wrong game can be given at runtime.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameFlags: u8 {
        const Ck3 = 0x01;
        const Vic3 = 0x02;
        const Imperator = 0x04;
        const Eu5 = 0x08;
        const Hoi4 = 0x10;
    }
}

impl GameFlags {
    /// Get a [`GameFlags`] value representing the game being validated.
    /// Useful for checking with `.contains`.
    pub fn game() -> Self {
        // Unfortunately we have to translate between the types here.
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => GameFlags::Ck3,
            #[cfg(feature = "vic3")]
            Game::Vic3 => GameFlags::Vic3,
            #[cfg(feature = "imperator")]
            Game::Imperator => GameFlags::Imperator,
            #[cfg(feature = "eu5")]
            Game::Eu5 => GameFlags::Eu5,
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => GameFlags::Hoi4,
        }
    }

    pub const fn jomini() -> Self {
        GameFlags::Ck3.union(GameFlags::Vic3).union(GameFlags::Imperator).union(GameFlags::Eu5)
    }
}

impl Display for GameFlags {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut vec = Vec::new();
        if self.contains(Self::Ck3) {
            vec.push("Crusader Kings 3");
        }
        if self.contains(Self::Vic3) {
            vec.push("Victoria 3");
        }
        if self.contains(Self::Imperator) {
            vec.push("Imperator: Rome");
        }
        if self.contains(Self::Eu5) {
            vec.push("Europa Universalis 5");
        }
        if self.contains(Self::Hoi4) {
            vec.push("Hearts of Iron 4");
        }
        display_choices(f, &vec, "and")
    }
}

impl From<Game> for GameFlags {
    /// Convert a [`Game`] into a [`GameFlags`] with just that game's flag set.
    fn from(game: Game) -> Self {
        match game {
            #[cfg(feature = "ck3")]
            Game::Ck3 => GameFlags::Ck3,
            #[cfg(feature = "vic3")]
            Game::Vic3 => GameFlags::Vic3,
            #[cfg(feature = "imperator")]
            Game::Imperator => GameFlags::Imperator,
            #[cfg(feature = "eu5")]
            Game::Eu5 => GameFlags::Eu5,
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => GameFlags::Hoi4,
        }
    }
}
