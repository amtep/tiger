use anyhow::Result;
use eu5_tiger::GAME_CONSTS;
use tiger_bin_shared::auto;

fn main() -> Result<()> {
    auto(GAME_CONSTS)
}
