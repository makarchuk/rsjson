use std::collections::HashMap;
use std::str::FromStr;

mod parser;

#[derive(Debug, PartialEq)]
pub enum JSONValue {
    JSONNull(),
    JSONString(String),
    JSONBool(bool),
    JSONNumber(f64),
    JSONObject(HashMap<String, Box<JSONValue>>),
    JSONArray(Vec<Box<JSONValue>>),
}

#[derive(Debug, Clone)]
pub struct JSONParseError {
    pub reason: String,
}

impl FromStr for JSONValue {
    type Err = JSONParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return parser::parse_json(s);
    }
}
