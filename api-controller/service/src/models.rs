use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub runtime: String,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    function_name: String,
    runtime: String,
    pub(crate) env: Option<HashMap<String, String>>,
}
