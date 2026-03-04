use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

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

    // Precomputed O(1) lookup table for escaping — not exposed to Python
    // Index is ASCII byte value, true = must be escaped
    pub special_table: [bool; 256],

    // Precomputed stop set for scalar parsing — stack allocated, no heap
    pub scalar_stop: [u8; 5],
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
            ("field_sep", field_sep),
            ("kv_sep", kv_sep),
            ("list_open", list_open),
            ("list_close", list_close),
            ("list_sep", list_sep),
            ("obj_open", obj_open),
            ("obj_close", obj_close),
            ("escape", escape),
        ];

        // All delimiters must be ASCII — byte parser requires this
        for (name, ch) in &delimiters {
            if !ch.is_ascii() {
                return Err(PyValueError::new_err(format!(
                    "'{}' must be an ASCII character, got {:?}",
                    name, ch
                )));
            }
        }

        // Collision check via HashSet — cleaner than O(n²) loop
        let mut seen = std::collections::HashMap::new();
        for (name, ch) in &delimiters {
            if let Some(prev) = seen.insert(ch, name) {
                return Err(PyValueError::new_err(format!(
                    "Delimiter conflict: '{}' and '{}' are both {:?}",
                    prev, name, ch
                )));
            }
        }

        // Sentinel strings must not be empty
        for (label, s) in [
            ("null_str", &null_str),
            ("true_str", &true_str),
            ("false_str", &false_str),
        ] {
            if s.is_empty() {
                return Err(PyValueError::new_err(format!(
                    "'{}' must not be empty",
                    label
                )));
            }
        }

        // Sentinel strings must be distinct from each other
        if null_str == true_str {
            return Err(PyValueError::new_err(
                "null_str and true_str must be different",
            ));
        }
        if null_str == false_str {
            return Err(PyValueError::new_err(
                "null_str and false_str must be different",
            ));
        }
        if true_str == false_str {
            return Err(PyValueError::new_err(
                "true_str and false_str must be different",
            ));
        }

        // Sentinel strings must not contain delimiter characters
        let sentinel_chars: Vec<char> = delimiters.iter().map(|(_, c)| *c).collect();
        for (label, s) in [
            ("null_str", &null_str),
            ("true_str", &true_str),
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

        // Build O(1) special_table lookup
        let mut special_table = [false; 256];
        for (_, ch) in &delimiters {
            special_table[*ch as usize] = true;
        }

        // Precompute scalar stop bytes — used on every scalar parse, stack only
        let scalar_stop = [
            field_sep as u8,
            kv_sep as u8,
            list_sep as u8,
            list_close as u8,
            obj_close as u8,
        ];

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
            special_table,
            scalar_stop,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "FormatConfig(field_sep={:?}, kv_sep={:?}, list_open={:?}, list_close={:?}, \
             list_sep={:?}, obj_open={:?}, obj_close={:?}, escape={:?}, \
             null_str={:?}, true_str={:?}, false_str={:?})",
            self.field_sep,
            self.kv_sep,
            self.list_open,
            self.list_close,
            self.list_sep,
            self.obj_open,
            self.obj_close,
            self.escape,
            self.null_str,
            self.true_str,
            self.false_str,
        )
    }

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
        .expect("default FormatConfig is always valid")
    }
}
