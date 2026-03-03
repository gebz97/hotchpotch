use std::collections::BTreeMap;
use crate::config::FormatConfig;
use crate::value::Value;

pub struct Parser<'a> {
    input: &'a [char],
    pos: usize,
    cfg: &'a FormatConfig,
}

impl<'a> Parser<'a> {
    pub fn new(chars: &'a [char], cfg: &'a FormatConfig) -> Self {
        Parser { input: chars, pos: 0, cfg }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        match self.advance() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(format!("Expected {:?} but got {:?} at pos {}", ch, c, self.pos - 1)),
            None => Err(format!("Expected {:?} but got EOF", ch)),
        }
    }

    /// Read a scalar string, handling escape sequences, stopping at any
    /// unescaped special character in `stop_at`.
    fn read_scalar(&mut self, stop_at: &[char]) -> String {
        let mut out = String::new();
        loop {
            match self.peek() {
                None => break,
                Some(ch) if ch == self.cfg.escape => {
                    self.advance(); // consume escape char
                    if let Some(next) = self.advance() {
                        out.push(next); // take the literal next char
                    }
                }
                Some(ch) if stop_at.contains(&ch) => break,
                Some(ch) => { self.advance(); out.push(ch); }
            }
        }
        out
    }

    /// Parse a value: list, nested object, or scalar.
    pub fn parse_value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some(ch) if ch == self.cfg.list_open => self.parse_list(),
            Some(ch) if ch == self.cfg.obj_open  => self.parse_nested_object(),
            _ => self.parse_scalar(),
        }
    }

    fn parse_list(&mut self) -> Result<Value, String> {
        self.expect(self.cfg.list_open)?;
        let mut items = Vec::new();

        // Handle empty list []
        if self.peek() == Some(self.cfg.list_close) {
            self.advance();
            return Ok(Value::List(items));
        }

        loop {
            items.push(self.parse_value()?);
            match self.peek() {
                Some(ch) if ch == self.cfg.list_sep   => { self.advance(); }
                Some(ch) if ch == self.cfg.list_close => { self.advance(); break; }
                Some(ch) => return Err(format!("Unexpected {:?} in list at pos {}", ch, self.pos)),
                None => return Err("Unterminated list".into()),
            }
        }
        Ok(Value::List(items))
    }

    fn parse_nested_object(&mut self) -> Result<Value, String> {
        self.expect(self.cfg.obj_open)?;
        let mut map = BTreeMap::new();

        if self.peek() == Some(self.cfg.obj_close) {
            self.advance();
            return Ok(Value::Object(map));
        }

        loop {
            // Stop chars for key: kv_sep
            let key = self.read_scalar(&[self.cfg.kv_sep, self.cfg.obj_close]);
            self.expect(self.cfg.kv_sep)?;

            // Stop chars for value inside nested obj: field_sep or obj_close
            let val = self.parse_value()?;
            map.insert(key, val);

            match self.peek() {
                Some(ch) if ch == self.cfg.field_sep => { self.advance(); }
                Some(ch) if ch == self.cfg.obj_close => { self.advance(); break; }
                Some(ch) => return Err(format!("Unexpected {:?} in object at pos {}", ch, self.pos)),
                None => return Err("Unterminated object".into()),
            }
        }
        Ok(Value::Object(map))
    }

    fn parse_scalar(&mut self) -> Result<Value, String> {
        // Stop at anything that could end a scalar in any context
        let stop = vec![
            self.cfg.field_sep, self.cfg.kv_sep,
            self.cfg.list_sep, self.cfg.list_close,
            self.cfg.obj_close,
        ];
        let s = self.read_scalar(&stop);
        Ok(coerce_scalar(s, self.cfg))
    }

    /// Parse a full top-level record: key=val;key=val;
    pub fn parse_record(&mut self) -> Result<Vec<(String, Value)>, String> {
        let mut fields = Vec::new();

        while self.pos < self.input.len() {
            // Skip trailing field_sep at end of input
            if self.peek() == Some(self.cfg.field_sep) {
                self.advance();
                continue;
            }

            let key = self.read_scalar(&[self.cfg.kv_sep]);
            if key.is_empty() { break; }

            self.expect(self.cfg.kv_sep)?;
            let val = self.parse_value()?;
            fields.push((key, val));

            // consume trailing field_sep after value
            if self.peek() == Some(self.cfg.field_sep) {
                self.advance();
            }
        }
        Ok(fields)
    }
}

/// Infer the type of a scalar string (int → float → bool → null → str)
fn coerce_scalar(s: String, cfg: &FormatConfig) -> Value {
    if s == cfg.null_str  { return Value::Null; }
    if s == cfg.true_str  { return Value::Bool(true); }
    if s == cfg.false_str { return Value::Bool(false); }
    if let Ok(i) = s.parse::<i64>()  { return Value::Int(i); }
    if let Ok(f) = s.parse::<f64>()  { return Value::Float(f); }
    Value::Str(s)
}

pub fn parse_record_str(s: &str, cfg: &FormatConfig) -> Result<Vec<(String, Value)>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut parser = Parser::new(&chars, cfg);
    parser.parse_record()
}