use std::fs::File;
use std::path::PathBuf;

pub enum MetadataError {
    UnsupportedFiletype(String),
    IoError(String),
}

pub struct ImageMetadata {
    pub format: String,
    pub extension: String,
    pub width: u32,
    pub height: u32,
}

impl ImageMetadata {
    pub fn read(path: &PathBuf) -> Result<ImageMetadata, MetadataError> {
        if path.extension().is_none() {
            return Err(MetadataError::UnsupportedFiletype(format!(
                "{} has no file extension",
                path.to_str().unwrap()
            )));
        }
        match path.extension().unwrap().to_str() {
            Some("png") => ImageMetadata::read_png(path),
            _ => Err(MetadataError::UnsupportedFiletype(format!(
                "Cannot get file extension for {}",
                path.to_str().unwrap()
            ))),
        }
    }

    /// Returns true if `path` points to a file and reading metadata
    /// is supported (based on file extension)
    ///
    /// # Arguments
    /// * `path` - A PathBuf reference to check
    pub fn is_supported(path: &PathBuf) -> bool {
        if !path.is_file() {
            return false;
        }
        match path.extension() {
            Some(extension) => extension == "png",
            None => false,
        }
    }

    pub fn unknown() -> ImageMetadata {
        ImageMetadata {
            format: "unknown".to_owned(),
            extension: "unknown".to_owned(),
            width: 0,
            height: 0,
        }
    }

    fn read_png(path: &PathBuf) -> Result<ImageMetadata, MetadataError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(MetadataError::IoError(format!(
                    "Cannot open file {}: {}",
                    path.to_str().unwrap(),
                    e
                )))
            }
        };
        let decoder = png::Decoder::new(file);
        let info = match decoder.read_info() {
            Ok((info, _)) => info,
            Err(e) => {
                return Err(MetadataError::IoError(format!(
                    "Cannot read {}: {}",
                    path.to_str().unwrap(),
                    e
                )))
            }
        };
        Ok(ImageMetadata {
            format: "image/png".to_owned(),
            extension: "png".to_owned(),
            width: info.width,
            height: info.height,
        })
    }
}
