#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Model {
    pub topics: HashMap<String, Topic>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Topic {
    pub notes: BTreeMap<time::PrimitiveDateTime, note::Signed<note::Note>>,
}

impl Topic {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_note(&mut self, note: note::Signed<note::Note>) {
        self.notes.insert(note.inner.created_at, note);
    }
}

use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::note;
