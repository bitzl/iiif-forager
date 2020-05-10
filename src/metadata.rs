use std::path::PathBuf;
use std::fs::File;


pub struct ImageMetadata {
    pub format: String,
    pub extension: String,
    pub width: u32,
    pub height: u32
}

impl ImageMetadata {
    pub fn read(path: &PathBuf) -> Result<ImageMetadata, String> {
        if path.extension().is_none() {
            return Err(format!("{} has no file extension", path.to_str().unwrap()))
        }
        match path.extension().unwrap().to_str() {
            Some("png") => ImageMetadata::read_png(path),
            _ => Err(format!("Cannot get file extension for {}", path.to_str().unwrap()))
        }
    }

    pub fn unknown() -> ImageMetadata {
        ImageMetadata {
            format: "unknown".to_owned(),
            extension: "unknown".to_owned(),
            width: 0,
            height: 0
        }
    }

    fn read_png(path: &PathBuf) -> Result<ImageMetadata, String> {
        let decoder = png::Decoder::new(File::open(path).unwrap());
        let (info, _) = decoder.read_info().unwrap();
        Ok(ImageMetadata{
            format: "image/png".to_owned(),
            extension: "png".to_owned(),
            width: info.width,
            height: info.height
        })
    }
}