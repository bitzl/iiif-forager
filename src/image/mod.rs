pub mod metadata;
mod png;
pub mod source;

#[derive(Debug, PartialEq)]
pub enum Format {
    PNG,
    JPEG,
    TIFF,
}

impl Format {
    pub fn extension(&self) -> &str {
        match self {
            Format::PNG => "png",
            Format::JPEG => "jpg",
            Format::TIFF => "tif",
        }
    }

    pub fn media_type(&self) -> &str {
        match self {
            Format::PNG => "image/png",
            Format::JPEG => "image/jpeg",
            Format::TIFF => "image/tiff",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.media_type())
    }
}

#[derive(Debug, PartialEq)]
pub enum Label {
    KV(String, String),
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::KV(key, value) => write!(f, "{}: {}", key, value),
        }
    }
}
