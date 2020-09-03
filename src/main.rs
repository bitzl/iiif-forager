#[macro_use]
extern crate actix_web;

mod config;
mod context;
mod iiif;
mod image;

use actix_web::{web, App, HttpResponse, HttpServer};
use clap;
use std::path::Path;

use crate::config::Config;
use crate::context::Context;
use crate::iiif::types::Id;
use crate::iiif::Manifest;
use crate::image::source::Image;
use crate::image::source::ImageSource;

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

    if !config.serving.path.exists() {
        panic!("path {} does not exist", config.serving.path.display());
    }
    if !config.serving.path.is_dir() {
        panic!("path {} is not a directory", config.serving.path.display());
    }

    let bind = config.serving.bind();
    let image_source = ImageSource::new(&config.serving.path);
    let manifest_generator = ManifestGenerator::new(config);
    web(manifest_generator, image_source, bind).unwrap()
}

#[actix_rt::main]
async fn web(
    manifest_generator: ManifestGenerator,
    image_source: ImageSource,
    bind: String,
) -> std::io::Result<()> {
    println!("Starting iiif-presenter on http://{}", bind);
    let manifest_generator_ref = web::Data::new(manifest_generator);
    let image_source_ref = web::Data::new(image_source);
    HttpServer::new(move || {
        App::new()
            .app_data(manifest_generator_ref.clone())
            .app_data(image_source_ref.clone())
            .service(index)
    })
    .bind(bind)?
    .run()
    .await
}

struct ManifestGenerator {
    config: Config,
}

impl ManifestGenerator {
    pub fn new(config: Config) -> ManifestGenerator {
        ManifestGenerator { config }
    }
    pub fn manifest_for(&self, id: &str, images: Vec<Image>) -> Result<Manifest, String> {
        let os_sep = std::path::MAIN_SEPARATOR.to_string();
        let path = id.replace(&self.config.urls.path_sep, os_sep.as_str());
        let source_path = self.config.serving.path.join(path);

        let item_id = Id::new(id.replace("/", &self.config.urls.path_sep));
        let context = Context::load_or_default(&source_path);
        let mut manifest = Manifest::new(
            &self.config.urls.presentation_api,
            &item_id,
            &id,
            context.metadata,
            context.description,
        );
        for image in images {
            let image_id = Id::new(
                format!(
                    "{}{}{}",
                    item_id.value, &self.config.urls.path_sep, image.name
                )
                .as_str(),
            );
            let urls = &self.config.urls;
            manifest.add_image(
                &urls.image_api,
                &urls.presentation_api,
                &item_id,
                &image_id,
                &image.name,
                &image,
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
    let images = match image_source.load(&id) {
        Some(images) => images,
        None => return HttpResponse::NotFound().body(id),
    };
    match manifest_generator.get_ref().manifest_for(&id, images) {
        Ok(manifest) => HttpResponse::Ok().json(manifest),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}
