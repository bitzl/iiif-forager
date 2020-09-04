use crate::iiif::annotations::{Annotation, AnnotationPage};
use crate::iiif::metadata::Metadata;
use crate::iiif::types::Id;
use crate::iiif::types::Uri;
use crate::iiif::{IiifImage, Resource, PRESENTATION};
use crate::image::source::Image;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type")] // valid Presentation API v3
pub struct Manifest {
    #[serde(rename = "@context")]
    context: Uri,
    id: Uri,
    label: String,
    metadata: Vec<Metadata>,
    description: Option<String>,
    items: Vec<Canvas>,
}

impl Manifest {
    pub fn id(presentation_api: &str, item_id: &Id) -> Uri {
        Uri::new(format!("{}/{}/manifest", presentation_api, item_id.encoded))
    }

    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        label: &str,
        metadata: Vec<Metadata>,
        description: Option<String>,
    ) -> Manifest {
        Manifest {
            context: Uri::new(PRESENTATION),
            id: Manifest::id(presentation_api, item_id),
            label: label.to_owned(),
            metadata,
            description,
            items: Vec::new(),
        }
    }

    pub fn add_image(
        &mut self,
        image_api: &str,
        presentation_api: &str,
        item_id: &Id,
        image_id: &Id,
        label: &str,
        image: &Image,
    ) {
        let index = self.items.len();
        let mut canvas = Canvas::new(
            presentation_api,
            item_id,
            index,
            label,
            image.width,
            image.height,
        );
        let image_resource = IiifImage::new(image_api, image_id, image);
        let annotation = Annotation::new_painting(
            presentation_api,
            item_id,
            index,
            Resource::Image(image_resource),
            (&canvas.id).clone(),
        );
        let annotation_page =
            AnnotationPage::new(presentation_api, item_id, index, vec![annotation]);
        &canvas.add_item(annotation_page);
        self.items.push(canvas);
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct Canvas {
    id: Uri,
    label: String,
    height: u32,
    width: u32,
    items: Vec<AnnotationPage>,
}

impl Canvas {
    pub fn id(presentation_api: &str, item_id: &Id, index: usize) -> Uri {
        Uri::new(format!(
            "{}/{}/canvas/{}",
            presentation_api, item_id.encoded, index
        ))
    }
    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        label: &str,
        width: u32,
        height: u32,
    ) -> Canvas {
        Canvas {
            id: Canvas::id(presentation_api, item_id, index),
            label: label.to_owned(),
            height,
            width,
            items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: AnnotationPage) {
        self.items.push(item);
    }
}
