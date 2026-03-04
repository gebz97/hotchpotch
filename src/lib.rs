mod config;
mod parser;
mod serializer;
mod value;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};

use config::FormatConfig;
use value::Value;

// ── Python → internal Value ───────────────────────────────────────────────────

fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    // Order matters: Bool must come before Int (bool is subclass of int in Python)
    if obj.is_none() {
        Ok(Value::Null)
    } else if obj.is_instance_of::<PyBool>() {
        Ok(Value::Bool(obj.extract::<bool>()?))
    } else if obj.is_instance_of::<PyInt>() {
        Ok(Value::Int(obj.extract::<i64>()?))
    } else if obj.is_instance_of::<PyFloat>() {
        Ok(Value::Float(obj.extract::<f64>()?))
    } else if obj.is_instance_of::<PyString>() {
        Ok(Value::Str(obj.extract::<String>()?))
    } else if obj.is_instance_of::<PyList>() {
        // downcast is correct here — we already confirmed the type above
        let list = obj.cast::<PyList>()?;
        let items: PyResult<Vec<Value>> = list.iter().map(|x| py_to_value(&x)).collect();
        Ok(Value::List(items?))
    } else if obj.is_instance_of::<PyDict>() {
        let dict = obj.cast::<PyDict>()?;
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, py_to_value(&v)?);
        }
        Ok(Value::Object(map))
    } else {
        // Fallback: convert to string via __str__
        Ok(Value::Str(obj.str()?.extract::<String>()?))
    }
}

// ── internal Value → Python ───────────────────────────────────────────────────

fn value_to_py<'py>(val: &Value, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    match val {
        Value::Null => Ok(py.None().into_bound(py)),

        // .to_owned() converts Borrowed<'_, '_, T> → Bound<'py, T> before into_any()
        Value::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
        Value::Int(i) => Ok(i.into_pyobject(py)?.to_owned().into_any()),
        Value::Float(f) => Ok(f.into_pyobject(py)?.to_owned().into_any()),

        // String already returns an owned Bound, no .to_owned() needed
        Value::Str(s) => Ok(s.clone().into_pyobject(py)?.into_any()),

        Value::List(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(value_to_py(item, py)?)?;
            }
            Ok(list.into_any())
        }

        Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, value_to_py(v, py)?)?;
            }
            Ok(dict.into_any())
        }
    }
}

// ── Exposed Python functions ──────────────────────────────────────────────────

/// Serialize a Python dict into the custom format string.
///
/// Example:
///     cfg = FormatConfig()
///     dumps({"name": "adam", "hobbies": ["cycling", "rowing"], "age": 30}, cfg)
///     # → "age=30;hobbies=[cycling|rowing];name=adam;"
#[pyfunction]
#[pyo3(signature = (obj, config=None))]
fn dumps(obj: &Bound<'_, PyDict>, config: Option<&FormatConfig>) -> PyResult<String> {
    let cfg = config.cloned().unwrap_or_default();
    let mut fields: Vec<(String, Value)> = Vec::new();

    for (k, v) in obj.iter() {
        let key: String = k.extract()?;
        let val = py_to_value(&v)?;
        fields.push((key, val));
    }

    Ok(serializer::serialize_object(&fields, &cfg))
}

/// Deserialize a custom format string into a Python dict.
///
/// Example:
///     cfg = FormatConfig()
///     loads("name=adam;hobbies=[cycling|rowing];age=30;", cfg)
///     # → {"name": "adam", "hobbies": ["cycling", "rowing"], "age": 30}
#[pyfunction]
#[pyo3(signature = (s, config=None))]
fn loads(py: Python<'_>, s: &str, config: Option<&FormatConfig>) -> PyResult<Py<PyAny>> {
    let cfg = config.cloned().unwrap_or_default();

    let fields = parser::parse_record_str(s, &cfg).map_err(|e| PyValueError::new_err(e))?;

    let dict = PyDict::new(py);
    for (k, v) in &fields {
        dict.set_item(k, value_to_py(v, py)?)?;
    }
    Ok(dict.into())
}

// ── Module ────────────────────────────────────────────────────────────────────

#[pymodule]
fn hotchpotch(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FormatConfig>()?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    Ok(())
}
