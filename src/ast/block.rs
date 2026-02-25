use super::inline::Inline;
use super::table::Table;
use super::{Attr, Caption, Format, ListAttrs};

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Plain(Vec<Inline>),
    Para(Vec<Inline>),
    LineBlock(Vec<Vec<Inline>>),
    Heading(Attr, u8, Vec<Inline>),
    CodeBlock(Attr, String),
    RawBlock(Format, String),
    BlockQuote(Vec<Block>),
    BulletList(Vec<Vec<Block>>),
    OrderedList(ListAttrs, Vec<Vec<Block>>),
    DefinitionList(Vec<(Vec<Inline>, Vec<Vec<Block>>)>),
    Table(Table),
    Figure(Attr, Caption, Vec<Block>),
    Div(Attr, Vec<Block>),
    HorizontalRule,
    PageBreak,
}
