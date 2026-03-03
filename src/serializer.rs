use crate::config::FormatConfig;
use crate::value::Value;

/// Escape a string scalar: prepend `config.escape` before any special char.
pub fn escape_str(s: &str, cfg: &FormatConfig) -> String {
    let specials = cfg.special_chars();
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if specials.contains(&ch) {
            out.push(cfg.escape);
        }
        out.push(ch);
    }
    out
}

/// Serialize a [`Value`] into a string using `cfg`.
pub fn serialize_value(val: &Value, cfg: &FormatConfig) -> String {
    match val {
        Value::Null => cfg.null_str.clone(),
        Value::Bool(b) => if *b { cfg.true_str.clone() } else { cfg.false_str.clone() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            // Always include decimal point so floats are unambiguous from ints
            if f.fract() == 0.0 { format!("{:.1}", f) } else { f.to_string() }
        }
        Value::Str(s) => escape_str(s, cfg),

        Value::List(items) => {
            let inner: Vec<String> = items.iter().map(|v| serialize_value(v, cfg)).collect();
            format!(
                "{}{}{}",
                cfg.list_open,
                inner.join(&cfg.list_sep.to_string()),
                cfg.list_close,
            )
        }

        Value::Object(map) => {
            let inner: Vec<String> = map
                .iter()
                .map(|(k, v)| format!(
                    "{}{}{}",
                    escape_str(k, cfg),
                    cfg.kv_sep,
                    serialize_value(v, cfg)
                ))
                .collect();
            format!(
                "{}{}{}",
                cfg.obj_open,
                inner.join(&cfg.field_sep.to_string()),
                cfg.obj_close,
            )
        }
    }
}

/// Serialize a top-level object (dict of key→value) into the full format.
/// Produces:  key1=val1;key2=val2;
pub fn serialize_object(map: &[(String, Value)], cfg: &FormatConfig) -> String {
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(&escape_str(k, cfg));
        out.push(cfg.kv_sep);
        out.push_str(&serialize_value(v, cfg));
        out.push(cfg.field_sep);
    }
    out
}