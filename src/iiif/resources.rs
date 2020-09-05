use crate::iiif::types::{Id, Uri};
use crate::image::source::Image;
use crate::image::Format;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Service {
    id: Uri,
    profile: Uri,
    protocol: Uri,
}

// TODO: better ContentResource?
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Resource {
    Image(IiifImage),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct IiifImage {
    id: Uri,
    format: String,
    service: ImageService2,
    width: u32,
    height: u32,
}

impl IiifImage {
    pub fn id(image_api: &str, image_id: &Id, format: &Format) -> Uri {
        Uri::new(format!(
            "{}/{}/full/full/0/default.{}",
            image_api,
            image_id.encoded,
            format.extension()
        ))
    }

    pub fn new(image_api: &str, image_id: &Id, image: &Image) -> IiifImage {
        IiifImage {
            id: IiifImage::id(image_api, image_id, &image.format),
            format: image.format.media_type().to_owned(),
            service: ImageService2::new(ImageService2::id(image_api, image_id)),
            width: image.width,
            height: image.height,
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
    pub fn id(image_api: &str, image_id: &Id) -> Uri {
        Uri::new(format!("{}/{}", image_api, image_id.encoded))
    }
    fn new(id: Uri) -> ImageService2 {
        ImageService2 {
            id,
            profile: "level2".to_owned(),
        }
    }
}
