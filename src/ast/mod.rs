pub mod block;
pub mod inline;
pub mod meta;
pub mod table;

pub use block::Block;
pub use inline::Inline;
pub use meta::{Attr, Document, Meta, MetaValue};
pub use table::{
    Alignment, Caption, Cell, ColSpec, ColWidth, Row, Table, TableBody, TableFoot, TableHead,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Format(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteType {
    SingleQuote,
    DoubleQuote,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MathType {
    DisplayMath,
    InlineMath,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAttrs {
    pub start: u32,
    pub style: ListNumberStyle,
    pub delim: ListNumberDelim,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListNumberStyle {
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListNumberDelim {
    Period,
    OneParen,
    TwoParens,
}

impl Default for ListAttrs {
    fn default() -> Self {
        Self {
            start: 1,
            style: ListNumberStyle::Decimal,
            delim: ListNumberDelim::Period,
        }
    }
}
