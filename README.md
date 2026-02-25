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

**CLI tool:**

```bash
cargo install pandorust
```

**Library (no clap overhead):**

```toml
[dependencies]
pandorust = { version = "0.1", default-features = false }
```

Or build from source:

```bash
git clone https://github.com/qhkm/pandorust.git
cd pandorust
cargo build --release
# Binary at target/release/pandorust (~5 MB)
```

## CLI Usage

```bash
# Markdown to HTML
pandorust input.md -o output.html

# Markdown to DOCX
pandorust input.md -o output.docx

# Explicit format flags
pandorust input.md -f markdown -t html -o output.html

# Read from stdin
cat input.md | pandorust - -t html -o output.html

# List supported formats
pandorust --list-formats
```

## Library Usage

### Markdown to HTML

```rust
use pandorust::readers::markdown::read_markdown;
use pandorust::writers::html::write_html;

let md = "# Hello\n\nA **bold** paragraph.";
let doc = read_markdown(md).unwrap();
let html = write_html(&doc);

assert!(html.contains("<h1"));
assert!(html.contains("<strong>bold</strong>"));
```

### Markdown to DOCX

```rust
use pandorust::readers::markdown::read_markdown;
use pandorust::writers::docx::write_docx;
use std::fs;

let md = "# Report\n\nGenerated from Rust.";
let doc = read_markdown(md).unwrap();
let bytes = write_docx(&doc).unwrap();

fs::write("report.docx", bytes).unwrap();
```

### With YAML Front Matter

```rust
use pandorust::readers::markdown::read_markdown;
use pandorust::writers::docx::write_docx;

let md = r#"---
title: Quarterly Report
author: Finance Team
fontsize: 11pt
---

# Q1 Results

Revenue grew **15%** compared to last quarter.

| Metric   | Q4 2025 | Q1 2026 |
|----------|---------|---------|
| Revenue  | RM 450k | RM 518k |
| Expenses | RM 320k | RM 335k |
| Profit   | RM 130k | RM 183k |
"#;

let doc = read_markdown(md).unwrap();

// title, author, fontsize are applied to DOCX metadata and styling
let bytes = write_docx(&doc).unwrap();
```

### Working with the AST

```rust
use pandorust::readers::markdown::read_markdown;
use pandorust::ast::{Block, Inline};

let doc = read_markdown("# Title\n\nSome text.").unwrap();

for block in &doc.blocks {
    match block {
        Block::Header(level, _, inlines) => {
            println!("H{}: {:?}", level, inlines);
        }
        Block::Para(inlines) => {
            println!("Paragraph: {:?}", inlines);
        }
        _ => {}
    }
}

// Access metadata
if let Some(title) = doc.meta.title() {
    println!("Title: {}", title);
}
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

| Key | Effect |
|-----|--------|
| `title` | HTML `<title>`, DOCX core properties |
| `author` | DOCX core properties |
| `date` | DOCX core properties |
| `fontsize` | Body text size (default: 12pt) |

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

76 tests across 7 test files covering AST construction, markdown parsing, HTML output, DOCX output, grid tables, CLI, and end-to-end integration.

## Dependencies

| Crate | Purpose | Library | CLI |
|-------|---------|---------|-----|
| comrak 0.50 | GFM markdown parsing | yes | yes |
| docx-rs 0.4 | DOCX generation | yes | yes |
| serde + serde_yaml | YAML front matter | yes | yes |
| thiserror 2 | Error types | yes | yes |
| clap 4.5 | CLI argument parsing | no | yes |

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
