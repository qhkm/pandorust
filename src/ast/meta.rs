use std::collections::HashMap;

use super::block::Block;
use super::inline::Inline;

#[derive(Debug, Clone)]
pub struct Document {
    pub meta: Meta,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub entries: HashMap<String, MetaValue>,
}

impl Meta {
    pub fn title(&self) -> Option<&str> {
        match self.entries.get("title") {
            Some(MetaValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn subtitle(&self) -> Option<&str> {
        match self.entries.get("subtitle") {
            Some(MetaValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn author(&self) -> Option<&str> {
        match self.entries.get("author") {
            Some(MetaValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn date(&self) -> Option<&str> {
        match self.entries.get("date") {
            Some(MetaValue::String(s)) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetaValue {
    String(String),
    Bool(bool),
    List(Vec<MetaValue>),
    Map(HashMap<String, MetaValue>),
    Inlines(Vec<Inline>),
    Blocks(Vec<Block>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Attr {
    pub id: String,
    pub classes: Vec<String>,
    pub attrs: Vec<(String, String)>,
}

impl Attr {
    pub fn empty() -> Self {
        Self::default()
    }
}
