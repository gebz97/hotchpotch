use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

/// All delimiters that define the serialization format.
/// Defaults match the example: name=adam;hobbies=[cycling|rowing|chess];height=175cm;
#[pyclass]
#[derive(Clone, Debug)]
pub struct FormatConfig {
    #[pyo3(get, set)]
    pub field_sep: char,
    #[pyo3(get, set)]
    pub kv_sep: char,
    #[pyo3(get, set)]
    pub list_open: char,
    #[pyo3(get, set)]
    pub list_close: char,
    #[pyo3(get, set)]
    pub list_sep: char,
    #[pyo3(get, set)]
    pub obj_open: char,
    #[pyo3(get, set)]
    pub obj_close: char,
    #[pyo3(get, set)]
    pub escape: char,
    #[pyo3(get, set)]
    pub null_str: String,
    #[pyo3(get, set)]
    pub true_str: String,
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
    ) -> PyResult<Self> {
        let delimiters = [
            ("field_sep",  field_sep),
            ("kv_sep",     kv_sep),
            ("list_open",  list_open),
            ("list_close", list_close),
            ("list_sep",   list_sep),
            ("obj_open",   obj_open),
            ("obj_close",  obj_close),
            ("escape",     escape),
        ];

        // Check every pair for collisions
        for i in 0..delimiters.len() {
            for j in (i + 1)..delimiters.len() {
                let (name_a, ch_a) = delimiters[i];
                let (name_b, ch_b) = delimiters[j];
                if ch_a == ch_b {
                    return Err(PyValueError::new_err(format!(
                        "Delimiter conflict: '{}' and '{}' are both {:?}",
                        name_a, name_b, ch_a
                    )));
                }
            }
        }

        // null/true/false strings must not contain any delimiter character
        let sentinel_chars: Vec<char> = delimiters.iter().map(|(_, c)| *c).collect();
        for (label, s) in [
            ("null_str",  &null_str),
            ("true_str",  &true_str),
            ("false_str", &false_str),
        ] {
            for ch in s.chars() {
                if sentinel_chars.contains(&ch) {
                    return Err(PyValueError::new_err(format!(
                        "'{}' contains delimiter character {:?}",
                        label, ch
                    )));
                }
            }
        }

        // null/true/false strings must all be distinct from each other
        if null_str == true_str {
            return Err(PyValueError::new_err(
                "null_str and true_str must be different".to_string()
            ));
        }
        if null_str == false_str {
            return Err(PyValueError::new_err(
                "null_str and false_str must be different".to_string()
            ));
        }
        if true_str == false_str {
            return Err(PyValueError::new_err(
                "true_str and false_str must be different".to_string()
            ));
        }

        // null/true/false strings must not be empty
        if null_str.is_empty() {
            return Err(PyValueError::new_err("null_str must not be empty"));
        }
        if true_str.is_empty() {
            return Err(PyValueError::new_err("true_str must not be empty"));
        }
        if false_str.is_empty() {
            return Err(PyValueError::new_err("false_str must not be empty"));
        }

        Ok(FormatConfig {
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
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "FormatConfig(field_sep={:?}, kv_sep={:?}, list_open={:?}, list_close={:?}, \
             list_sep={:?}, obj_open={:?}, obj_close={:?}, escape={:?}, \
             null_str={:?}, true_str={:?}, false_str={:?})",
            self.field_sep, self.kv_sep, self.list_open, self.list_close,
            self.list_sep, self.obj_open, self.obj_close, self.escape,
            self.null_str, self.true_str, self.false_str,
        )
    }

    /// Returns the set of all characters that must be escaped in scalar values.
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
            ';', '=', '[', ']', '|', '{', '}', '\\',
            "null".into(),
            "true".into(),
            "false".into(),
        )
        .expect("default FormatConfig is always valid")
    }
}