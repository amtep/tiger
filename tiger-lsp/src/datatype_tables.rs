use std::collections::HashMap;

use tiger_tables::datatype::*;

use crate::config::Game;

#[derive(Debug)]
struct GameDatatypeTables {
    global_functions: HashMap<&'static str, (Args, Datatype)>,
    global_promotes: HashMap<&'static str, (Args, Datatype)>,
    functions: HashMap<&'static str, HashMap<Datatype, (Args, Datatype)>>,
    promotes: HashMap<&'static str, HashMap<Datatype, (Args, Datatype)>>,
}

#[derive(Debug)]
pub struct DatatypeTables {
    ck3: GameDatatypeTables,
    vic3: GameDatatypeTables,
    imperator: GameDatatypeTables,
    eu5: GameDatatypeTables,
}

impl GameDatatypeTables {
    pub fn new(game: Game) -> Self {
        match game {
            Game::Ck3 => Self {
                global_functions: Self::load_global(GLOBAL_FUNCTIONS_CK3),
                global_promotes: Self::load_global(GLOBAL_PROMOTES_CK3),
                functions: Self::load(FUNCTIONS_CK3),
                promotes: Self::load(PROMOTES_CK3),
            },
            Game::Vic3 => Self {
                global_functions: Self::load_global(GLOBAL_FUNCTIONS_VIC3),
                global_promotes: Self::load_global(GLOBAL_PROMOTES_VIC3),
                functions: Self::load(FUNCTIONS_VIC3),
                promotes: Self::load(PROMOTES_VIC3),
            },
            Game::Imperator => Self {
                global_functions: Self::load_global(GLOBAL_FUNCTIONS_IMPERATOR),
                global_promotes: Self::load_global(GLOBAL_PROMOTES_IMPERATOR),
                functions: Self::load(FUNCTIONS_IMPERATOR),
                promotes: Self::load(PROMOTES_IMPERATOR),
            },
            Game::Eu5 => Self {
                global_functions: Self::load_global(GLOBAL_FUNCTIONS_EU5),
                global_promotes: Self::load_global(GLOBAL_PROMOTES_EU5),
                functions: Self::load(FUNCTIONS_EU5),
                promotes: Self::load(PROMOTES_EU5),
            },
        }
    }

    fn load_global(
        table: &[(&'static str, Args, Datatype)],
    ) -> HashMap<&'static str, (Args, Datatype)> {
        let mut map = HashMap::default();
        for (name, args, dtype) in table.iter().copied() {
            map.insert(name, (args, dtype));
        }
        map
    }

    fn load(
        table: &[(&'static str, Datatype, Args, Datatype)],
    ) -> HashMap<&'static str, HashMap<Datatype, (Args, Datatype)>> {
        let mut map: HashMap<_, HashMap<_, _>> = HashMap::default();
        for (name, from, args, to) in table.iter().copied() {
            map.entry(name).or_default().insert(from, (args, to));
        }
        map
    }
}

impl DatatypeTables {
    pub fn new() -> Self {
        Self {
            ck3: GameDatatypeTables::new(Game::Ck3),
            vic3: GameDatatypeTables::new(Game::Vic3),
            imperator: GameDatatypeTables::new(Game::Imperator),
            eu5: GameDatatypeTables::new(Game::Eu5),
        }
    }

    fn get_tables(&self, game: Game) -> &GameDatatypeTables {
        match game {
            Game::Ck3 => &self.ck3,
            Game::Vic3 => &self.vic3,
            Game::Imperator => &self.imperator,
            Game::Eu5 => &self.eu5,
        }
    }

    pub fn lookup_global_function(&self, game: Game, name: &str) -> Option<(Args, Datatype)> {
        self.get_tables(game).global_functions.get(name).copied()
    }

    pub fn lookup_global_promote(&self, game: Game, name: &str) -> Option<(Args, Datatype)> {
        self.get_tables(game).global_promotes.get(name).copied()
    }

    pub fn lookup_function(
        &self,
        game: Game,
        dtype: Datatype,
        name: &str,
    ) -> Option<(Args, Datatype)> {
        self.get_tables(game)
            .functions
            .get(name)
            .and_then(|hash| Self::lookup_function_or_promote(hash, dtype))
    }

    pub fn lookup_promote(
        &self,
        game: Game,
        dtype: Datatype,
        name: &str,
    ) -> Option<(Args, Datatype)> {
        self.get_tables(game)
            .promotes
            .get(name)
            .and_then(|hash| Self::lookup_function_or_promote(hash, dtype))
    }

    fn lookup_function_or_promote(
        table: &HashMap<Datatype, (Args, Datatype)>,
        dtype: Datatype,
    ) -> Option<(Args, Datatype)> {
        // TODO: use a table of scope datatypes.
        if dtype == Datatype::Unknown || dtype == Datatype::AnyScope {
            let mut args = None;
            let mut outtype = None;
            for (a, o) in table.values() {
                if args.is_none() {
                    args = Some(*a);
                    outtype = Some(*o);
                } else {
                    if args != Some(*a) {
                        args = Some(Args::Unknown);
                    }
                    if outtype != Some(*o) {
                        outtype = Some(Datatype::Unknown);
                    }
                }
            }
            if let Some(args) = args
                && let Some(outtype) = outtype
            {
                Some((args, outtype))
            } else {
                None
            }
        } else {
            table.get(&dtype).copied()
        }
    }
}
