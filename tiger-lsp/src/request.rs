use std::fmt::Write;

use serde_derive::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Map<String, Value>,
}

impl Request {
    pub fn dump(&self) -> String {
        format!("ID {} METHOD {}\n{}", &self.id, &self.method, dump_object(&self.params, "  "))
    }
}

fn dump_object(map: &Map<String, Value>, indent: &str) -> String {
    let mut result = String::new();
    for (key, value) in map {
        if let Some(map) = value.as_object() {
            let new_indent = format!("{indent}  ");
            _ = write!(&mut result, "{indent}{key} = MAP\n{}", dump_object(map, &new_indent));
        } else {
            _ = writeln!(&mut result, "{indent}{key} = {value}");
        }
    }
    result
}
