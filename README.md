# pandorust

A pure-Rust document converter. Single binary, no runtime dependencies.

**Markdown → HTML | DOCX** (PDF and PPTX planned)

## Features

- **Markdown reader** — GFM (GitHub Flavored Markdown) via comrak, with YAML front matter
- **Grid table support** — Pandoc-style `+---+---+` grid tables converted automatically
- **HTML writer** — Styled output with Calibri font, table styling, syntax-highlighted code blocks
- **DOCX writer** — Professional Word documents with fonts, spacing, table borders, and metadata
- **Font control** — Set `fontsize: 11pt` in YAML front matter; Calibri body font throughout
- **CLI** — Auto-detects formats from file extensions

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
git clone https://github.com/nicenoz/pandorust.git
cd pandorust
cargo build --release
# Binary at target/release/pandorust (~5 MB)
```

## Usage

```bash
# Markdown to HTML
pandorust input.md -o output.html

# Markdown to DOCX
pandorust input.md -o output.docx

# Explicit format flags
pandorust input.md -f markdown -t html -o output.html
```

### YAML Front Matter

```markdown
---
title: My Document
author: John Doe
date: 2026-02-25
fontsize: 11pt
---

# Content starts here
```

The `title`, `author`, and `date` fields are included in DOCX metadata. `fontsize` controls body text size (default: 12pt).

## Architecture

```
src/
├── ast/           # Document AST (15 Block types, 18 Inline types)
│   ├── block.rs   # Block-level elements
│   ├── inline.rs  # Inline elements
│   ├── meta.rs    # Document metadata
│   └── table.rs   # Pandoc-compatible table model
├── readers/
│   ├── markdown.rs    # comrak → AST
│   └── grid_table.rs  # Grid table preprocessor
├── writers/
│   ├── html.rs    # AST → styled HTML
│   └── docx.rs    # AST → DOCX (via docx-rs)
├── utils/
│   └── error.rs   # Error types
├── main.rs        # CLI (clap)
└── lib.rs         # Library exports
```

### AST

The internal AST is inspired by pandoc's type system:

- **Block**: `Para`, `Header`, `CodeBlock`, `BlockQuote`, `BulletList`, `OrderedList`, `Table`, `HorizontalRule`, `RawBlock`, `Div`, `LineBlock`, `DefinitionList`
- **Inline**: `Str`, `Space`, `Emph`, `Strong`, `Code`, `Link`, `Image`, `LineBreak`, `SoftBreak`, `Strikeout`, `Superscript`, `Subscript`, `SmallCaps`, `Quoted`, `RawInline`, `Math`, `Note`, `Span`
- **Table**: `TableHead`, `TableBody`, `TableFoot`, `Row`, `Cell` with column alignments and widths

## Tests

```bash
cargo test
```

74 tests across 7 test files covering AST construction, markdown parsing, HTML output, DOCX output, grid tables, CLI, and end-to-end integration.

## Dependencies

| Crate | Purpose |
|-------|---------|
| comrak 0.50 | GFM markdown parsing |
| docx-rs 0.4 | DOCX generation |
| clap 4.5 | CLI argument parsing |
| serde + serde_yaml | YAML front matter |
| thiserror 2 | Error types |

## Roadmap

- [ ] PDF writer
- [ ] PPTX writer
- [ ] LaTeX writer
- [ ] RST reader
- [ ] HTML reader
- [ ] Pandoc filter compatibility
- [ ] Template support
- [ ] Citation processing (CSL)

## License

MIT
