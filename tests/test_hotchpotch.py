"""
Extensive test suite for hotchpotch.
Run with: pytest test_hotchpotch.py -v
"""
import pytest
import hotchpotch as hp

# ── Fixtures ──────────────────────────────────────────────────────────────────

@pytest.fixture
def cfg():
    return hp.FormatConfig()

def roundtrip(obj, cfg):
    """Serialize then deserialize, assert equality."""
    s = hp.dumps(obj, cfg)
    result = hp.loads(s, cfg)
    assert result == obj, f"\nInput:    {obj}\nEncoded:  {s}\nDecoded:  {result}"
    return s

# ── Scalar types ──────────────────────────────────────────────────────────────

class TestScalars:
    def test_string(self, cfg):
        roundtrip({"key": "hello"}, cfg)

    def test_empty_string(self, cfg):
        roundtrip({"key": ""}, cfg)

    def test_integer_positive(self, cfg):
        roundtrip({"n": 42}, cfg)

    def test_integer_negative(self, cfg):
        roundtrip({"n": -99}, cfg)

    def test_integer_zero(self, cfg):
        roundtrip({"n": 0}, cfg)

    def test_float(self, cfg):
        roundtrip({"f": 3.14}, cfg)

    def test_float_negative(self, cfg):
        roundtrip({"f": -0.001}, cfg)

    def test_float_whole(self, cfg):
        # whole floats should survive as float, not coerced to int
        s = hp.dumps({"f": 2.0}, cfg)
        result = hp.loads(s, cfg)
        assert result["f"] == 2.0
        assert isinstance(result["f"], float)

    def test_bool_true(self, cfg):
        roundtrip({"b": True}, cfg)

    def test_bool_false(self, cfg):
        roundtrip({"b": False}, cfg)

    def test_none(self, cfg):
        roundtrip({"n": None}, cfg)

    def test_multiple_scalars(self, cfg):
        roundtrip({"name": "adam", "age": 30, "score": 9.5, "active": True, "note": None}, cfg)


# ── Lists ─────────────────────────────────────────────────────────────────────

class TestLists:
    def test_string_list(self, cfg):
        roundtrip({"hobbies": ["cycling", "chess", "rowing"]}, cfg)

    def test_int_list(self, cfg):
        roundtrip({"nums": [1, 2, 3]}, cfg)

    def test_float_list(self, cfg):
        roundtrip({"vals": [1.1, 2.2, 3.3]}, cfg)

    def test_mixed_list(self, cfg):
        roundtrip({"mixed": ["hello", 42, 3.14, True, None]}, cfg)

    def test_empty_list(self, cfg):
        roundtrip({"empty": []}, cfg)

    def test_single_item_list(self, cfg):
        roundtrip({"one": ["only"]}, cfg)

    def test_nested_list_of_lists(self, cfg):
        roundtrip({"matrix": [[1, 2], [3, 4]]}, cfg)

    def test_large_list(self, cfg):
        roundtrip({"big": list(range(100))}, cfg)


# ── Nested objects ────────────────────────────────────────────────────────────

class TestNestedObjects:
    def test_shallow_nested(self, cfg):
        roundtrip({"address": {"city": "London", "zip": "EC1A"}}, cfg)

    def test_deeply_nested(self, cfg):
        roundtrip({"a": {"b": {"c": {"d": "deep"}}}}, cfg)

    def test_nested_with_list(self, cfg):
        roundtrip({
            "hobbies": {
                "sports": ["cycling", "swimming", "basketball"],
                "other": "chess"
            }
        }, cfg)

    def test_nested_with_all_types(self, cfg):
        roundtrip({
            "meta": {
                "count": 3,
                "ratio": 0.75,
                "enabled": True,
                "label": None,
                "tags": ["a", "b"],
            }
        }, cfg)

    def test_empty_nested_object(self, cfg):
        roundtrip({"empty": {}}, cfg)

    def test_full_complex_object(self, cfg):
        roundtrip({
            "name": "adam",
            "age": 30,
            "hobbies": {
                "sports": ["cycling", "swimming", "basketball"],
                "other": "chess"
            },
            "address": {"city": "London", "zip": "EC1A"},
            "active": True,
            "score": 9.5,
            "nickname": None,
        }, cfg)


# ── Escape handling ───────────────────────────────────────────────────────────

class TestEscaping:
    def test_equals_in_value(self, cfg):
        roundtrip({"expr": "a=b"}, cfg)

    def test_semicolon_in_value(self, cfg):
        roundtrip({"note": "end;start"}, cfg)

    def test_pipe_in_value(self, cfg):
        roundtrip({"note": "a|b"}, cfg)

    def test_brackets_in_value(self, cfg):
        roundtrip({"note": "[bracketed]"}, cfg)

    def test_braces_in_value(self, cfg):
        roundtrip({"note": "{braced}"}, cfg)

    def test_backslash_in_value(self, cfg):
        roundtrip({"path": "C:\\Users\\adam"}, cfg)

    def test_multiple_specials(self, cfg):
        roundtrip({"expr": "price=10;qty=5|total=50"}, cfg)

    def test_special_in_key(self, cfg):
        # keys with special chars should also be escaped
        roundtrip({"key=with=equals": "value"}, cfg)

    def test_all_specials_combined(self, cfg):
        roundtrip({"x": "a=b;c[d]e{f}g|h\\i"}, cfg)


# ── Custom config ─────────────────────────────────────────────────────────────

class TestCustomConfig:
    def test_custom_field_sep(self):
        cfg = hp.FormatConfig(field_sep='&')
        roundtrip({"a": 1, "b": 2}, cfg)

    def test_custom_kv_sep(self):
        cfg = hp.FormatConfig(kv_sep=':')
        roundtrip({"key": "val"}, cfg)

    def test_custom_list_sep(self):
        cfg = hp.FormatConfig(list_sep=',')
        roundtrip({"items": ["x", "y", "z"]}, cfg)

    def test_fully_custom(self):
        cfg = hp.FormatConfig(
            field_sep='&', kv_sep=':', list_open='(', list_close=')',
            list_sep=',', obj_open='<', obj_close='>'
        )
        roundtrip({
            "name": "adam",
            "tags": ["a", "b"],
            "meta": {"x": 1}
        }, cfg)

    def test_custom_null(self):
        cfg = hp.FormatConfig(null_str="nil")
        s = hp.dumps({"x": None}, cfg)
        assert "nil" in s
        assert hp.loads(s, cfg) == {"x": None}

    def test_custom_bool(self):
        cfg = hp.FormatConfig(true_str="yes", false_str="no")
        s = hp.dumps({"a": True, "b": False}, cfg)
        assert "yes" in s and "no" in s
        assert hp.loads(s, cfg) == {"a": True, "b": False}


# ── Type coercion on loads ────────────────────────────────────────────────────

class TestTypeCoercion:
    def test_int_coercion(self, cfg):
        result = hp.loads("n=42;", cfg)
        assert result["n"] == 42
        assert isinstance(result["n"], int)

    def test_float_coercion(self, cfg):
        result = hp.loads("f=3.14;", cfg)
        assert result["f"] == 3.14
        assert isinstance(result["f"], float)

    def test_bool_true_coercion(self, cfg):
        result = hp.loads("b=true;", cfg)
        assert result["b"] is True

    def test_bool_false_coercion(self, cfg):
        result = hp.loads("b=false;", cfg)
        assert result["b"] is False

    def test_null_coercion(self, cfg):
        result = hp.loads("n=null;", cfg)
        assert result["n"] is None

    def test_string_not_coerced(self, cfg):
        result = hp.loads("s=hello;", cfg)
        assert result["s"] == "hello"
        assert isinstance(result["s"], str)

    def test_numeric_string_preserved(self, cfg):
        # "175cm" should stay as string, not fail
        result = hp.loads("height=175cm;", cfg)
        assert result["height"] == "175cm"
        assert isinstance(result["height"], str)


# ── Error handling ────────────────────────────────────────────────────────────

class TestErrors:
    def test_loads_wrong_type(self, cfg):
        with pytest.raises(TypeError):
            hp.loads({"not": "a string"}, cfg)

    def test_dumps_empty_dict(self, cfg):
        s = hp.dumps({}, cfg)
        assert hp.loads(s, cfg) == {}

    def test_loads_empty_string(self, cfg):
        result = hp.loads("", cfg)
        assert result == {}

    def test_loads_trailing_sep_only(self, cfg):
        result = hp.loads(";;;", cfg)
        assert result == {}


# ── CSV embedding ─────────────────────────────────────────────────────────────

class TestCSVEmbedding:
    def test_no_comma_in_output(self, cfg):
        """Default format should never produce bare commas — safe for CSV."""
        s = hp.dumps({"name": "adam", "tags": ["a", "b"], "age": 30}, cfg)
        assert "," not in s

    def test_single_line_output(self, cfg):
        """Output must be a single line for CSV embedding."""
        s = hp.dumps({"name": "adam", "bio": "line one"}, cfg)
        assert "\n" not in s
        assert "\r" not in s

    def test_roundtrip_via_csv_string(self, cfg):
        import io, csv
        data = {"env": "prod", "tags": ["web", "nginx"], "tier": 1}
        row = {"vm": "web-01", "meta": hp.dumps(data, cfg)}

        # write to CSV string
        buf = io.StringIO()
        writer = csv.DictWriter(buf, fieldnames=["vm", "meta"])
        writer.writeheader()
        writer.writerow(row)

        # read back
        buf.seek(0)
        reader = csv.DictReader(buf)
        for parsed_row in reader:
            result = hp.loads(parsed_row["meta"], cfg)
            assert result == data


# ── VMware notes simulation ───────────────────────────────────────────────────

class TestVMwareNotesSimulation:
    def test_typical_vm_metadata(self, cfg):
        roundtrip({
            "env": "prod",
            "owner": "platform-team",
            "tier": 1,
            "tags": ["web", "nginx", "lb"],
            "monitoring": {"enabled": True, "interval": 60},
            "cost_center": "CC-1042",
            "last_patched": "2026-03-01",
        }, cfg)

    def test_update_single_field(self, cfg):
        meta = {"env": "staging", "owner": "devs", "tier": 2}
        s = hp.dumps(meta, cfg)
        restored = hp.loads(s, cfg)
        restored["env"] = "prod"
        s2 = hp.dumps(restored, cfg)
        final = hp.loads(s2, cfg)
        assert final["env"] == "prod"
        assert final["owner"] == "devs"

    def test_notes_field_size(self, cfg):
        """vCenter notes field limit is 32KB — ensure we stay well under."""
        big_meta = {f"key_{i}": f"value_{i}" for i in range(200)}
        s = hp.dumps(big_meta, cfg)
        assert len(s.encode("utf-8")) < 32 * 1024