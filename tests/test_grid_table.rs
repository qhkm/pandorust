use pandorust::ast::*;
use pandorust::readers::markdown::read_markdown;

#[test]
fn test_simple_grid_table() {
    let md = "\
+-----+-----+
| A   | B   |
+=====+=====+
| 1   | 2   |
+-----+-----+
| 3   | 4   |
+-----+-----+";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::Table(table) => {
            assert_eq!(table.head.rows.len(), 1);
            assert_eq!(table.head.rows[0].cells.len(), 2);
            assert_eq!(table.bodies[0].body.len(), 2);
        }
        other => panic!("Expected Table, got {:?}", other),
    }
}

#[test]
fn test_grid_table_with_header_content() {
    let md = "\
+------+--------+
| Name | Value  |
+======+========+
| foo  | 42     |
+------+--------+";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::Table(table) => {
            // Check header cell text
            let header_cell = &table.head.rows[0].cells[0];
            let text = extract_text(&header_cell.content);
            assert!(
                text.contains("Name"),
                "Header should contain 'Name', got '{}'",
                text
            );
        }
        other => panic!("Expected Table, got {:?}", other),
    }
}

#[test]
fn test_multiline_grid_table_cells() {
    let md = "\
+-----+---------------------------+----------+
| No. | Description               | Cost     |
+=====+===========================+==========+
| 1   | **First item**            | 3,500    |
|     | With extra detail         |          |
|     | And more detail           |          |
+-----+---------------------------+----------+
| 2   | **Second item**           | 3,000    |
|     | Another detail            |          |
+-----+---------------------------+----------+";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::Table(table) => {
            assert_eq!(table.head.rows.len(), 1);
            assert_eq!(table.bodies[0].body.len(), 2);
            // The multiline content should be joined
            let cell = &table.bodies[0].body[0].cells[1];
            let text = extract_text(&cell.content);
            assert!(
                text.contains("First item"),
                "Should contain 'First item', got '{}'",
                text
            );
        }
        other => panic!("Expected Table, got {:?}", other),
    }
}

#[test]
fn test_grid_table_surrounded_by_text() {
    let md = "# Title\n\nSome text before.\n\n\
+-----+-----+
| A   | B   |
+=====+=====+
| 1   | 2   |
+-----+-----+\n\nSome text after.";
    let doc = read_markdown(md).unwrap();
    // Should have: Heading, Para, Table, Para
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::Table(_))));
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::Heading(..))));
}

#[test]
fn test_grid_table_no_header_separator() {
    // Grid table without === header separator (all rows are body)
    let md = "\
+-----+-----+
| A   | B   |
+-----+-----+
| 1   | 2   |
+-----+-----+";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::Table(table) => {
            // First row becomes header in GFM
            assert_eq!(table.head.rows.len(), 1);
        }
        other => panic!("Expected Table, got {:?}", other),
    }
}

#[test]
fn test_newpage_latex_command() {
    let md = "Above\n\n\\newpage\n\nBelow";
    let doc = read_markdown(md).unwrap();
    assert!(
        doc.blocks.iter().any(|b| matches!(b, Block::PageBreak)),
        "Should contain PageBreak, got: {:?}",
        doc.blocks
    );
}

#[test]
fn test_fenced_div_custom_style() {
    let md = "Above\n\n::: {custom-style=\"Footer\"}\n**Kitakod Ventures** | Pembangun Perisian\n:::\n\nBelow";
    let doc = read_markdown(md).unwrap();
    // The fenced div content should be rendered, not literal ::: syntax
    let has_literal_colons = doc.blocks.iter().any(|b| match b {
        Block::Para(inlines) | Block::Plain(inlines) => {
            inlines.iter().any(|i| match i {
                Inline::Str(s) => s.contains(":::"),
                _ => false,
            })
        }
        _ => false,
    });
    assert!(!has_literal_colons, "Should not contain literal ':::' syntax, blocks: {:?}", doc.blocks);
    // The inner content (bold text) should be present
    let all_text = doc.blocks.iter().map(|b| format!("{:?}", b)).collect::<String>();
    assert!(all_text.contains("Kitakod Ventures"), "Should contain 'Kitakod Ventures', got: {}", all_text);
}

#[test]
fn test_standalone_backslash_removed() {
    let md = "Above\n\n\\\n\nBelow";
    let doc = read_markdown(md).unwrap();
    // Standalone backslash should not appear as literal text
    let has_lone_backslash = doc.blocks.iter().any(|b| match b {
        Block::Para(inlines) | Block::Plain(inlines) => {
            inlines.len() == 1 && matches!(&inlines[0], Inline::Str(s) if s.trim() == "\\")
        }
        _ => false,
    });
    assert!(!has_lone_backslash, "Should not have a paragraph with just a backslash, blocks: {:?}", doc.blocks);
}

// Helper to extract text from blocks
fn extract_text(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(|b| match b {
            Block::Plain(inlines) | Block::Para(inlines) => inlines
                .iter()
                .map(|i| match i {
                    Inline::Str(s) => s.clone(),
                    Inline::Space | Inline::SoftBreak => " ".to_string(),
                    Inline::Strong(inner) => inner
                        .iter()
                        .map(|i2| match i2 {
                            Inline::Str(s) => s.clone(),
                            _ => String::new(),
                        })
                        .collect(),
                    _ => String::new(),
                })
                .collect::<String>(),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
