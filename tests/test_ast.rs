use pandorust::ast::*;
use std::collections::HashMap;

#[test]
fn test_create_empty_document() {
    let doc = Document {
        meta: Meta::default(),
        blocks: vec![],
    };
    assert!(doc.blocks.is_empty());
}

#[test]
fn test_meta_accessors() {
    let mut entries = HashMap::new();
    entries.insert("title".to_string(), MetaValue::String("My Doc".to_string()));
    entries.insert("author".to_string(), MetaValue::String("Tester".to_string()));
    entries.insert("date".to_string(), MetaValue::String("2026-01-01".to_string()));
    entries.insert("subtitle".to_string(), MetaValue::String("A subtitle".to_string()));

    let meta = Meta { entries };
    assert_eq!(meta.title(), Some("My Doc"));
    assert_eq!(meta.author(), Some("Tester"));
    assert_eq!(meta.date(), Some("2026-01-01"));
    assert_eq!(meta.subtitle(), Some("A subtitle"));
}

#[test]
fn test_meta_missing_fields_return_none() {
    let meta = Meta::default();
    assert_eq!(meta.title(), None);
    assert_eq!(meta.author(), None);
}

#[test]
fn test_attr_empty() {
    let attr = Attr::empty();
    assert!(attr.id.is_empty());
    assert!(attr.classes.is_empty());
    assert!(attr.attrs.is_empty());
}

#[test]
fn test_create_paragraph_block() {
    let block = Block::Para(vec![Inline::Str("Hello".to_string())]);
    match block {
        Block::Para(inlines) => assert_eq!(inlines.len(), 1),
        _ => panic!("Expected Para"),
    }
}

#[test]
fn test_create_heading_block() {
    let block = Block::Heading(
        Attr::empty(),
        1,
        vec![Inline::Str("Title".to_string())],
    );
    match block {
        Block::Heading(_, level, inlines) => {
            assert_eq!(level, 1);
            assert_eq!(inlines.len(), 1);
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_create_table_with_header_and_body() {
    let header_cell = Cell {
        attr: Attr::empty(),
        align: Alignment::AlignDefault,
        row_span: 1,
        col_span: 1,
        content: vec![Block::Plain(vec![Inline::Str("Name".to_string())])],
    };
    let body_cell = Cell {
        attr: Attr::empty(),
        align: Alignment::AlignDefault,
        row_span: 1,
        col_span: 1,
        content: vec![Block::Plain(vec![Inline::Str("Alice".to_string())])],
    };

    let table = Table {
        attr: Attr::empty(),
        caption: Caption::default(),
        col_specs: vec![ColSpec {
            align: Alignment::AlignLeft,
            width: ColWidth::Default,
        }],
        head: TableHead {
            attr: Attr::empty(),
            rows: vec![Row {
                attr: Attr::empty(),
                cells: vec![header_cell],
            }],
        },
        bodies: vec![TableBody {
            attr: Attr::empty(),
            row_head_columns: 0,
            head: vec![],
            body: vec![Row {
                attr: Attr::empty(),
                cells: vec![body_cell],
            }],
        }],
        foot: TableFoot {
            attr: Attr::empty(),
            rows: vec![],
        },
    };

    assert_eq!(table.head.rows.len(), 1);
    assert_eq!(table.bodies[0].body.len(), 1);
    assert_eq!(table.col_specs.len(), 1);
}

#[test]
fn test_inline_strong_contains_children() {
    let inline = Inline::Strong(vec![
        Inline::Str("bold".to_string()),
    ]);
    match inline {
        Inline::Strong(children) => assert_eq!(children.len(), 1),
        _ => panic!("Expected Strong"),
    }
}

#[test]
fn test_list_attrs_default() {
    let attrs = ListAttrs::default();
    assert_eq!(attrs.start, 1);
    assert_eq!(attrs.style, ListNumberStyle::Decimal);
    assert_eq!(attrs.delim, ListNumberDelim::Period);
}

#[test]
fn test_page_break_block() {
    let block = Block::PageBreak;
    assert!(matches!(block, Block::PageBreak));
}
