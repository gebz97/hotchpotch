use std::collections::BTreeMap; // BTreeMap keeps key order deterministic

/// Internal representation of any serializable value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),  // preserves insertion order via BTree sort
}