use crate::iiif::types::Uri;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type")] // valid Presentation API v3
pub struct Collection {
    items: Vec<Item>,
}

impl Collection {
    pub fn new() -> Collection {
        let items: Vec<Item> = Vec::new();
        Collection { items }
    }
    pub fn add_manifest(&mut self, id: Uri) {
        let item = Item {
            id,
            iiif_type: "manifest".to_owned(),
        };
        self.items.push(item);
    }
}

#[derive(Debug, Serialize)]
pub struct Item {
    id: Uri,
    iiif_type: String,
}
