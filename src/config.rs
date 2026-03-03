use pyo3::prelude::*;

/*
All delimiters that define the serialization format.
Defaults match the example: name=adam;hobbies=[cycling|rowing|chess];height=175cm;
 */
#[pyclass]
#[derive(Clone, Debug)]
pub struct FormatConfig {
    // between key=value pairs  → ';'
    #[pyo3(get, set)]
    pub field_sep: char,
    // between key and value    → '='
    #[pyo3(get, set)]
    pub kv_sep: char,
    // list open bracket        → '['
    #[pyo3(get, set)]
    pub list_open: char,
    // list close bracket       → ']'
    #[pyo3(get, set)]
    pub list_close: char,
    // between list items       → '|'
    #[pyo3(get, set)]
    pub list_sep: char,
    // nested object open       → '{'
    #[pyo3(get, set)]
    pub obj_open: char,
    // nested object close      → '}'
    #[pyo3(get, set)]
    pub obj_close: char,
    // escape character         → '\'
    #[pyo3(get, set)]
    pub escape: char,
    // null representation      → 'null'
    #[pyo3(get, set)]
    pub null_str: String,
    // bool true representation → 'true'
    #[pyo3(get, set)]
    pub true_str: String,
    // bool false representation→ 'false'
    #[pyo3(get, set)]
    pub false_str: String,
}

#[pymethods]
impl FormatConfig {
    #[new]
    #[pyo3(signature = (
        field_sep = ';',
        kv_sep = '=',
        list_open = '[',
        list_close = ']',
        list_sep = '|',
        obj_open = '{',
        obj_close = '}',
        escape = '\\',
        null_str = "null".to_string(),
        true_str = "true".to_string(),
        false_str = "false".to_string(),
    ))]
    pub fn new(
        field_sep: char,
        kv_sep: char,
        list_open: char,
        list_close: char,
        list_sep: char,
        obj_open: char,
        obj_close: char,
        escape: char,
        null_str: String,
        true_str: String,
        false_str: String,
    ) -> Self {
        FormatConfig {
            field_sep,
            kv_sep,
            list_open,
            list_close,
            list_sep,
            obj_open,
            obj_close,
            escape,
            null_str,
            true_str,
            false_str,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "FormatConfig(field_sep={:?}, kv_sep={:?}, list_sep={:?})",
            self.field_sep, self.kv_sep, self.list_sep
        )
    }

    /// Returns the set of all characters that must be escaped in scalar values
    pub fn special_chars(&self) -> Vec<char> {
        vec![
            self.field_sep,
            self.kv_sep,
            self.list_open,
            self.list_close,
            self.list_sep,
            self.obj_open,
            self.obj_close,
            self.escape,
        ]
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig::new(
            ';',
            '=',
            '[',
            ']',
            '|',
            '{',
            '}',
            '\\',
            "null".into(),
            "true".into(),
            "false".into(),
        )
    }
}
