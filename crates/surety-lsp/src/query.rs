//! Hover, go to definition and document symbols. Stubs until the second
//! acceptance test; every answer is the empty one.

use std::path::Path;

use serde_json::{Value as Json, json};

pub fn hover(_path: &Path, _text: &str, _position: &Json, _graph: Option<&Path>) -> Json {
    Json::Null
}

pub fn definition(_path: &Path, _text: &str, _position: &Json) -> Json {
    Json::Null
}

pub fn symbols(_text: &str) -> Json {
    json!([])
}
