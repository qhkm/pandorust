use pandorust::ast::*;
use pandorust::readers::markdown::read_markdown;

#[test]
fn test_parse_heading() {
    let doc = read_markdown("# Hello World").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    match &doc.blocks[0] {
        Block::Heading(_, level, inlines) => {
            assert_eq!(level, &1);
            assert!(matches!(&inlines[0], Inline::Str(s) if s == "Hello World"));
        }
        other => panic!("Expected Heading, got {:?}", other),
    }
}

#[test]
fn test_parse_paragraph_with_bold() {
    let doc = read_markdown("This is **bold** text").unwrap();
    assert_eq!(doc.blocks.len(), 1);
    match &doc.blocks[0] {
        Block::Para(inlines) => {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Strong(_))));
        }
        other => panic!("Expected Para, got {:?}", other),
    }
}

#[test]
fn test_parse_table() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |";
    let doc = read_markdown(md).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    match &doc.blocks[0] {
        Block::Table(table) => {
            assert_eq!(table.head.rows.len(), 1);
            assert_eq!(table.head.rows[0].cells.len(), 2);
            assert_eq!(table.bodies.len(), 1);
            assert_eq!(table.bodies[0].body.len(), 2);
        }
        other => panic!("Expected Table, got {:?}", other),
    }
}

#[test]
fn test_parse_yaml_front_matter() {
    let md = "---\ntitle: My Doc\nauthor: Test\ndate: 2026-01-01\n---\n\n# Hello";
    let doc = read_markdown(md).unwrap();
    assert_eq!(doc.meta.title(), Some("My Doc"));
    assert_eq!(doc.meta.author(), Some("Test"));
    assert_eq!(doc.meta.date(), Some("2026-01-01"));
    assert_eq!(doc.blocks.len(), 1);
}

#[test]
fn test_parse_bullet_list() {
    let md = "- Item A\n- Item B\n- Item C";
    let doc = read_markdown(md).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    match &doc.blocks[0] {
        Block::BulletList(items) => assert_eq!(items.len(), 3),
        other => panic!("Expected BulletList, got {:?}", other),
    }
}

#[test]
fn test_parse_code_block() {
    let md = "```rust\nfn main() {}\n```";
    let doc = read_markdown(md).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    match &doc.blocks[0] {
        Block::CodeBlock(attr, code) => {
            assert_eq!(attr.classes, vec!["rust"]);
            assert!(code.contains("fn main()"));
        }
        other => panic!("Expected CodeBlock, got {:?}", other),
    }
}

#[test]
fn test_parse_link() {
    let md = "[click here](https://example.com)";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::Para(inlines) => {
            assert!(inlines.iter().any(|i| matches!(
                i,
                Inline::Link(_, _, Target { url, .. }) if url == "https://example.com"
            )));
        }
        other => panic!("Expected Para with Link, got {:?}", other),
    }
}

#[test]
fn test_parse_horizontal_rule() {
    let md = "Above\n\n---\n\nBelow";
    let doc = read_markdown(md).unwrap();
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::HorizontalRule)));
}

#[test]
fn test_parse_ordered_list() {
    let md = "1. First\n2. Second\n3. Third";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::OrderedList(attrs, items) => {
            assert_eq!(attrs.start, 1);
            assert_eq!(items.len(), 3);
        }
        other => panic!("Expected OrderedList, got {:?}", other),
    }
}

#[test]
fn test_parse_blockquote() {
    let md = "> This is a quote";
    let doc = read_markdown(md).unwrap();
    assert!(matches!(&doc.blocks[0], Block::BlockQuote(_)));
}

#[test]
fn test_parse_strikethrough() {
    let md = "This is ~~deleted~~ text";
    let doc = read_markdown(md).unwrap();
    match &doc.blocks[0] {
        Block::Para(inlines) => {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Strikeout(_))));
        }
        other => panic!("Expected Para with Strikeout, got {:?}", other),
    }
}

#[test]
fn test_parse_no_front_matter() {
    let doc = read_markdown("Just a paragraph").unwrap();
    assert_eq!(doc.meta.title(), None);
    assert_eq!(doc.blocks.len(), 1);
}
