use crate::iiif::resources::Resource;
use crate::iiif::types::{Id, Uri};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Motivation {
    #[serde(rename = "painting")]
    Painting,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct AnnotationPage {
    id: Uri,
    items: Vec<Annotation>,
}

impl AnnotationPage {
    pub fn id(presentation_api: &str, item_id: &Id, index: usize) -> Uri {
        Uri::new(format!(
            "{}/{}/page/{}",
            presentation_api, item_id.encoded, index
        ))
    }

    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        items: Vec<Annotation>,
    ) -> AnnotationPage {
        AnnotationPage {
            id: AnnotationPage::id(presentation_api, item_id, index),
            items,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct Annotation {
    id: Uri,
    motivation: Motivation,
    body: Resource,
    target: Uri,
}

impl Annotation {
    pub fn id(presentation_api: &str, item_id: &Id, page: usize, suffix: &str) -> Uri {
        Uri::new(format!(
            "{}/{}/annotation/{}-{}",
            presentation_api, item_id.encoded, page, suffix
        ))
    }

    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        resource: Resource,
        target: Uri,
        motivation: Motivation,
    ) -> Annotation {
        let id = match resource {
            Resource::Image(_) => Annotation::id(presentation_api, item_id, index, "image"),
        };
        Annotation {
            id,
            motivation,
            body: resource,
            target,
        }
    }

    pub fn new_painting(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        resource: Resource,
        target: Uri,
    ) -> Annotation {
        Annotation::new(
            presentation_api,
            item_id,
            index,
            resource,
            target,
            Motivation::Painting,
        )
    }
}
