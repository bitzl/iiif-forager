pub mod metadata;
pub mod types;

use crate::iiif::metadata::Metadata;
use crate::iiif::types::{Id, IiifUrls, Uri};
use crate::image::ImageInfo;

use serde::Serialize;

const PRESENTATION: &str = "http://iiif.io/api/presentation/3/context.json";

#[derive(Debug, Serialize)]
enum Motivation {
    #[serde(rename = "painting")]
    Painting,
}

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
    pub fn new(
        iiif_urls: &IiifUrls,
        item_id: &Id,
        label: &str,
        metadata: Vec<Metadata>,
        description: Option<String>,
    ) -> Manifest {
        Manifest {
            context: Uri::new(PRESENTATION),
            id: iiif_urls.manifest_id(item_id),
            label: label.to_owned(),
            metadata,
            description,
            items: Vec::new(),
        }
    }

    pub fn add_image(
        &mut self,
        iiif_urls: &IiifUrls,
        item_id: &Id,
        image_id: &Id,
        label: &str,
        image_info: &ImageInfo,
    ) {
        let index = self.items.len();
        let mut canvas = Canvas::new(
            iiif_urls,
            item_id,
            index,
            label,
            image_info.width,
            image_info.height,
        );
        let image_resource = Image::new(iiif_urls, image_id, image_info);
        let annotation = Annotation::new(
            iiif_urls.annotation_id(item_id, index, "image"),
            Resource::Image(image_resource),
            (&canvas.id).clone(),
        );
        let annotation_page = AnnotationPage {
            id: iiif_urls.annotation_page_id(item_id, index),
            items: vec![annotation],
        };
        &canvas.add_item(annotation_page);
        for label in &image_info.labels {
            let body = match label {
                crate::image::Label::KV(key, value) => {
                    format!("<strong>{}:</strong> {}", key, value)
                }
            };
        }
        self.items.push(canvas);
    }
}

#[derive(Debug, Serialize)]
pub struct Service {
    id: Uri,
    profile: Uri,
    protocol: Uri,
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
    pub fn new(
        iiif_urls: &IiifUrls,
        item_id: &Id,
        index: usize,
        label: &str,
        width: u32,
        height: u32,
    ) -> Canvas {
        Canvas {
            id: iiif_urls.canvas_id(&item_id, index),
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

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
struct AnnotationPage {
    id: Uri,
    items: Vec<Annotation>,
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
    pub fn new(id: Uri, resource: Resource, target: Uri) -> Annotation {
        Annotation {
            id,
            motivation: Motivation::Painting,
            body: resource,
            target,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Resource {
    Image(Image),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct Image {
    id: Uri,
    format: String,
    service: ImageService2,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(iiif_urls: &IiifUrls, image_id: &Id, image_info: &ImageInfo) -> Image {
        Image {
            id: iiif_urls.image_id(image_id, &image_info.format),
            format: image_info.format.media_type().to_owned(),
            service: ImageService2::new(iiif_urls.image_service_id(&image_id)),
            width: image_info.width,
            height: image_info.height,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct ImageService2 {
    id: Uri,
    profile: String,
}

impl ImageService2 {
    fn new(id: Uri) -> ImageService2 {
        ImageService2 {
            id,
            profile: "level2".to_owned(),
        }
    }
}
