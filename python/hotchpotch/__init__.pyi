from typing import Any

class FormatConfig:
    field_sep: str
    kv_sep: str
    list_open: str
    list_close: str
    list_sep: str
    obj_open: str
    obj_close: str
    escape: str
    null_str: str
    true_str: str
    false_str: str

    def __init__(
        self,
        field_sep: str = ";",
        kv_sep: str = "=",
        list_open: str = "[",
        list_close: str = "]",
        list_sep: str = "|",
        obj_open: str = "{",
        obj_close: str = "}",
        escape: str = "\\",
        null_str: str = "null",
        true_str: str = "true",
        false_str: str = "false",
    ) -> None: ...
    def __repr__(self) -> str: ...

def dumps(obj: dict[str, Any], config: FormatConfig | None = None) -> str: ...
def loads(s: str, config: FormatConfig | None = None) -> dict[str, Any]: ...