# qstream documentation

This VitePress site follows the structure of the Wickra documentation while
documenting qstream's Python API and Rust-backed behavior. The project-root
`README.md`, `design.md`, the API workbook, and the installed native module are
the sources for its content. The generated API pages are based on the actual
Python exports and docstrings, not a copy of Wickra's indicator catalog.

## Local development

Requires Node.js 20 or newer. From this directory:

```bash
npm install
npm run dev
npm run build
npm run preview
```

VitePress writes static output to `.vitepress/dist`. No hosting provider or
production URL is assumed; deploy that directory wherever desired.

## Refresh the catalog

After changing the Python bindings or workbook, install the current qstream
extension (`maturin develop --release` from the project root) and run:

```bash
.venv/bin/python docs/scripts/generate_reference.py
```

The script regenerates `docs/api/index.md`, `docs/api/<name>.md`, and
`docs/workbook.md`. It reads `src/lib.rs` for registration categories, the
installed `qstream` module for public signatures and docstrings, and the PyO3
binding source for input/output types and result fields. Human-authored use
cases, parameter explanations, and runnable examples live in
`docs/scripts/reference_content.py`. Do not hand-edit generated pages. The
concept and example pages are maintained by hand.
