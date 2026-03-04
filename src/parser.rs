use crate::config::FormatConfig;
use crate::value::Value;
use std::collections::BTreeMap;

pub struct Parser<'a> {
    input: &'a [u8], // raw UTF-8 bytes — no Vec<char> allocation
    pos: usize,
    cfg: &'a FormatConfig,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], cfg: &'a FormatConfig) -> Self {
        Parser { input, pos: 0, cfg }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    #[inline]
    fn advance(&mut self) -> Option<u8> {
        let b = self.input.get(self.pos).copied();
        self.pos += 1;
        b
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        let expected = ch as u8;
        match self.advance() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(format!(
                "Expected {:?} but got {:?} at pos {}",
                ch,
                b as char,
                self.pos - 1
            )),
            None => Err(format!("Expected {:?} but got EOF", ch)),
        }
    }

    /// Read bytes into a String, handling escape sequences.
    /// Stops at any unescaped byte in `stop_at`.
    /// Unicode multibyte sequences are copied as-is since their bytes
    /// are all > 127 and can never match ASCII delimiter bytes.
    fn read_scalar(&mut self, stop_at: &[u8]) -> String {
        let mut out = String::new();
        loop {
            match self.peek() {
                None => break,
                Some(b) if b == self.cfg.escape as u8 => {
                    self.advance(); // consume escape byte
                    // Advance past the escaped character — may be multibyte
                    if self.pos < self.input.len() {
                        let start = self.pos;
                        self.advance(); // consume first byte
                        // If it's a multibyte UTF-8 sequence, consume continuation bytes
                        while self.pos < self.input.len() && (self.input[self.pos] & 0xC0) == 0x80 {
                            self.advance();
                        }
                        // SAFETY: input is valid UTF-8 (came from Python str)
                        out.push_str(
                            std::str::from_utf8(&self.input[start..self.pos]).unwrap_or(""),
                        );
                    }
                }
                Some(b) if stop_at.contains(&b) => break,
                Some(b) => {
                    self.advance();
                    if b < 128 {
                        // Fast path: ASCII
                        out.push(b as char);
                    } else {
                        // Multibyte UTF-8: consume all continuation bytes together
                        let start = self.pos - 1;
                        while self.pos < self.input.len() && (self.input[self.pos] & 0xC0) == 0x80 {
                            self.advance();
                        }
                        out.push_str(
                            std::str::from_utf8(&self.input[start..self.pos]).unwrap_or(""),
                        );
                    }
                }
            }
        }
        out
    }

    pub fn parse_value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some(b) if b == self.cfg.list_open as u8 => self.parse_list(),
            Some(b) if b == self.cfg.obj_open as u8 => self.parse_nested_object(),
            _ => self.parse_scalar(),
        }
    }

    fn parse_list(&mut self) -> Result<Value, String> {
        self.expect(self.cfg.list_open)?;
        let mut items = Vec::new();

        if self.peek() == Some(self.cfg.list_close as u8) {
            self.advance();
            return Ok(Value::List(items));
        }

        let stop = [self.cfg.list_sep as u8, self.cfg.list_close as u8];
        loop {
            items.push(self.parse_value()?);
            match self.peek() {
                Some(b) if b == self.cfg.list_sep as u8 => {
                    self.advance();
                }
                Some(b) if b == self.cfg.list_close as u8 => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    return Err(format!(
                        "Unexpected {:?} in list at pos {}",
                        b as char, self.pos
                    ));
                }
                None => return Err("Unterminated list".into()),
            }
            let _ = stop; // suppress unused warning
        }
        Ok(Value::List(items))
    }

    fn parse_nested_object(&mut self) -> Result<Value, String> {
        self.expect(self.cfg.obj_open)?;
        let mut map = BTreeMap::new();

        if self.peek() == Some(self.cfg.obj_close as u8) {
            self.advance();
            return Ok(Value::Object(map));
        }

        loop {
            let key = self.read_scalar(&[self.cfg.kv_sep as u8, self.cfg.obj_close as u8]);
            self.expect(self.cfg.kv_sep)?;
            let val = self.parse_value()?;
            map.insert(key, val);

            match self.peek() {
                Some(b) if b == self.cfg.field_sep as u8 => {
                    self.advance();
                }
                Some(b) if b == self.cfg.obj_close as u8 => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    return Err(format!(
                        "Unexpected {:?} in object at pos {}",
                        b as char, self.pos
                    ));
                }
                None => return Err("Unterminated object".into()),
            }
        }
        Ok(Value::Object(map))
    }

    fn parse_scalar(&mut self) -> Result<Value, String> {
        // Uses precomputed stack-allocated stop array from config — zero heap
        let s = self.read_scalar(&self.cfg.scalar_stop);
        Ok(coerce_scalar(s, self.cfg))
    }

    pub fn parse_record(&mut self) -> Result<Vec<(String, Value)>, String> {
        let mut fields = Vec::new();

        while self.pos < self.input.len() {
            if self.peek() == Some(self.cfg.field_sep as u8) {
                self.advance();
                continue;
            }

            let key = self.read_scalar(&[self.cfg.kv_sep as u8]);
            if key.is_empty() {
                break;
            }

            self.expect(self.cfg.kv_sep)?;
            let val = self.parse_value()?;
            fields.push((key, val));

            if self.peek() == Some(self.cfg.field_sep as u8) {
                self.advance();
            }
        }
        Ok(fields)
    }
}

fn coerce_scalar(s: String, cfg: &FormatConfig) -> Value {
    if s == cfg.null_str {
        return Value::Null;
    }
    if s == cfg.true_str {
        return Value::Bool(true);
    }
    if s == cfg.false_str {
        return Value::Bool(false);
    }
    if let Ok(i) = s.parse::<i64>() {
        return Value::Int(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    Value::Str(s)
}

pub fn parse_record_str(s: &str, cfg: &FormatConfig) -> Result<Vec<(String, Value)>, String> {
    // Pass raw UTF-8 bytes directly — no Vec<char> allocation
    let mut parser = Parser::new(s.as_bytes(), cfg);
    parser.parse_record()
}
