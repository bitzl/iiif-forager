pub mod metadata;
pub mod types;

use crate::iiif::metadata::Metadata;
use crate::iiif::types::{Id, IiifUrls, Uri};
use crate::image::{ImageInfo, Label};

use serde::Serialize;

const PRESENTATION: &str = "http://iiif.io/api/presentation/2/context.json";

#[derive(Debug, Serialize)]
enum IiifType {
    #[serde(rename = "oa:Annotation")]
    Annotation,
    #[serde(rename = "sc:Canvas")]
    Canvas,
    #[serde(rename = "dctypes:Image")]
    Image,
    #[serde(rename = "sc:Manifest")]
    Manifest,
    #[serde(rename = "sc:Sequence")]
    Sequence,
    #[serde(rename = "dctypes:Text")]
    Text,
}

#[derive(Debug, Serialize)]
enum Motivation {
    #[serde(rename = "oa:commenting")]
    Commenting,
    #[serde(rename = "sc:Painting")]
    Painting,
}

#[derive(Debug, Serialize)]
pub struct Manifest {
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@id")]
    id: Uri,
    #[serde(rename = "@type")]
    iiif_type: IiifType,
    label: String,
    metadata: Vec<Metadata>,
    description: Option<String>,
    sequences: Vec<Sequence>,
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
            iiif_type: IiifType::Manifest,
            label: label.to_owned(),
            metadata,
            description,
            sequences: Vec::new(),
        }
    }

    pub fn add_sequence(&mut self, sequence: Sequence) {
        self.sequences.push(sequence);
    }
}

#[derive(Debug, Serialize)]
pub struct Image {
    #[serde(rename = "@id")]
    id: Uri,
    service: Service,
}

#[derive(Debug, Serialize)]
pub struct Service {
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@id")]
    id: Uri,
    profile: Uri,
    protocol: Uri,
}

impl Service {
    fn new_image_service(iiif_urls: &IiifUrls, image_id: &Id) -> Service {
        Service {
            context: Uri::new("http://iiif.io/api/image/2/context.json"),
            id: iiif_urls.image_service_id(image_id),
            profile: Uri::new("http://iiif.io/api/image/2/level2.json"),
            protocol: Uri::new("http://iiiif.io/api/image".to_owned()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Sequence {
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@id")]
    id: Uri,
    #[serde(rename = "@type")]
    iiif_type: IiifType,
    canvases: Vec<Canvas>,
}

impl Sequence {
    pub fn new(iiif_urls: &IiifUrls, item_id: &Id) -> Sequence {
        Sequence {
            context: Uri::new(PRESENTATION),
            id: iiif_urls.sequence_id(item_id),
            iiif_type: IiifType::Sequence,
            canvases: Vec::new(),
        }
    }

    pub fn add_image(
        &mut self,
        base_urls: &IiifUrls,
        item_id: &Id,
        image_id: &Id,
        label: &str,
        image_info: &ImageInfo,
    ) {
        let index = self.canvases.len();
        let mut canvas = Canvas::new(
            base_urls,
            item_id,
            index,
            label,
            image_info.width,
            image_info.height,
        );
        let image_resource = ImageResource::new(base_urls, image_id, image_info);
        let annotation = Annotation::new(Resource::Image(image_resource), (&canvas.id).clone());
        &canvas.add_image(annotation);
        for label in &image_info.labels {
            let body = match label {
                crate::image::Label::KV(key, value) => {
                    format!("<strong>{}:</strong> {}", key, value)
                }
            };
            let annotation = Annotation::comment(body, &canvas.id);
            canvas.add_annotation(annotation);
        }
        self.canvases.push(canvas);
    }
}

#[derive(Debug, Serialize)]
pub struct Canvas {
    #[serde(rename = "@id")]
    id: Uri,
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@type")]
    iiif_type: IiifType,
    label: String,
    height: u32,
    width: u32,
    images: Vec<Annotation>,
    annotations: Vec<Annotation>,
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
            context: Uri::new(PRESENTATION),
            iiif_type: IiifType::Canvas,
            label: label.to_owned(),
            height,
            width,
            images: Vec::new(),
            annotations: Vec::new(),
        }
    }

    pub fn add_image(&mut self, image: Annotation) {
        self.images.push(image);
    }

    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation)
    }
}

#[derive(Debug, Serialize)]
pub struct Thumbnail {
    id: Uri,
    iiif_type: IiifType,
    height: u32,
    width: u32,
}

#[derive(Debug, Serialize)]
pub struct Annotation {
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@type")]
    iiif_type: IiifType,
    motivation: Motivation,
    resource: Resource,
    on: Uri,
}

impl Annotation {
    pub fn new(resource: Resource, on: Uri) -> Annotation {
        Annotation {
            context: Uri::new(PRESENTATION),
            iiif_type: IiifType::Annotation,
            motivation: Motivation::Painting,
            resource,
            on,
        }
    }
    pub fn comment(body: String, on: &Uri) -> Annotation {
        // "@context": "http://iiif.io/api/presentation/2/context.json",
        // "@id": "http://example.org/iiif/book1/annotation/anno1",
        // "@type": "oa:Annotation",
        // "motivation": "oa:commenting",
        // "resource":{
        //     "@id": "http://example.org/iiif/book1/res/comment1.html",
        //     "@type": "dctypes:Text",
        //     "format": "text/html"
        // },
        // "on": "http://example.org/iiif/book1/canvas/p1"
        let resource = CommentResource {
            iiif_type: IiifType::Text,
            format: "text/html".to_owned(),
            body,
        };
        Annotation {
            context: Uri::new(PRESENTATION),
            iiif_type: IiifType::Annotation,
            motivation: Motivation::Commenting,
            resource: Resource::Comment(resource),
            on: on.to_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Resource {
    Comment(CommentResource),
    Image(ImageResource),
}

#[derive(Debug, Serialize)]
pub struct CommentResource {
    // #[serde(rename = "@id")]
    // id: Uri,
    #[serde(rename = "@type")]
    iiif_type: IiifType,
    format: String,
    body: String,
}

#[derive(Debug, Serialize)]
pub struct ImageResource {
    #[serde(rename = "@id")]
    id: Uri,
    #[serde(rename = "@type")]
    iiif_type: IiifType,
    format: String,
    service: Service,
    width: u32,
    height: u32,
}

impl ImageResource {
    pub fn new(iiif_urls: &IiifUrls, image_id: &Id, image_info: &ImageInfo) -> ImageResource {
        ImageResource {
            id: iiif_urls.image_id(image_id, &image_info.format),
            iiif_type: IiifType::Image,
            format: image_info.format.media_type().to_owned(),
            service: Service::new_image_service(iiif_urls, image_id),
            width: image_info.width,
            height: image_info.height,
        }
    }
}
