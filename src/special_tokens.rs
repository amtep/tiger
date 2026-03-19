use crate::helpers::TigerHashSet;
use crate::token::{Token, TokenIdentity};

#[derive(Debug, Clone)]
pub struct SpecialTokens {
    st: Option<TigerHashSet<TokenIdentity>>,
}

impl SpecialTokens {
    pub fn none() -> Self {
        SpecialTokens { st: None }
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        SpecialTokens { st: Some(TigerHashSet::default()) }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &Token> {
        self.st.iter().flat_map(|set| set.iter().map(TokenIdentity::inner))
    }

    #[allow(dead_code)]
    pub fn into_iter(self) -> impl IntoIterator<Item = Token> {
        self.st.into_iter().flat_map(|set| set.into_iter().map(TokenIdentity::into_inner))
    }

    pub fn insert<T: Into<Token>>(&mut self, token: T) {
        if let Some(ref mut st) = self.st {
            st.insert(TokenIdentity::new(token.into()));
        }
    }

    pub fn merge(&mut self, other: &Self) {
        for token in other.iter() {
            self.insert(token);
        }
    }
}
