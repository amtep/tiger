use tiger_tables::datatype::*;
use tiger_tables::game::Game;

use crate::util::{HashMap, HashSet};

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
        table.iter().copied().map(|(name, args, dtype)| (name, (args, dtype))).collect()
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
        dtypes: &mut Vec<Datatype>,
        name: &str,
    ) -> Option<Vec<(Args, Vec<Datatype>)>> {
        self.get_tables(game)
            .functions
            .get(name)
            .map(|hash| Self::lookup_function_or_promote(hash, dtypes))
    }

    pub fn lookup_promote(
        &self,
        game: Game,
        dtypes: &mut Vec<Datatype>,
        name: &str,
    ) -> Option<Vec<(Args, Vec<Datatype>)>> {
        self.get_tables(game)
            .promotes
            .get(name)
            .map(|hash| Self::lookup_function_or_promote(hash, dtypes))
    }

    fn lookup_function_or_promote(
        table: &HashMap<Datatype, (Args, Datatype)>,
        dtypes: &mut Vec<Datatype>,
    ) -> Vec<(Args, Vec<Datatype>)> {
        // TODO: use a table of scope datatypes.
        let mut args_outtypes: Vec<(Args, Vec<Datatype>)> = Vec::new();
        let mut removed = vec![];

        if dtypes.contains(&Datatype::Unknown) || dtypes.contains(&Datatype::AnyScope) {
            for (index, dtype) in dtypes
                .iter()
                .enumerate()
                .filter(|(_, dtype)| !matches!(dtype, Datatype::Unknown | Datatype::AnyScope))
            {
                if !table.contains_key(dtype) {
                    removed.push(index);
                }
            }

            for (args, outtype) in table.values().copied() {
                if let Some((_, outtypes)) = args_outtypes.iter_mut().find(|(a, _)| a == &args) {
                    if !outtypes.contains(&outtype) {
                        outtypes.push(outtype);
                    }
                } else {
                    args_outtypes.push((args, vec![outtype]));
                }
            }
        } else {
            for (index, dtype) in dtypes.iter().enumerate() {
                if let Some((args, outtype)) = table.get(dtype).copied() {
                    if let Some((_, outtypes)) = args_outtypes.iter_mut().find(|(a, _)| a == &args)
                    {
                        if !outtypes.contains(&outtype) {
                            outtypes.push(outtype);
                        }
                    } else {
                        args_outtypes.push((args, vec![outtype]));
                    }
                } else {
                    removed.push(index);
                }
            }
        }

        for r in removed {
            // * remove all incompatible datatypes from possible input
            dtypes.remove(r);
        }
        args_outtypes
    }

    #[allow(clippy::unused_self)] // will need self once we add indexes
    pub fn list_promotes(
        &self,
        game: Game,
        intypes: &HashSet<Datatype>,
    ) -> impl Iterator<Item = (&'static str, Datatype, Args, Datatype)> {
        let iter = match game {
            Game::Ck3 => PROMOTES_CK3.iter(),
            Game::Vic3 => PROMOTES_VIC3.iter(),
            Game::Imperator => PROMOTES_IMPERATOR.iter(),
            Game::Eu5 => PROMOTES_EU5.iter(),
        };
        // TODO: handle AnyScope
        let unknown_ok =
            intypes.contains(&Datatype::Unknown) || intypes.contains(&Datatype::AnyScope);
        // TODO: add an index for intypes
        // This will require a function signature change, probably to a boxed dyn iterator
        iter.filter(move |(_, dtype, _, _)| unknown_ok || intypes.contains(dtype)).copied()
    }

    #[allow(clippy::unused_self)] // will need self once we add indexes
    pub fn list_functions(
        &self,
        game: Game,
        intypes: &HashSet<Datatype>,
        outtypes: &HashSet<Datatype>,
    ) -> impl Iterator<Item = (&'static str, Datatype, Args, Datatype)> {
        let iter = match game {
            Game::Ck3 => FUNCTIONS_CK3.iter(),
            Game::Vic3 => FUNCTIONS_VIC3.iter(),
            Game::Imperator => FUNCTIONS_IMPERATOR.iter(),
            Game::Eu5 => FUNCTIONS_EU5.iter(),
        };
        let unknown_in_ok =
            intypes.contains(&Datatype::Unknown) || intypes.contains(&Datatype::AnyScope);
        let unknown_out_ok =
            outtypes.contains(&Datatype::Unknown) || outtypes.contains(&Datatype::AnyScope);
        // TODO: add indexes for intypes and outtypes? Picking which to use will require some logic.
        // This will require a function signature change, probably to a boxed dyn iterator
        iter.filter(move |(_, intype, _, outtype)| {
            (unknown_in_ok || intypes.contains(intype))
                && (unknown_out_ok || outtypes.contains(outtype))
        })
        .copied()
    }

    pub fn list_global_promotes(
        &self,
        game: Game,
    ) -> impl Iterator<Item = (&'static str, Args, Datatype)> {
        match game {
            Game::Ck3 => GLOBAL_PROMOTES_CK3.iter(),
            Game::Vic3 => GLOBAL_PROMOTES_VIC3.iter(),
            Game::Imperator => GLOBAL_PROMOTES_IMPERATOR.iter(),
            Game::Eu5 => GLOBAL_PROMOTES_EU5.iter(),
        }
        .copied()
    }

    pub fn list_global_functions(
        &self,
        game: Game,
        outtypes: &HashSet<Datatype>,
    ) -> impl Iterator<Item = (&'static str, Args, Datatype)> {
        let iter = match game {
            Game::Ck3 => GLOBAL_FUNCTIONS_CK3.iter(),
            Game::Vic3 => GLOBAL_FUNCTIONS_VIC3.iter(),
            Game::Imperator => GLOBAL_FUNCTIONS_IMPERATOR.iter(),
            Game::Eu5 => GLOBAL_FUNCTIONS_EU5.iter(),
        };
        let unknown_ok =
            outtypes.contains(&Datatype::Unknown) || outtypes.contains(&Datatype::AnyScope);
        // TODO: add an index for outtypes
        // This will require a function signature change, probably to a boxed dyn iterator
        iter.filter(move |(_, _, dtype)| unknown_ok || outtypes.contains(dtype)).copied()
    }
}
