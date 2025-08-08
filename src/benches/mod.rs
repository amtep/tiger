use std::path::PathBuf;

use crate::game::Game;

pub mod ck3;
pub mod vic3;

pub fn bench_mods<'a>() -> impl Iterator<Item = (&'a str, &'a PathBuf)> {
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => ck3::bench_mods(),
        #[cfg(feature = "vic3")]
        Game::Vic3 => vic3::bench_mods(),
        #[cfg(feature = "imperator")]
        Game::Imperator => unimplemented!(),
        #[cfg(feature = "hoi4")]
        Game::Hoi4 => unimplemented!(),
    }
}
