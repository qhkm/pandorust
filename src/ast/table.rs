use super::block::Block;
use super::inline::Inline;
use super::Attr;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Alignment {
    #[default]
    AlignDefault,
    AlignLeft,
    AlignRight,
    AlignCenter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColWidth {
    Fixed(f64),
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColSpec {
    pub align: Alignment,
    pub width: ColWidth,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Caption {
    pub short: Option<Vec<Inline>>,
    pub long: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub attr: Attr,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub attr: Attr,
    pub align: Alignment,
    pub row_span: u32,
    pub col_span: u32,
    pub content: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableHead {
    pub attr: Attr,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableBody {
    pub attr: Attr,
    pub row_head_columns: u32,
    pub head: Vec<Row>,
    pub body: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableFoot {
    pub attr: Attr,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub attr: Attr,
    pub caption: Caption,
    pub col_specs: Vec<ColSpec>,
    pub head: TableHead,
    pub bodies: Vec<TableBody>,
    pub foot: TableFoot,
}
