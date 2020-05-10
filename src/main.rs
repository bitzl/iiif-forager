#[macro_use]
extern crate actix_web;

mod iiif;
mod metadata;

use actix_web::{web, App, HttpResponse, HttpServer};
use std::path::PathBuf;

use crate::iiif::{BaseUrls, Manifest, Metadata, Sequence};
use crate::metadata::{ImageMetadata, MetadataError};

struct ManifestSource {
    base_path: PathBuf,
    base_urls: BaseUrls,
}

impl ManifestSource {
    fn new(base_path: PathBuf, base_urls: BaseUrls) -> ManifestSource {
        ManifestSource {
            base_path,
            base_urls,
        }
    }

    fn manifest_for(&self, id: &str) -> Result<Manifest, String> {
        let metadata: Vec<Metadata> = vec![Metadata::key_value("location", id)];
        let description = Some(id.to_owned());

        let source_path = self.base_path.join(id);
        if !source_path.exists() {
            return Err(format!(
                "path {} does not exist",
                source_path.into_os_string().to_str().unwrap()
            ));
        }
        if !source_path.is_dir() {
            return Err(format!(
                "path {} is not a directory",
                source_path.into_os_string().to_str().unwrap()
            ));
        }

        let mut sequence = Sequence::new(&self.base_urls, id, id);

        for entry in std::fs::read_dir(source_path).unwrap() {
            let file = match entry {
                Ok(file) => file,
                Err(e) => {
                    let label = format!("error reading entry: {}", e);
                    sequence.add_image(
                        &self.base_urls,
                        &id,
                        "???",
                        &label,
                        &ImageMetadata::unknown(),
                    );
                    continue;
                }
            };
            
            // got file, get metadata
            let path = file.path();
            println!("file: {}", path.to_str().unwrap());
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().unwrap().to_str().unwrap();
            match ImageMetadata::read(&path) {
                Ok(image_metadata) => sequence.add_image(
                    &self.base_urls,
                    &id,
                    &file_name,
                    &file_name,
                    &image_metadata,
                ),
                Err(MetadataError::IoError(e)) => {
                    // TODO skip errors for non-image files, but show broken image files as broken
                    println!("Error: {}", e);
                    continue;
                }
                Err(MetadataError::UnsupportedFiletype(_)) => {
                    // Probably not an image file, skip
                    continue;
                }
            }
        }

        let mut manifest = Manifest::new(&self.base_urls, id, id, metadata, description);
        manifest.add_sequence(sequence);
        Ok(manifest)
    }
}

#[get("/{id:.*}/manifest")]
// async fn index(manifest_source: web::Data<ManifestSource>, path: web::Path<String>) -> HttpResponse {
async fn index(
    manifest_source: web::Data<ManifestSource>,
    path: web::Path<String>,
) -> HttpResponse {
    println!("Url-Path: {}", path.to_string());
    match manifest_source.get_ref().manifest_for(&path.to_string()) {
        Ok(manifest) => {
            let data = serde_json::to_string(&manifest).unwrap();
            println!("data = {}", data);
            HttpResponse::Ok().body(data)
        }
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

fn main() {
    web().unwrap()
}

#[actix_rt::main]
async fn web() -> std::io::Result<()> {
    let base_urls = BaseUrls::new(
        "http://127.0.0.1:8000/iiif/presentation".to_owned(),
        "http://127.0.0.1:8000/iiif/image".to_owned(),
    );
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
