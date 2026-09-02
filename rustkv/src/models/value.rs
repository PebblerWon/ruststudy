use crate::models::linked_list::LinkedList;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Ingeger(i64),
    List(LinkedList),
    Hash(HashMap<String, String>),
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Ingeger(i)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Ingeger(i) => write!(f, "{i}"),
            Value::List(list) => write!(f, "{list}"),
            Value::Hash(map) => {
                let pairs: Vec<String> = map.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                write!(f, "{{{}}}", pairs.join(","))
            }
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "String",
            Value::Ingeger(_) => "Ingeger",
            Value::List(_) => "List",
            Value::Hash(_) => "Hash",
        }
    }
}
