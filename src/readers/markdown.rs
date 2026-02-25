use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{parse_document, Arena, Options};

use crate::ast::*;
use crate::utils::error::Result;

/// Parse a markdown string into a Document AST.
pub fn read_markdown(input: &str) -> Result<Document> {
    let (yaml, body) = split_front_matter(input);
    let meta = parse_yaml_meta(yaml)?;

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;

    let root = parse_document(&arena, body, &options);
    let blocks = convert_children(root);

    Ok(Document { meta, blocks })
}

fn split_front_matter(input: &str) -> (Option<&str>, &str) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (None, input);
    }

    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find("\n---") {
        let yaml = after_open[..close_pos].trim();
        let body = &after_open[close_pos + 4..];
        (Some(yaml), body)
    } else {
        (None, input)
    }
}

fn parse_yaml_meta(yaml: Option<&str>) -> Result<Meta> {
    let mut meta = Meta::default();

    if let Some(yaml_str) = yaml {
        if yaml_str.is_empty() {
            return Ok(meta);
        }
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)?;
        if let serde_yaml::Value::Mapping(map) = value {
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    meta.entries.insert(key, yaml_to_meta(v));
                }
            }
        }
    }

    Ok(meta)
}

fn yaml_to_meta(value: serde_yaml::Value) -> MetaValue {
    match value {
        serde_yaml::Value::String(s) => MetaValue::String(s),
        serde_yaml::Value::Bool(b) => MetaValue::Bool(b),
        serde_yaml::Value::Number(n) => MetaValue::String(n.to_string()),
        serde_yaml::Value::Sequence(seq) => {
            MetaValue::List(seq.into_iter().map(yaml_to_meta).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut m = std::collections::HashMap::new();
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    m.insert(key, yaml_to_meta(v));
                }
            }
            MetaValue::Map(m)
        }
        _ => MetaValue::String(String::new()),
    }
}

fn convert_children<'a>(node: &'a AstNode<'a>) -> Vec<Block> {
    node.children().map(convert_node).collect()
}

fn convert_node<'a>(node: &'a AstNode<'a>) -> Block {
    match &node.data.borrow().value {
        NodeValue::Paragraph => Block::Para(collect_inlines(node)),
        NodeValue::Heading(heading) => {
            Block::Heading(Attr::empty(), heading.level, collect_inlines(node))
        }
        NodeValue::CodeBlock(code) => {
            let lang = code.info.clone();
            let attr = if lang.is_empty() {
                Attr::empty()
            } else {
                Attr {
                    id: String::new(),
                    classes: vec![lang],
                    attrs: vec![],
                }
            };
            Block::CodeBlock(attr, code.literal.clone())
        }
        NodeValue::BlockQuote => Block::BlockQuote(convert_children(node)),
        NodeValue::List(list) => {
            let items: Vec<Vec<Block>> =
                node.children().map(|item| convert_children(item)).collect();
            match list.list_type {
                ListType::Bullet => Block::BulletList(items),
                ListType::Ordered => Block::OrderedList(
                    ListAttrs {
                        start: list.start as u32,
                        ..Default::default()
                    },
                    items,
                ),
            }
        }
        NodeValue::ThematicBreak => Block::HorizontalRule,
        NodeValue::Table(table_data) => convert_table(node, table_data),
        NodeValue::HtmlBlock(html) => {
            let content = html.literal.trim();
            if content == "<div style=\"page-break-after: always;\"></div>"
                || content == "\\newpage"
            {
                Block::PageBreak
            } else {
                Block::RawBlock(Format("html".into()), html.literal.clone())
            }
        }
        _ => {
            let inlines = collect_inlines(node);
            if inlines.is_empty() {
                Block::Plain(vec![])
            } else {
                Block::Para(inlines)
            }
        }
    }
}

fn convert_table<'a>(
    node: &'a AstNode<'a>,
    table_data: &comrak::nodes::NodeTable,
) -> Block {
    let col_specs: Vec<ColSpec> = table_data
        .alignments
        .iter()
        .map(|a| ColSpec {
            align: match a {
                TableAlignment::Left => Alignment::AlignLeft,
                TableAlignment::Right => Alignment::AlignRight,
                TableAlignment::Center => Alignment::AlignCenter,
                TableAlignment::None => Alignment::AlignDefault,
            },
            width: ColWidth::Default,
        })
        .collect();

    let mut head_rows = Vec::new();
    let mut body_rows = Vec::new();

    for (i, row_node) in node.children().enumerate() {
        let cells: Vec<Cell> = row_node
            .children()
            .map(|cell_node| Cell {
                attr: Attr::empty(),
                align: Alignment::AlignDefault,
                row_span: 1,
                col_span: 1,
                content: vec![Block::Plain(collect_inlines(cell_node))],
            })
            .collect();

        let row = Row {
            attr: Attr::empty(),
            cells,
        };

        if i == 0 {
            head_rows.push(row);
        } else {
            body_rows.push(row);
        }
    }

    Block::Table(Table {
        attr: Attr::empty(),
        caption: Caption::default(),
        col_specs,
        head: TableHead {
            attr: Attr::empty(),
            rows: head_rows,
        },
        bodies: vec![TableBody {
            attr: Attr::empty(),
            row_head_columns: 0,
            head: vec![],
            body: body_rows,
        }],
        foot: TableFoot {
            attr: Attr::empty(),
            rows: vec![],
        },
    })
}

fn collect_inlines<'a>(node: &'a AstNode<'a>) -> Vec<Inline> {
    node.children().flat_map(convert_inline).collect()
}

fn convert_inline<'a>(node: &'a AstNode<'a>) -> Vec<Inline> {
    match &node.data.borrow().value {
        NodeValue::Text(text) => vec![Inline::Str(text.to_string())],
        NodeValue::SoftBreak => vec![Inline::SoftBreak],
        NodeValue::LineBreak => vec![Inline::LineBreak],
        NodeValue::Code(code) => vec![Inline::Code(Attr::empty(), code.literal.clone())],
        NodeValue::Emph => vec![Inline::Emph(collect_inlines(node))],
        NodeValue::Strong => vec![Inline::Strong(collect_inlines(node))],
        NodeValue::Strikethrough => vec![Inline::Strikeout(collect_inlines(node))],
        NodeValue::Superscript => vec![Inline::Superscript(collect_inlines(node))],
        NodeValue::Link(link) => vec![Inline::Link(
            Attr::empty(),
            collect_inlines(node),
            Target {
                url: link.url.clone(),
                title: link.title.clone(),
            },
        )],
        NodeValue::Image(link) => vec![Inline::Image(
            Attr::empty(),
            collect_inlines(node),
            Target {
                url: link.url.clone(),
                title: link.title.clone(),
            },
        )],
        NodeValue::HtmlInline(html) => {
            vec![Inline::RawInline(Format("html".into()), html.clone())]
        }
        _ => collect_inlines(node),
    }
}
