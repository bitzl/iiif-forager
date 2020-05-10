  
#[macro_use]
extern crate actix_web;

mod iiif;

use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse, Responder};
use std::path::PathBuf;

use crate::iiif::{BaseUrls, Manifest, Metadata, Sequence, ImageFormat};

struct ManifestSource {
    base_path: PathBuf,
    base_urls: BaseUrls
}

impl ManifestSource {
    fn new(base_path: PathBuf, base_urls: BaseUrls) -> ManifestSource {
        ManifestSource{base_path, base_urls}
    }

    fn manifest_for(&self, id: &str) -> Result<Manifest, String> {
        let metadata: Vec<Metadata> = vec![Metadata::key_value("location", id)];
        let description = Some(id.to_owned());

        let source_path = self.base_path.join(id);
        if !source_path.exists() {
            return Err(format!("path {} does not exist", source_path.into_os_string().to_str().unwrap()));
        }
        if !source_path.is_dir()  {
            return Err(format!("path {} is not a directory", source_path.into_os_string().to_str().unwrap()));
        }

        let mut sequence = Sequence::new(&self.base_urls, id, id);

        for entry in std::fs::read_dir(source_path).unwrap() {
            match entry {
                Ok(file) => {
                    let path = file.path();
                    println!("file: {}", path.to_str().unwrap());
                    if !path.is_file() {
                        continue;
                    }

                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let image_meta = get_image_metadata(&path);
                    sequence.add_image(&self.base_urls, &id, &file_name, &file_name, &image_meta.image_format, image_meta.width, image_meta.height);
                },
                Err(e) => {
                    let label = format!("error reading entry: {}", e);
                    sequence.add_image(&self.base_urls, &id, "???", &label, &ImageFormat::Unknown, 0, 0);
                }
            }
        }

        let mut manifest = Manifest::new(&self.base_urls, id, id, metadata, description);
        manifest.add_sequence(sequence);
        Ok(manifest)
    }
}

struct ImageMetadata {
    image_format: ImageFormat,
    height: u64,
    width: u64
}

impl ImageMetadata {
    fn unknown() -> ImageMetadata {
        ImageMetadata{ 
            image_format: ImageFormat::Unknown,
            width: 0,
            height: 0
        }
    }
}

fn get_image_metadata(path: &PathBuf) -> ImageMetadata {
    let extension = path.extension();
    if path.extension().is_none() {
        return ImageMetadata::unknown()
    }
    match extension.unwrap().to_str().unwrap().to_lowercase().as_str() {
        "png" => ImageMetadata{ 
            image_format: ImageFormat::PNG,
            width: 0,
            height: 0
        },
        "jpg" => ImageMetadata{
            image_format: ImageFormat::JPEG,
            width: 0,
            height: 0
        },
        _ => ImageMetadata::unknown()
    }
}



#[get("/{id:.*}/manifest")]
// async fn index(manifest_source: web::Data<ManifestSource>, path: web::Path<String>) -> HttpResponse {
async fn index(manifest_source: web::Data<ManifestSource>, path: web::Path<String>) -> HttpResponse {
    println!("Url-Path: {}", path.to_string());
    match manifest_source.get_ref().manifest_for(&path.to_string()) {
        Ok(manifest) => {
            let data = serde_json::to_string(&manifest).unwrap();
            println!("data = {}", data);
            HttpResponse::Ok().body(data)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(e)
        }
    }
}


fn main()  {
    web().unwrap()
}

#[actix_rt::main]
async fn web() -> std::io::Result<()> {
    let base_urls = BaseUrls::new("http://127.0.0.1:8000/iiif/presentation".to_owned(), "http://127.0.0.1:8000/iiif/image".to_owned());
    let manifest_source = ManifestSource::new(PathBuf::new(), base_urls);
    let manifest_source_ref = web::Data::new(manifest_source);
  
    HttpServer::new(move || {
        App::new()
            .app_data(manifest_source_ref.clone())
            .service(index)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}