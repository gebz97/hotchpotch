use crate::config::FormatConfig;
use crate::value::Value;

/// Escape a string scalar using the precomputed O(1) special_table.
/// Unicode content is preserved — only ASCII delimiter bytes are escaped.
pub fn escape_str(s: &str, cfg: &FormatConfig) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        // Only ASCII chars can be delimiters — safe to index table
        if ch.is_ascii() && cfg.special_table[ch as usize] {
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
        Value::Bool(b) => {
            if *b {
                cfg.true_str.clone()
            } else {
                cfg.false_str.clone()
            }
        }
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        Value::Str(s) => escape_str(s, cfg),

        Value::List(items) => {
            // Inline construction avoids Vec<String> + join + separator allocation
            let mut out = String::new();
            out.push(cfg.list_open);
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(cfg.list_sep);
                }
                out.push_str(&serialize_value(item, cfg));
            }
            out.push(cfg.list_close);
            out
        }

        Value::Object(map) => {
            let mut out = String::new();
            out.push(cfg.obj_open);
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(cfg.field_sep);
                }
                out.push_str(&escape_str(k, cfg));
                out.push(cfg.kv_sep);
                out.push_str(&serialize_value(v, cfg));
            }
            out.push(cfg.obj_close);
            out
        }
    }
}

/// Serialize a top-level object into the full format: key1=val1;key2=val2;
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
