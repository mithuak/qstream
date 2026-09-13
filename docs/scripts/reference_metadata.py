"""Read public input and output types from qstream's PyO3 binding source."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

MACRO_METHODS = {
    "scalar_period_indicator": ("period: usize", "value: f64", "PyResult<Option<f64>>"),
    "ohlc_indicator": ("period: usize", "open: f64, high: f64, low: f64, close: f64", "PyResult<Option<f64>>"),
    "spectral_indicator": ("window: usize, update_every: usize, fs: f64, window_type: String", "value: f64", "PyResult<Option<SpectrumResult>>"),
    "cross_spectral_indicator": ("window: usize, update_every: usize, fs: f64, window_type: String", "x: f64, y: f64", "PyResult<Option<SpectrumResult>>"),
    "wavelet_indicator": ("window: usize, levels: usize, update_every: usize", "value: f64", "PyResult<Option<WaveletResult>>"),
    "cross_wavelet_indicator": ("window: usize, levels: usize, update_every: usize", "x: f64, y: f64", "PyResult<Option<SpectrumResult>>"),
    "make_spectral_heavy": ("window: usize, n_sources: usize, nfft: usize, update_every: usize", "value: f64", "PyResult<Option<SpectrumResult>>"),
}


def split_top_level(value: str) -> list[str]:
    """Split Rust/Python argument lists without splitting nested generic types."""
    chunks, start, depth = [], 0, 0
    for index, char in enumerate(value):
        if char in "<([":
            depth += 1
        elif char in ">)]":
            depth -= 1
        elif char == "," and depth == 0:
            chunks.append(value[start:index].strip())
            start = index + 1
    chunks.append(value[start:].strip())
    return [chunk for chunk in chunks if chunk]


def method(block: str, name: str) -> tuple[dict[str, str], str] | None:
    found = re.search(
        rf"\bfn\s+{re.escape(name)}\s*\((.*?)\)\s*(?:->\s*(.*?))?\s*\{{",
        block,
        re.DOTALL,
    )
    if not found:
        return None
    params = {}
    for arg in split_top_level(found.group(1)):
        if ":" not in arg:
            continue
        key, typ = arg.split(":", 1)
        params[key.strip()] = typ.strip()
    return params, (found.group(2) or "()").strip()


def class_methods(name: str, source: str) -> dict[str, tuple[dict[str, str], str]]:
    text = (ROOT / source).read_text()
    found = re.search(rf"(?ms)^impl\s+{re.escape(name)}\s*\{{(.*?)^\}}", text)
    if found:
        return {key: value for key in ("new", "update", "update_many", "compute", "price") if (value := method(found.group(1), key))}
    for macro, (constructor, inputs, output) in MACRO_METHODS.items():
        if re.search(rf"(?m)^{macro}!\(\s*{re.escape(name)}\s*,", text):
            return {
                "new": method(f"fn new({constructor}) -> PyResult<Self> {{", "new"),
                "update": method(f"fn update({inputs}) -> {output} {{", "update"),
            }
    return {}


def function_method(name: str, source: str) -> tuple[dict[str, str], str] | None:
    return method((ROOT / source).read_text(), name)


def result_fields(name: str, source: str) -> dict[str, str]:
    text = (ROOT / source).read_text()
    found = re.search(rf"(?ms)^pub struct\s+{re.escape(name)}\s*\{{(.*?)^\}}", text)
    if not found:
        return {}
    fields = {}
    for line in found.group(1).splitlines():
        field = re.match(r"\s*pub\s+(\w+):\s*(.*?)(?:,)?\s*$", line)
        if field:
            fields[field.group(1)] = field.group(2).removesuffix(",")
    return fields


def python_type(rust: str) -> str:
    rust = rust.strip()
    if rust.startswith("PyResult<") and rust.endswith(">"):
        return python_type(rust[9:-1])
    if rust.startswith("Option<") and rust.endswith(">"):
        return f"{python_type(rust[7:-1])} | None"
    if rust.startswith("Vec<") and rust.endswith(">"):
        return f"list[{python_type(rust[4:-1])}]"
    if rust.startswith("(") and rust.endswith(")"):
        parts = split_top_level(rust[1:-1])
        return "None" if not parts else "tuple[" + ", ".join(python_type(part) for part in parts) + "]"
    return {"f64": "float", "usize": "int", "bool": "bool", "String": "str", "&str": "str", "Self": "object"}.get(rust, rust)


def python_params(signature: str) -> list[tuple[str, str | None]]:
    inner = signature.strip()[1:-1]
    result = []
    for part in split_top_level(inner):
        if part in {"$self", "/", "*"}:
            continue
        name, _, default = part.partition("=")
        result.append((name.strip(), default.strip() or None))
    return result
