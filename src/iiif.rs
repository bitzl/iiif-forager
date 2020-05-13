use crate::metadata::ImageMetadata;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;

use url::Url;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    // Many(Vec<String>),
    // Multilang(Vec<LocalizedValue>),
}

#[derive(Debug, Serialize)]
pub struct LocalizedValue {
    value: String,
    language: String,
}

#[derive(Debug, Serialize)]
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
// #[serde(serialize_with="serde::with_skip_serializing_none")]
pub struct Manifest {
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@id")]
    id: Uri,
    #[serde(rename = "@type")]
    iiif_type: String,
    label: String,
    metadata: Vec<Metadata>,
    description: Option<String>,
    // thumbnail: Image,

    // see_also: Vec<Uri>,
    sequences: Vec<Sequence>,
}

impl Manifest {
    pub fn new(
        base_urls: &BaseUrls,
        item_id: &Id,
        label: &str,
        metadata: Vec<Metadata>,
        description: Option<String>,
        // thumbnail: Image,
        // see_also: Repeated<Uri>,
    ) -> Manifest {
        let id = Uri::new(format!("{}/{}/manifest", base_urls.presentation, item_id.encoded));
        let context = Uri::new("http://iiif.io/api/presentation/2/context.json".to_owned());
        let sequences: Vec<Sequence> = Vec::new();
        Manifest {
            context: context,
            id: id,
            iiif_type: "sc:Manifest".to_owned(),
            label: label.to_owned(),
            metadata: metadata,
            description: description,
            // thumbnail: thumbnail,
            // see_also: see_also,
            sequences: sequences,
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
    pub fn new(value: &str) -> Id {
        let encoded = value.replace("/", "%2F");
        let value = value.to_owned();
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
    fn new_image_service(base_urls: &BaseUrls, image_id: &Id) -> Service {
        let context = Uri::new("http://iiif.io/api/image/2/context.json".to_owned());
        let id = Uri::new(format!("{}/{}", base_urls.image, image_id.encoded));
        let profile = Uri::new("http://iiif.io/api/image/2/level2.json".to_owned());
        let protocol = Uri::new("http://iiiif.io/api/image".to_owned());
        Service {
            context,
            id,
            profile,
            protocol,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Uri {
    value: String,
}

impl Uri {
    pub fn new(value: String) -> Uri {
        Uri {
            value: value.to_owned(),
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
    iiif_type: String,
    canvases: Vec<Canvas>,
}

impl Sequence {
    pub fn new(base_urls: &BaseUrls, item_id: &Id) -> Sequence {
        let id = Uri::new(format!(
            "{}/{}/sequence/normal",
            base_urls.presentation, item_id.encoded
        ));
        let context = Uri::new("http://iiif.io/api/presentation/2/context.json".to_owned());
        let iiif_type = "sc:Sequence".to_owned();
        let canvases: Vec<Canvas> = Vec::new();
        Sequence {
            context,
            id,
            iiif_type,
            canvases,
        }
    }

    pub fn add_image(
        &mut self,
        base_urls: &BaseUrls,
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

    pub fn add_placeholder(&mut self, base_urls: &BaseUrls, item_id: &Id, label: &str) {
        let index = self.canvases.len();
        let fake_id = format!("??? ({})", index);
        self.add_image(
            base_urls,
            item_id,
            &Id::new(fake_id.as_str()),
            label,
            &ImageMetadata::unknown(),
        )
    }
}

pub struct BaseUrls {
    presentation: String,
    image: String,
}

impl BaseUrls {
    pub fn new(presentation: String, image: String) -> BaseUrls {
        BaseUrls {
            presentation,
            image,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Canvas {
    #[serde(rename = "@id")]
    id: Uri,
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@type")]
    iiif_type: String,
    label: String,
    height: u32,
    width: u32,
    images: Vec<Annotation>,
}

impl Canvas {
    pub fn new(
        base_urls: &BaseUrls,
        item_id: &Id,
        index: usize,
        label: &str,
        width: u32,
        height: u32,
    ) -> Canvas {
        let id = Uri::new(format!(
            "{}/{}/canvas/{}",
            base_urls.presentation, item_id.encoded, index
        ));
        let context = Uri::new("http://iiif.io/api/presentation/2/context.json".to_owned());
        let iiif_type = "sc:Canvas".to_owned();
        let images: Vec<Annotation> = Vec::new();
        let label = label.to_owned();
        Canvas {
            id,
            context,
            iiif_type,
            label,
            height,
            width,
            images,
        }
    }

    pub fn add_image(&mut self, image: Annotation) {
        self.images.push(image);
    }
}

#[derive(Debug, Serialize)]
pub struct Thumbnail {
    id: Uri,
    iiif_type: String,
    height: u32,
    width: u32,
}

#[derive(Debug, Serialize)]
pub struct Annotation {
    #[serde(rename = "@context")]
    context: Uri,
    #[serde(rename = "@type")]
    iiif_type: String,
    motivation: String,
    resource: Resource,
    on: Uri,
}

impl Annotation {
    pub fn new(resource: Resource, on: Uri) -> Annotation {
        let context = Uri::new("http://iiif.io/api/presentation/2/context.json".to_owned());
        let iiif_type = "oa:Annotation".to_owned();
        let motivation = "sc:painting".to_owned();
        Annotation {
            context,
            iiif_type,
            motivation,
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
    iiif_type: String,
    format: String,
    service: Service,
    width: u32,
    height: u32,
}

impl ImageResource {
    pub fn new(
        base_urls: &BaseUrls,
        image_id: &Id,
        image_metadata: &ImageMetadata,
    ) -> ImageResource {
        let id = Uri::new(format!(
            "{}/{}/full/full/0/default.{}",
            base_urls.image, image_id.value.replace("/", "%2F"), image_metadata.extension
        ));
        let iiif_type = "dctypes:Image".to_owned();
        let service = Service::new_image_service(base_urls, image_id);
        let format = image_metadata.format.to_owned();
        let width = image_metadata.width;
        let height = image_metadata.height;
        ImageResource {
            id,
            iiif_type,
            format,
            service,
            width,
            height,
        }
    }
}
