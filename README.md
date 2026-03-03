# hotchpotch

Fast Rust-powered serialization for embedding Python objects in CSV fields.

## Format
```
name=adam;hobbies=[cycling|rowing|chess];height=175cm;
```

Supports: strings, ints, floats, bools, None, lists, nested dicts. 
Special characters are backslash-escaped. All delimiters are configurable.

## Install
```bash
pip install hotchpotch
```

## Usage
```python
import hotchpotch

cfg = hotchpotch.FormatConfig()

s = hotchpotch.dumps({"name": "adam", "hobbies": ["cycling", "rowing"], "age": 30}, cfg)
# → "age=30;hobbies=[cycling|rowing];name=adam;"

data = hotchpotch.loads(s, cfg)
# → {"age": 30, "hobbies": ["cycling", "rowing"], "name": "adam"}
```

## Custom delimiters
```python
cfg = hotchpotch.FormatConfig(field_sep='&', kv_sep=':', list_sep=',')
```