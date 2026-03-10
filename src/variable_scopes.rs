use scc::HashMap as SccHashMap;

use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;

/// A registry of what is known about the scope types of variables.
/// There will be one registry per namespace (globals, global lists, variables, variable lists).
#[derive(Debug)]
pub struct VariableScopes {
    /// The string to be used when reporting scope conflicts to the user.
    namespace: &'static str,
    /// The registry. It is a hashmap that allows concurrent access and has internal mutability.
    /// The values are the scope types, and a bool that is true if the scope was overridden in
    /// configuration or false if the scope was deduced.
    scopes: SccHashMap<&'static str, (Scopes, bool)>,
}

impl VariableScopes {
    pub fn new(namespace: &'static str) -> Self {
        Self { namespace, scopes: SccHashMap::default() }
    }

    #[allow(dead_code)]
    pub fn config_override(&self, name: &'static str, scopes: Scopes) {
        self.scopes.upsert_sync(name, (scopes, true));
    }

    pub fn scopes(&self, name: &str) -> Scopes {
        self.scopes.read_sync(name, |_, (s, _)| *s).unwrap_or(Scopes::all())
    }

    pub fn expect(&self, name: &'static str, token: &Token, scopes: Scopes) {
        self.scopes
            .entry_sync(name)
            .and_modify(|(s, overridden)| {
                if s.intersects(scopes) {
                    if !*overridden {
                        *s &= scopes;
                    }
                } else {
                    let verb = if *overridden { "configured" } else { "deduced" };
                    let msg = format!(
                        "{}{name} was {verb} to be {s} but scope seems to be {scopes}",
                        self.namespace
                    );
                    warn(ErrorKey::Scopes).weak().msg(msg).loc(token).push();
                }
            })
            .or_insert((scopes, false));
    }
}
