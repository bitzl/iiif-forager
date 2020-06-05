use serde::Serialize;

use crate::image::Format;

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

pub struct IiifUrls {
    presentation: String,
    image: String,
}

impl IiifUrls {
    pub fn new<S: Into<String>>(presentation: S, image: S) -> IiifUrls {
        IiifUrls {
            presentation: presentation.into(),
            image: image.into(),
        }
    }

    pub fn annotation_page_id(&self, item_id: &Id, index: usize) -> Uri {
        Uri::new(format!(
            "{}/{}/page/{}",
            self.presentation, item_id.encoded, index
        ))
    }

    pub fn annotation_id(&self, item_id: &Id, page: usize, suffix: &str) -> Uri {
        Uri::new(format!(
            "{}/{}/annotation/{}-{}",
            self.presentation, item_id.encoded, page, suffix
        ))
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

    pub fn image_id(&self, image_id: &Id, format: &Format) -> Uri {
        Uri::new(format!(
            "{}/{}/full/full/0/default.{}",
            self.image,
            image_id.encoded,
            format.extension()
        ))
    }

    pub fn image_service_id(&self, image_id: &Id) -> Uri {
        Uri::new(format!("{}/{}", self.image, image_id.encoded))
    }
}
