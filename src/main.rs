#[macro_use]
extern crate actix_web;

mod config;
mod context;
mod http_api;
mod iiif;
mod image;

use clap;

use crate::iiif::IiifGenerator;
use crate::image::source::ImageSource;

use crate::config::Config;
use std::path::Path;

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
    let image_source = ImageSource::new(config.clone());
    let manifest_generator = IiifGenerator::new(config);
    http_api::start(manifest_generator, image_source, bind).unwrap()
}
