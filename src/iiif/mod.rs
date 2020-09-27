pub mod annotations;
pub mod collections;
pub mod manifests;
pub mod metadata;
pub mod resources;
pub mod types;

use crate::config::Config;
use crate::context::Context;
use crate::iiif::collections::Collection;
use crate::iiif::manifests::Manifest;
use crate::iiif::types::Id;
use crate::image::source::Image;

use std::error::Error;
use std::ffi::OsStr;

pub struct IiifGenerator {
    config: Config,
}

impl IiifGenerator {
    pub fn new(config: Config) -> IiifGenerator {
        IiifGenerator { config }
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

    pub fn collection_for(&self, id: &str) -> Result<Collection, Box<dyn Error>> {
        let os_sep = std::path::MAIN_SEPARATOR.to_string();
        let path = id.replace(&self.config.urls.path_sep, os_sep.as_str());
        let source_path = self.config.serving.path.join(path);

        let mut directory_paths: Vec<_> = std::fs::read_dir(source_path)?
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        directory_paths.sort();
        let mut collection = Collection::new();
        for path in directory_paths {
            let name = path.file_name().and_then(OsStr::to_str);
            if name.is_none() {
                continue;
            }
            let item_id = Id::new(format!(
                "{}{}{}",
                id,
                &self.config.urls.path_sep,
                name.unwrap()
            ));
            let manifest_id = Manifest::id(&self.config.urls.presentation_api, &item_id);
            collection.add_manifest(manifest_id);
        }
        Ok(collection)
    }
}
