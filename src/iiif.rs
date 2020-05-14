use crate::metadata::ImageMetadata;
use serde::{Deserialize, Serialize};

const CONTEXT: &'static str = "http://iiif.io/api/presentation/2/context.json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    // Many(Vec<String>),
    // Multilang(Vec<LocalizedValue>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalizedValue {
    value: String,
    language: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    label: String,
    value: Value,
}

impl Metadata {
    pub fn key_value(label: &str, value: &str) -> Metadata {
        Metadata {
            label: label.to_owned(),
            value: Value::Single(value.to_owned()),
        }
    }
}

#[derive(Debug, Serialize)]
enum IiifType {
    #[serde(rename = "oa:Annotation")]
    Annotation,
    #[serde(rename = "sc:Canvas")]
    Canvas,
    Image,
    #[serde(rename = "sc:Manifest")]
    Manifest,
    #[serde(rename = "sc:Sequence")]
    Sequence,
}

#[derive(Debug, Serialize)]
enum Motivation {
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
            context: Uri::new(CONTEXT),
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

pub struct Id {
    pub value: String,
    pub encoded: String,
}

impl Id {
    pub fn new<S: Into<String>>(value: S) -> Id {
        let value = value.into();
        let encoded = value.replace("/", "%2F");
        Id { value, encoded }
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
            context: Uri::new("http://iiif.io/api/image/2/context.json".to_owned()),
            id: iiif_urls.image_service_id(image_id),
            profile: Uri::new("http://iiif.io/api/image/2/level2.json".to_owned()),
            protocol: Uri::new("http://iiiif.io/api/image".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Uri {
    value: String,
}

impl Uri {
    pub fn new<S: Into<String>>(value: S) -> Uri {
        Uri {
            value: value.into(),
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
            context: Uri::new(CONTEXT),
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
        image_metadata: &ImageMetadata,
    ) {
        let index = self.canvases.len();
        let mut canvas = Canvas::new(
            base_urls,
            item_id,
            index,
            label,
            image_metadata.width,
            image_metadata.height,
        );
        let image_resource = ImageResource::new(base_urls, image_id, image_metadata);
        let annotation = Annotation::new(Resource::Image(image_resource), (&canvas.id).clone());
        &canvas.add_image(annotation);
        self.canvases.push(canvas);
    }

    pub fn add_placeholder(&mut self, base_urls: &IiifUrls, item_id: &Id, label: &str) {
        let index = self.canvases.len();
        let fake_id = format!("??? ({})", index);
        self.add_image(
            base_urls,
            item_id,
            &Id::new(fake_id),
            label,
            &ImageMetadata::unknown(),
        )
    }
}

pub struct IiifUrls {
    presentation: String,
    image: String,
}

impl IiifUrls {
    pub fn new(presentation: String, image: String) -> IiifUrls {
        IiifUrls {
            presentation,
            image,
        }
    }

    pub fn canvas_id(&self, item_id: &Id, index: usize) -> Uri {
        Uri::new(format!(
            "{}/{}/canvas/{}",
            self.presentation, item_id.encoded, index
        ))
    }
    pub fn manifest_id(&self, item_id: &Id) -> Uri {
        Uri::new(format!(
            "{}/{}/manifest",
            self.presentation, item_id.encoded
        ))
    }

    pub fn image_id(&self, image_id: &Id, image_metadata: &ImageMetadata) -> Uri {
        Uri::new(format!(
            "{}/{}/full/full/0/default.{}",
            self.image, image_id.encoded, image_metadata.extension
        ))
    }

    pub fn sequence_id(&self, item_id: &Id) -> Uri {
        Uri::new(format!(
            "{}/{}/sequence/normal",
            self.presentation, item_id.encoded
        ))
    }

    pub fn image_service_id(&self, image_id: &Id) -> Uri {
        Uri::new(format!("{}/{}", self.image, image_id.encoded))
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
            context: Uri::new(CONTEXT),
            iiif_type: IiifType::Canvas,
            label: label.to_owned(),
            height,
            width,
            images: Vec::new(),
        }
    }

    pub fn add_image(&mut self, image: Annotation) {
        self.images.push(image);
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
            context: Uri::new(CONTEXT),
            iiif_type: IiifType::Annotation,
            motivation: Motivation::Painting,
            resource,
            on,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Resource {
    Image(ImageResource),
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
    pub fn new(
        iiif_urls: &IiifUrls,
        image_id: &Id,
        image_metadata: &ImageMetadata,
    ) -> ImageResource {
        ImageResource {
            id: iiif_urls.image_id(image_id, image_metadata),
            iiif_type: IiifType::Image,
            format: image_metadata.format.to_owned(),
            service: Service::new_image_service(iiif_urls, image_id),
            width: image_metadata.width,
            height: image_metadata.height,
        }
    }
}
