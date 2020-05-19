pub mod metadata;
mod png;

use std::ffi::OsStr;
use std::path::PathBuf;

use png::{Chunk, PNG};

#[derive(Debug, PartialEq)]
pub struct ImageInfo {
    pub format: Format,
    pub width: u32,
    pub height: u32,
    pub labels: Vec<Label>,
}

impl ImageInfo {
    pub fn for_file(path: &PathBuf) -> Option<ImageInfo> {
        if path.extension().is_none() {
            return None;
        }

        match path.extension().and_then(OsStr::to_str) {
            Some("png") => {
                let png = match PNG::load(path) {
                    Ok(value) => value,
                    Err(_) => return None,
                };

                let labels: Vec<Label> = png
                    .chunks
                    .into_iter()
                    .filter_map(|chunk| match chunk {
                        Chunk::Text(key, value, _crc) => Some(Label::KV(key, value)),
                        Chunk::InternationalText(text, _crc) => {
                            Some(Label::KV(text.keyword, text.text))
                        }
                        _ => None,
                    })
                    .collect();

                Some(ImageInfo {
                    format: Format::PNG,
                    width: png.width,
                    height: png.height,
                    labels,
                })
            }
            _ => None,
        }
    }
}

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
