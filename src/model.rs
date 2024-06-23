use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::note;

#[derive(Debug)]
pub struct Model {
    topics: HashMap<String, Topic>,
}

#[derive(Debug)]
pub struct Topic {
    notes: BTreeMap<time::PrimitiveDateTime, note::Note>,
}
