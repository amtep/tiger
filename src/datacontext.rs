//! [`DataContext`] tracks what we know about the GUI datamodels and datacontexts.
//! Currently it only tracks the `ScriptedGui` name.

use crate::token::Token;

#[derive(Debug, Clone)]
pub struct DataContext {
    sgui_name: Option<Token>,
}

impl DataContext {
    pub fn new() -> Self {
        Self { sgui_name: None }
    }

    pub fn set_sgui_name(&mut self, name: Token) {
        self.sgui_name = Some(name);
    }

    #[allow(dead_code)]
    pub fn sgui_name(&self) -> Option<&Token> {
        self.sgui_name.as_ref()
    }
}
