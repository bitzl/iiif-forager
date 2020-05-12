#[macro_use]
extern crate actix_web;

mod iiif;
mod metadata;

use actix_web::{web, App, HttpResponse, HttpServer};
use clap;
use std::path::{Path, PathBuf};

use crate::iiif::{BaseUrls, Manifest, Metadata, Sequence};
use crate::metadata::{ImageMetadata, MetadataError};

struct ManifestSource {
    base_path: PathBuf,
    base_urls: BaseUrls,
    path_sep: String
}

impl ManifestSource {
    fn new(base_path: PathBuf, base_urls: BaseUrls, path_sep: String) -> ManifestSource {
        ManifestSource {
            base_path,
            base_urls,
            path_sep
        }
    }

    fn path_for_id(&self, id: &str) -> PathBuf {
        let os_sep = std::path::MAIN_SEPARATOR.to_string();
        let path = id.replace(&self.path_sep, os_sep.as_str());
        self.base_path.join(path)
    }

    fn manifest_for(&self, item_id: &str) -> Result<Manifest, String> {
        let source_path = self.path_for_id(item_id);
        if !source_path.exists() {
            return Err(format!(
                "path {} does not exist",
                source_path.to_str().unwrap()
            ));
        }
        if !source_path.is_dir() {
            return Err(format!(
                "path {} is not a directory",
                source_path.to_str().unwrap()
            ));
        }

        let mut sequence = Sequence::new(&self.base_urls, item_id);
        for entry in std::fs::read_dir(&source_path).unwrap() {
            let path = match entry {
                Ok(file) => file.path(),
                Err(e) => {
                    println!(
                        "Cannot read entry in {}: {}",
                        &source_path.to_str().unwrap(),
                        e
                    );
                    continue;
                }
            };

            // got file, get metadata
            if !ImageMetadata::is_supported(&path) {
                continue;
            }

            let file_name = path.file_name().unwrap().to_str().unwrap();
            match ImageMetadata::read(&path) {
                Ok(metadata) => {
                    let image_id = format!("{}{}{}", &item_id, self.path_sep, &file_name);
                    sequence.add_image(&self.base_urls, &item_id, &image_id, &file_name, &metadata)
                }
                Err(MetadataError::IoError(e)) => {
                    println!("Error: {}", e);
                    sequence.add_placeholder(&self.base_urls, &item_id, &format!("Error: {}", e));
                }
                Err(MetadataError::UnsupportedFiletype(_)) => {
                    // Should never happen
                }
            }
        }

        let metadata: Vec<Metadata> = vec![Metadata::key_value("location", item_id)];
        let description = Some(item_id.to_owned());
        let mut manifest = Manifest::new(&self.base_urls, item_id, item_id, metadata, description);
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
            HttpResponse::Ok().body(data)
        }
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

fn main() {
    let matches = clap::App::new("IIIF Presenter")
        .version("0.0.1")
        .author("Marcus Bitzl")
        .about("Serve manifests for images in directories")
        .arg(
            clap::Arg::with_name("SOURCE")
                .help("Directory containing the image directories")
                .required(true)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("bind")
                .help("Bind address and port")
                .long("--bind")
                .short("-b")
                .default_value("localhost:8989")
                .required(true)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("presentation_base_url")
                .help("Base Url for all IIIF Presentation API urls")
                .long("--presentation-api")
                .short("-p")
                .required(false)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("image_base_url")
                .help("Base Url for all IIIF Image API urls")
                .long("--image-api")
                .short("-i")
                .required(true)
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("url_path_sep")
                .help("Separator for paths when turning these into ids")
                .long("--url-path-sep")
                .short("-u")
                .default_value("-")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let source = Path::new(matches.value_of("SOURCE").unwrap());
    let bind = matches.value_of("bind").unwrap();
    let presentation_base_url = match matches.value_of("presentation_base_url") {
        Some(url) => url.to_owned(),
        None => format!("http://{}", bind)
    };
    let base_urls = BaseUrls::new(
        presentation_base_url,
        matches.value_of("image_base_url").unwrap().to_owned()
    );
    let path_sep = matches.value_of("url_path_sep").unwrap().to_owned();

    let manifest_source = ManifestSource::new(source.to_path_buf(), base_urls, path_sep);
    web(manifest_source, bind.to_owned()).unwrap()
}

#[actix_rt::main]
async fn web(manifest_source: ManifestSource, bind: String) -> std::io::Result<()> {
    let manifest_source_ref = web::Data::new(manifest_source);
    HttpServer::new(move || {
        App::new()
            .app_data(manifest_source_ref.clone())
            .service(index)
    })
    .bind(bind)?
    .run()
    .await
}
