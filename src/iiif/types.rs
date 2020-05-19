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

    pub fn image_id(&self, image_id: &Id, format: &Format) -> Uri {
        Uri::new(format!(
            "{}/{}/full/full/0/default.{}",
            self.image,
            image_id.encoded,
            format.extension()
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
