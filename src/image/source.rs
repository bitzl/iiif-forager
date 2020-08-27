use crate::image::Format;
use crate::image::Label;

use std::ffi::OsStr;
use std::path::PathBuf;

use crate::image::png::{Chunk, PNG};

pub struct Image {
    pub format: Format,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub labels: Vec<Label>,
}

pub struct ImageSource {
    path: PathBuf,
}

impl ImageSource {
    pub fn load(&self, sub_path: &str) -> Vec<Image> {
        let source_path = self.path.join(sub_path);
        let mut dir_entries: Vec<_> = std::fs::read_dir(&source_path)
            .unwrap()
            .map(|p| p.unwrap())
            .collect();
        dir_entries.sort_by_key(|dir_entry| dir_entry.path());

        for entry in dir_entries {
            let path = entry.path();
        }

        vec![Image {
            format: Format::PNG,
            name: "abc".to_owned(),
            width: 128,
            height: 128,
            labels: Vec::new(),
        }]
    }
}

impl Image {
    pub fn for_file(path: &PathBuf) -> Option<Image> {
        if path.extension().is_none() {
            return None; // Skip if it's not an image and has no extension
        }

        let name: String = match path.file_name() {
            Some(n) => n.to_str().unwrap().to_owned(),
            None => String::new(),
        };

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

                Some(Image {
                    name,
                    format: Format::PNG,
                    width: png.width,
                    height: png.height,
                    labels,
                })
            }
            Some("jpg") | Some("jpeg") => {
                let dimensions = match imagesize::size(path) {
                    Ok(dim) => dim,
                    Err(_) => return None,
                };

                Some(Image {
                    name,
                    format: Format::JPEG,
                    width: dimensions.width as u32,
                    height: dimensions.height as u32,
                    labels: Vec::new(),
                })
            }
            Some("tif") | Some("tiff") => {
                let dimensions = match imagesize::size(path) {
                    Ok(dim) => dim,
                    Err(_) => return None,
                };
                Some(Image {
                    name,
                    format: Format::TIFF,
                    width: dimensions.width as u32,
                    height: dimensions.height as u32,
                    labels: Vec::new(),
                })
            }
            _ => None,
        }
    }
}
