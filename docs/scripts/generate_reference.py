"""Regenerate VitePress API and workbook pages from the current qstream build.

Run from the repository root after installing the local extension:
    .venv/bin/python docs/scripts/generate_reference.py
"""

from __future__ import annotations

import inspect
import re
import shutil
import sys
import textwrap
import xml.etree.ElementTree as ET
import zipfile
from collections import defaultdict
from pathlib import Path

from reference_content import (
    EXAMPLES,
    FIELD_MEANINGS,
    OUTPUT_MEANINGS,
    computation,
    input_meaning,
    parameter_meaning,
    why_use,
)
from reference_metadata import class_methods, function_method, python_params, python_type, result_fields

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
API = DOCS / "api"
ALIASES = {
    "ExpectedShortfall", "CVaR", "ES", "WelchPSD", "BetaAlpha",
    "FamaFrenchThree", "FamaFrenchFive", "Carhart", "UpDownCapture",
    "DWT", "MODWT", "CWT", "RLS", "LMS",
}
PREFIX_SOURCE = {
    "fin": "src/python/finance.rs",
    "fin_exp": "src/python/finance_experimental.rs",
    "sig": "src/python/signal.rs",
    "sm": "src/python/spectral_missing.rs",
    "exp": "src/python/experimental.rs",
    "rt": "src/python/result_types.rs",
    "FeatureEngine": "src/python/engine.rs",
}


def registration_map() -> dict[str, tuple[str, str]]:
    """Return exported name -> (group, source), using lib.rs registration order."""
    lines = (ROOT / "src/lib.rs").read_text().splitlines()
    result: dict[str, tuple[str, str]] = {}
    group = "Result types"
    section = "Core"
    for line in lines:
        note = re.match(r"\s*//\s+([^/=].*)", line)
        if note:
            label = note.group(1).strip(" .")
            if label.startswith("Phase 7"):
                section = "Experimental"
            elif label.startswith(("Finance:", "Signal:")):
                group = label.replace(":", " ·", 1)
            elif label in {"Result types", "Grouped execution", "Decomposition", "Higher-order spectra", "Time-frequency", "Wavelet extras", "Heavy Kalman", "Cycle", "FastICA", "Spectral heavy methods"}:
                group = label
            elif label.startswith("Remaining spectral"):
                group = "Signal · Spectral and filtering"
        found = re.search(r"m\.add_class::<(?:(\w+)::)?(\w+)>\(\)", line)
        if found:
            prefix, rust_name = found.groups()
            source = PREFIX_SOURCE.get(prefix or rust_name, "src/lib.rs")
            result[rust_name] = (f"{section} · {group}" if section == "Experimental" else group, source)
        found = re.search(r"wrap_pyfunction!\((\w+)::(\w+), m\)", line)
        if found:
            prefix, name = found.groups()
            source = PREFIX_SOURCE.get(prefix, "src/lib.rs")
            result[name] = (f"{section} · {group}" if section == "Experimental" else group, source)
    return result


def first_sentence(value: str | None) -> str:
    if not value:
        return "See the public signature and source for behavior."
    paragraph = value.strip().split("\n\n", 1)[0]
    return " ".join(paragraph.split())


def signature(obj: object) -> str:
    raw = getattr(obj, "__text_signature__", None) or "()"
    return raw.replace("($self, ", "(").replace("($self)", "()")


def table_cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def input_table(params: list[tuple[str, str | None]], types: dict[str, str], group: str, constructor: bool, indicator: str) -> list[str]:
    lines = ["| Parameter | Type | Default | Meaning |" if constructor else "| Input | Type | Meaning |"]
    lines += ["|---|---|---|---|" if constructor else "|---|---|---|"]
    for name, default in params:
        typ = python_type(types.get(name, "object"))
        meaning = parameter_meaning(indicator, name) if constructor else input_meaning(name, group)
        if constructor:
            lines.append(f"| `{name}` | `{table_cell(typ)}` | `{table_cell(default or 'required')}` | {table_cell(meaning)} |")
        else:
            lines.append(f"| `{name}` | `{table_cell(typ)}` | {table_cell(meaning)} |")
    return lines + [""]


def render_api_page(name: str, obj: object, group: str, source: str) -> str:
    is_class = inspect.isclass(obj)
    is_result = group.endswith("Result types")
    doc = getattr(obj, "__doc__", "") or ""
    methods = class_methods(name, source) if is_class and not is_result else {}
    update = methods.get("update") if is_class else function_method(name, source)
    summary = first_sentence(getattr(obj, "__doc__", None))
    if summary == "See the public signature and source for behavior." and is_result:
        summary = "Structured output returned by a qstream calculation."
    lines = [
        f"# {name}", "",
        f"> {summary}", "",
        "| Field | Value |", "|---|---|",
        f"| Group | {group} |",
        f"| Source | `{source}` |",
        f"| Python import | `from qstream import {name}` |", "",
    ]
    if is_result:
        lines += ["Result objects are returned by feature updates; they are not constructed directly.", ""]
        fields = result_fields(name, source)
        if fields:
            lines += ["## Output fields", "", "| Field | Type | Meaning |", "|---|---|---|"]
            for field, rust_type in fields.items():
                lines.append(f"| `{field}` | `{table_cell(python_type(rust_type))}` | {table_cell(FIELD_MEANINGS.get(field, 'Result value; see the producing indicator for interpretation.'))} |")
            lines.append("")
    else:
        lines += ["## Signature", "", "```python", f"{name}{signature(obj)}", "```", ""]
        lines += ["## What it computes", "", computation(doc), "", "## Why use it", "", why_use(group), ""]
        if is_class:
            constructor = python_params(signature(obj))
            if constructor:
                lines += ["## Parameters", ""]
                lines += input_table(constructor, methods.get("new", ({}, ""))[0], group, True, name)
        if update:
            rust_params, rust_output = update
            method_obj = getattr(obj, "update", None) if is_class else obj
            params = python_params(signature(method_obj))
            lines += ["## Inputs and output", ""]
            if params:
                lines += input_table(params, rust_params, group, False, name)
            output = python_type(rust_output)
            output_description = OUTPUT_MEANINGS.get(output.replace(" | None", ""))
            lines += [f"**Output:** `{output}`" + (f" — {output_description}" if output_description else "") + ".", ""]
            result_name = output.split(" | ", 1)[0]
            if result_name.endswith("Result"):
                lines += [f"See [{result_name}](/api/{result_name}) for its output fields.", ""]
            if " | None" in output or "[float | None]" in output:
                note = "A `None` result means no value is available yet."
                if is_class and any(param == "update_every" for param, _ in python_params(signature(obj))):
                    note += " It can occur during warmup or between scheduled calculations."
                else:
                    note += " This commonly occurs during warmup."
                if output.startswith("list["):
                    note = note.replace("A `None` result", "A `None` element")
                lines += [note, ""]
            elif output == "None":
                lines += ["The update changes internal state. Read a public property or calculation method for the current result.", ""]
            if is_class and any(param == "update_every" for param, _ in python_params(signature(obj))):
                lines += ["**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.", ""]
        elif is_class:
            lines += ["## Inputs and output", "", "This class exposes its calculation through the public members listed below.", ""]
    if doc.strip():
        lines += ["## Formula and method", "", textwrap.dedent(doc).strip(), ""]
    if name in EXAMPLES:
        lines += ["## Example", "", "```python", EXAMPLES[name], "```", ""]
    if is_class:
        members = []
        for attr in dir(obj):
            if attr.startswith("_"):
                continue
            member = getattr(obj, attr)
            if callable(member):
                description = first_sentence(getattr(member, "__doc__", None))
                if description == "See the public signature and source for behavior.":
                    description = {
                        "reset": "Clear indicator state.",
                        "update": "Consume one observation and return its current result."
                        if update and "Option<" not in update[1]
                        else "Consume one observation and return its result, or `None` when unavailable.",
                        "update_many": "Consume a sequence and return one result per input.",
                        "compute": "Calculate a result from the current state and supplied arguments.",
                        "pnl": "Calculate profit or loss from the current variance estimate.",
                        "portfolio_volatility": "Calculate portfolio volatility for the supplied weights.",
                        "is_locked": "Report whether the phase loop currently considers itself locked.",
                    }.get(attr, description)
                members.append((attr, f"{attr}{signature(member)}", description))
            else:
                members.append((attr, attr, "Property"))
        if members:
            lines += ["## Public members", ""]
            for _, call, description in members:
                lines += [f"- `{call}` — {description}"]
            lines += [""]
    lines += ["[Back to the API catalog](/api/).", ""]
    return "\n".join(lines)


def workbook_rows() -> list[tuple[str, list[list[str]]]]:
    workbook = ROOT / "low_latency_finance_signal_api.xlsx"
    ns = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
    output = []
    with zipfile.ZipFile(workbook) as archive:
        shared = []
        if "xl/sharedStrings.xml" in archive.namelist():
            shared = [
                "".join(t.text or "" for t in item.iter(ns + "t"))
                for item in ET.fromstring(archive.read("xl/sharedStrings.xml"))
            ]
        for sheet_name, title in [
            ("xl/worksheets/sheet1.xml", "Finance"),
            ("xl/worksheets/sheet2.xml", "Signal processing"),
        ]:
            rows = []
            root = ET.fromstring(archive.read(sheet_name))
            for row in root.findall(".//" + ns + "sheetData/" + ns + "row"):
                values = []
                for cell in row.findall(ns + "c"):
                    value = cell.find(ns + "v")
                    if value is not None:
                        values.append(shared[int(value.text)] if cell.get("t") == "s" else value.text or "")
                    else:
                        inline = cell.find(ns + "is")
                        values.append("".join(t.text or "" for t in inline.iter(ns + "t")) if inline is not None else "")
                rows.append(values)
            output.append((title, rows))
    return output


def render_workbook() -> str:
    lines = [
        "# Workbook map", "",
        "The <a href=\"/low_latency_finance_signal_api.xlsx\" download>source workbook</a> is qstream's planning catalog. "
        "Its function names are descriptive research names, not necessarily Python export names. "
        "Use the [API catalog](/api/) for exact callable names, signatures, and current behavior.", "",
        "The workbook's *Wickra Status* column records an earlier comparison with Wickra; "
        "it is not a current qstream implementation-status field. The table below includes the "
        "workbook's suggested module, streaming model, and priority without treating that old column as current status.", "",
    ]
    for title, rows in workbook_rows():
        lines += [f"## {title}", "", "| Function in workbook | Suggested module | Streaming model | Priority |", "|---|---|---|---|"]
        for row in rows[1:]:
            values = (row[0], row[4], row[5], row[7])
            safe = [value.replace("|", "\\|").replace("\n", " ") for value in values]
            lines.append("| " + " | ".join(safe) + " |")
        lines.append("")
    return "\n".join(lines)


def main() -> None:
    sys.path.insert(0, str(ROOT / "python"))
    import qstream

    registered = registration_map()
    names = [name for name in qstream.__all__ if name not in ALIASES and name != "__version__"]
    missing = sorted(set(names) - set(registered))
    if missing:
        raise RuntimeError(f"Public exports missing from src/lib.rs registrations: {missing}")
    # Fail closed if a binding changes in a way the generated input/output
    # tables cannot represent. This prevents silently publishing stale types.
    for name in names:
        obj = getattr(qstream, name)
        group, source = registered[name]
        if group.endswith("Result types"):
            if not result_fields(name, source):
                raise RuntimeError(f"No readable output fields for {name}")
            continue
        if inspect.isclass(obj):
            methods = class_methods(name, source)
            constructor = [arg for arg, _ in python_params(signature(obj))]
            if constructor != list(methods.get("new", ({}, ""))[0]):
                raise RuntimeError(f"Constructor parameter mismatch for {name}")
            if hasattr(obj, "update"):
                update = methods.get("update")
                if not update or [arg for arg, _ in python_params(signature(obj.update))] != list(update[0]):
                    raise RuntimeError(f"Update parameter mismatch for {name}")
        elif not function_method(name, source):
            raise RuntimeError(f"No readable function signature for {name}")
    API.mkdir(parents=True, exist_ok=True)
    groups: dict[str, list[str]] = defaultdict(list)
    for name in sorted(names):
        group, source = registered[name]
        obj = getattr(qstream, name)
        groups[group].append(name)
        (API / f"{name}.md").write_text(render_api_page(name, obj, group, source))

    lines = [
        "# Python API catalog", "",
        f"This catalog documents **{len(names)} canonical public callables** in the installed "
        f"qstream {qstream.__version__} build. Signatures and descriptions come from the native "
        "Python module; categories follow registration in `src/lib.rs`.", "",
        "The *Experimental* groups use the project's Phase 7 terminology; these exports "
        "are available in the main package without a feature gate.", "",
        "Select a name for its constructor or function signature, public methods, and source location. "
        "Generated pages should be refreshed whenever bindings change.", "",
    ]
    ordered_groups = dict.fromkeys(group for group, _ in registered.values())
    for group in ordered_groups:
        group_names = groups.get(group, [])
        if not group_names:
            continue
        lines += [f"## {group}", "", " · ".join(f"[{name}](/api/{name})" for name in group_names), ""]
    lines += [
        "## Convenience aliases", "",
        "These names refer to canonical exports and do not have separate implementations.", "",
        "| Alias | Canonical export |", "|---|---|",
    ]
    for alias in sorted(ALIASES):
        canonical = getattr(qstream, alias).__name__
        lines.append(f"| `{alias}` | [{canonical}](/api/{canonical}) |")
    lines.append("")
    (API / "index.md").write_text("\n".join(lines))
    (DOCS / "workbook.md").write_text(render_workbook())
    public = DOCS / "public"
    public.mkdir(exist_ok=True)
    shutil.copyfile(ROOT / "low_latency_finance_signal_api.xlsx", public / "low_latency_finance_signal_api.xlsx")
    print(f"Generated {len(names)} API pages and workbook map")


if __name__ == "__main__":
    main()
