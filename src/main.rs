#[macro_use]
extern crate actix_web;

mod config;
mod context;
mod iiif;
mod image;

use actix_web::{web, App, HttpResponse, HttpServer};
use clap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::context::Context;
use crate::iiif::metadata::Metadata;
use crate::iiif::types::{Id, IiifUrls};
use crate::iiif::Manifest;
use crate::image::ImageInfo;
use crate::image::source::ImageSource;
use crate::image::source::Image;


struct ManifestSource {
    base_path: PathBuf,
    base_urls: IiifUrls,
    path_sep: String,
}

impl ManifestSource {
    fn new(base_path: PathBuf, base_urls: IiifUrls, path_sep: String) -> ManifestSource {
        ManifestSource {
            base_path,
            base_urls,
            path_sep,
        }
    }

    fn path_for_id(&self, id: &Id) -> PathBuf {
        let os_sep = std::path::MAIN_SEPARATOR.to_string();
        let path = id.value.replace(&self.path_sep, os_sep.as_str());
        self.base_path.join(path)
    }

    fn manifest_for(&self, item_id: &Id) -> Result<Manifest, String> {
        let source_path = self.path_for_id(item_id);
        if !source_path.exists() {
            return Err(format!("path {} does not exist", source_path.display()));
        }
        if !source_path.is_dir() {
            return Err(format!("path {} is not a directory", source_path.display()));
        }

        let context = match Context::load(&source_path) {
            Ok(value) => value,
            Err(e) => {
                println!("Could not load context file: {}", e);
                Context::empty()
            }
        };

        let description = Option::from(context.description.unwrap_or(item_id.value.clone()));
        let mut metadata = context.metadata;
        metadata.push(Metadata::key_value("location", &item_id.value));

        let mut manifest = Manifest::new(
            &self.base_urls,
            item_id,
            item_id.value.as_str(),
            metadata,
            description,
        );

        let mut dir_entries: Vec<_> = std::fs::read_dir(&source_path)
            .unwrap()
            .map(|p| p.unwrap())
            .collect();
        dir_entries.sort_by_key(|dir_entry| dir_entry.path());
        // for entry in dir_entries {
        //     let path = entry.path();
        //     match ImageInfo::for_file(&path) {
        //         Some(image_info) => {
        //             let file_name = match path.file_name().and_then(OsStr::to_str) {
        //                 Some(file_name) => file_name,
        //                 None => continue, // should not happen, but if it does there is nothing we can do
        //             };
        //             let image_id = Id::new(
        //                 format!("{}{}{}", item_id.value, self.path_sep, &file_name).as_str(),
        //             );
        //             manifest.add_image(
        //                 &self.base_urls,
        //                 &item_id,
        //                 &image_id,
        //                 &file_name,
        //                 &image_info,
        //             )
        //         }
        //         None => {
        //             // cant't make sense of file, skipping
        //         }
        //     }
        // }
        Ok(manifest)
    }
}


fn main() {
    let matches = clap::App::new("IIIF Forager")
        .version("0.0.1")
        .author("Marcus Bitzl")
        .about("Serve manifests for images in directories")
        .arg(
            clap::Arg::with_name("CONFIG")
                .help("Path to configuration file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let config_path = Path::new(matches.value_of("CONFIG").unwrap());
    let config = match Config::load(config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Could not load config, exiting: {}", e);
            return;
        }
    };

    let source = PathBuf::from(&config.serving.path);
    let base_urls = IiifUrls::new(&config.urls.presentation_api, &config.urls.image_api);

    let manifest_source = ManifestSource::new(source, base_urls, config.urls.path_sep.clone());
    web(manifest_source, config.serving.bind()).unwrap()
}

#[actix_rt::main]
async fn web(manifest_source: ManifestSource, bind: String) -> std::io::Result<()> {
    println!("Starting iiif-presenter on http://{}", bind);
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

struct ManifestGenerator {
    config: Config,
    iiif_urls: IiifUrls,
}

impl ManifestGenerator {
    pub fn manifest_for(&self, id: &str, images: Vec<Image>) -> Result<Manifest, String> {
        let source_path = Path::new("something");
        let item_id = Id::new(id.replace("/", &self.config.urls.path_sep));
        let context = Context::load_or_default(&source_path);
        let mut manifest = Manifest::new(
            &self.iiif_urls,
            &item_id,
            &id,
            context.metadata,
            context.description);
        for image in images {
            let image_id = Id::new(
                format!("{}{}{}", item_id.value, &self.config.urls.path_sep, image.name).as_str(),
            );
            manifest.add_image(
                &self.iiif_urls,
                &item_id,
                &image_id,
                &image.name,
                &image
            )
        
        }
        Ok(manifest)
    }
}

#[get("/{id:.*}/manifest")]
async fn index(
    image_source: web::Data<ImageSource>,
    manifest_generator: web::Data<ManifestGenerator>,
    path: web::Path<String>,
) -> HttpResponse {
    println!("Url-Path: {}", path.to_string());
    let id = path.to_string();
    let images = image_source.load(&id);
    match manifest_generator.get_ref().manifest_for(&id, images) {
        Ok(manifest) => HttpResponse::Ok().json(manifest),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}