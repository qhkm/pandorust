use pandorust::readers::markdown::read_markdown;
use pandorust::writers::docx::write_docx;
use pandorust::writers::html::write_html;

#[test]
fn test_full_document_md_to_html() {
    let md = r#"---
title: Project Proposal
author: Test Author
date: 2026-01-01
---

# Introduction

This is a **comprehensive** test of the full pipeline.

## Features

- Markdown parsing with *front matter*
- HTML output with tables
- DOCX output with **borders**

| Feature | Status |
|---------|--------|
| Parser | Done |
| HTML | Done |
| DOCX | Done |

### Code Example

```rust
fn main() {
    println!("Hello, Pandorust!");
}
```

> This is a blockquote with **bold** and *italic*.

---

1. First ordered item
2. Second ordered item
3. Third ordered item
"#;

    let doc = read_markdown(md).unwrap();

    // Verify the document was parsed correctly
    assert!(
        !doc.blocks.is_empty(),
        "Document should have parsed blocks"
    );
    assert_eq!(
        doc.meta.title(),
        Some("Project Proposal"),
        "Title should be parsed from YAML front matter"
    );
    assert_eq!(
        doc.meta.author(),
        Some("Test Author"),
        "Author should be parsed from YAML front matter"
    );

    // Test HTML output
    let html = write_html(&doc);
    assert!(
        html.contains("<title>Project Proposal</title>"),
        "HTML should contain <title> from front matter"
    );
    assert!(
        html.contains("<h1"),
        "HTML should contain an h1 heading"
    );
    assert!(
        html.contains("<strong>comprehensive</strong>"),
        "HTML should render bold text with <strong>"
    );
    assert!(
        html.contains("<table>"),
        "HTML should contain a table"
    );
    assert!(
        html.contains("<th"),
        "HTML should contain table header cells"
    );
    assert!(
        html.contains("<ul>"),
        "HTML should contain an unordered list"
    );
    assert!(
        html.contains("<ol>"),
        "HTML should contain an ordered list"
    );
    assert!(
        html.contains("<blockquote>"),
        "HTML should contain a blockquote"
    );
    assert!(
        html.contains("<pre><code"),
        "HTML should contain a code block"
    );
    assert!(
        html.contains("<hr>"),
        "HTML should contain a horizontal rule"
    );

    // Test DOCX output
    let docx_bytes = write_docx(&doc).unwrap();
    assert!(
        docx_bytes.len() > 1000,
        "DOCX should have substantial content, got {} bytes",
        docx_bytes.len()
    );
    assert_eq!(
        &docx_bytes[0..2],
        b"PK",
        "DOCX should start with PK zip magic bytes"
    );
}

#[test]
fn test_full_pipeline_preserves_inline_formatting() {
    let md = r#"A paragraph with **bold**, *italic*, ~~strikethrough~~, and `code`.
"#;

    let doc = read_markdown(md).unwrap();
    let html = write_html(&doc);

    assert!(html.contains("<strong>bold</strong>"), "Bold should be preserved");
    assert!(html.contains("<em>italic</em>"), "Italic should be preserved");
    assert!(html.contains("<del>strikethrough</del>"), "Strikethrough should be preserved");
    assert!(html.contains("<code>code</code>"), "Inline code should be preserved");
}

#[test]
fn test_full_pipeline_no_metadata() {
    let md = "# Simple Document\n\nJust a paragraph.\n";

    let doc = read_markdown(md).unwrap();
    let html = write_html(&doc);

    assert!(!html.contains("<title>"), "No title tag when no front matter");
    assert!(html.contains("<h1"), "Should still have heading");
    assert!(html.contains("Just a paragraph"), "Should have paragraph text");

    // DOCX should also work without metadata
    let docx_bytes = write_docx(&doc).unwrap();
    assert_eq!(&docx_bytes[0..2], b"PK", "DOCX should be a valid zip");
}
