//! Dealing with which game we are validating

use std::sync::OnceLock;

use serde_derive::Deserialize;

/// Records at runtime which game we are validating, in case there are multiple feature flags set.
static GAME: OnceLock<Game> = OnceLock::new();

/// Enum specifying which game we are validating.
///
/// This enum is meant to be optimized away entirely when there is only one feature flag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    pub fn set(game: Game) -> Result<(), String> {
        GAME.set(game).map_err(|_| "tried to set game type twice".to_string())?;
        Ok(())
    }

    /// Return which game we are validating. Should only be called after [`Game::set`].
    ///
    /// ## Panics
    /// May panic if called before [`Game::set`].
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
    pub fn is_ck3() -> bool {
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
    pub fn is_vic3() -> bool {
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
    pub fn is_imperator() -> bool {
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
    pub fn is_eu5() -> bool {
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

    /// Convenience function indicating whether we are validating one of the newer games
    /// which use the Jomini scripting engine.
    #[inline]
    pub fn is_jomini() -> bool {
        Game::is_ck3() || Game::is_vic3() || Game::is_imperator() || Game::is_eu5()
    }

    /// Convenience function indicating whether we are validating Hearts of Iron 4 mods.
    #[inline]
    pub fn is_hoi4() -> bool {
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
