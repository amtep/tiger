extern crate tiger_lib;
use tiger_lib::Game;

fn main() {
    #[cfg(feature = "ck3")]
    Game::set(Game::Ck3).unwrap();
    #[cfg(feature = "vic3")]
    Game::set(Game::Vic3).unwrap();
    divan::main();
}
