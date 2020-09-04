pub mod collections;
pub mod metadata;
pub mod types;

use crate::config::Config;
use crate::context::Context;
use crate::iiif::collections::Collection;
use crate::iiif::metadata::Metadata;
use crate::iiif::types::{Id, Uri};
use crate::image::source::Image;
use crate::image::Format;

use serde::Serialize;
use std::error::Error;
use std::ffi::OsStr;

const PRESENTATION: &str = "http://iiif.io/api/presentation/3/context.json";

#[derive(Debug, Serialize)]
pub enum Motivation {
    #[serde(rename = "painting")]
    Painting,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")] // valid Presentation API v3
pub struct Manifest {
    #[serde(rename = "@context")]
    context: Uri,
    id: Uri,
    label: String,
    metadata: Vec<Metadata>,
    description: Option<String>,
    items: Vec<Canvas>,
}

impl Manifest {
    pub fn id(presentation_api: &str, item_id: &Id) -> Uri {
        Uri::new(format!("{}/{}/manifest", presentation_api, item_id.encoded))
    }

    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        label: &str,
        metadata: Vec<Metadata>,
        description: Option<String>,
    ) -> Manifest {
        Manifest {
            context: Uri::new(PRESENTATION),
            id: Manifest::id(presentation_api, item_id),
            label: label.to_owned(),
            metadata,
            description,
            items: Vec::new(),
        }
    }

    pub fn add_image(
        &mut self,
        image_api: &str,
        presentation_api: &str,
        item_id: &Id,
        image_id: &Id,
        label: &str,
        image: &Image,
    ) {
        let index = self.items.len();
        let mut canvas = Canvas::new(
            presentation_api,
            item_id,
            index,
            label,
            image.width,
            image.height,
        );
        let image_resource = IiifImage::new(image_api, image_id, image);
        let annotation = Annotation::new_painting(
            presentation_api,
            item_id,
            index,
            Resource::Image(image_resource),
            (&canvas.id).clone(),
        );
        let annotation_page =
            AnnotationPage::new(presentation_api, item_id, index, vec![annotation]);
        &canvas.add_item(annotation_page);
        self.items.push(canvas);
    }
}

#[derive(Debug, Serialize)]
pub struct Service {
    id: Uri,
    profile: Uri,
    protocol: Uri,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct Canvas {
    id: Uri,
    label: String,
    height: u32,
    width: u32,
    items: Vec<AnnotationPage>,
}

impl Canvas {
    pub fn id(presentation_api: &str, item_id: &Id, index: usize) -> Uri {
        Uri::new(format!(
            "{}/{}/canvas/{}",
            presentation_api, item_id.encoded, index
        ))
    }
    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        label: &str,
        width: u32,
        height: u32,
    ) -> Canvas {
        Canvas {
            id: Canvas::id(presentation_api, item_id, index),
            label: label.to_owned(),
            height,
            width,
            items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: AnnotationPage) {
        self.items.push(item);
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
struct AnnotationPage {
    id: Uri,
    items: Vec<Annotation>,
}

impl AnnotationPage {
    pub fn id(presentation_api: &str, item_id: &Id, index: usize) -> Uri {
        Uri::new(format!(
            "{}/{}/page/{}",
            presentation_api, item_id.encoded, index
        ))
    }

    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        items: Vec<Annotation>,
    ) -> AnnotationPage {
        AnnotationPage {
            id: AnnotationPage::id(presentation_api, item_id, index),
            items,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct Annotation {
    id: Uri,
    motivation: Motivation,
    body: Resource,
    target: Uri,
}

impl Annotation {
    pub fn id(presentation_api: &str, item_id: &Id, page: usize, suffix: &str) -> Uri {
        Uri::new(format!(
            "{}/{}/annotation/{}-{}",
            presentation_api, item_id.encoded, page, suffix
        ))
    }

    pub fn new(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        resource: Resource,
        target: Uri,
        motivation: Motivation,
    ) -> Annotation {
        let id = match resource {
            Resource::Image(_) => Annotation::id(presentation_api, item_id, index, "image"),
        };
        Annotation {
            id,
            motivation,
            body: resource,
            target,
        }
    }

    pub fn new_painting(
        presentation_api: &str,
        item_id: &Id,
        index: usize,
        resource: Resource,
        target: Uri,
    ) -> Annotation {
        Annotation::new(
            presentation_api,
            item_id,
            index,
            resource,
            target,
            Motivation::Painting,
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Resource {
    Image(IiifImage),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct IiifImage {
    id: Uri,
    format: String,
    service: ImageService2,
    width: u32,
    height: u32,
}

impl IiifImage {
    pub fn id(image_api: &str, image_id: &Id, format: &Format) -> Uri {
        Uri::new(format!(
            "{}/{}/full/full/0/default.{}",
            image_api,
            image_id.encoded,
            format.extension()
        ))
    }

    pub fn new(image_api: &str, image_id: &Id, image: &Image) -> IiifImage {
        IiifImage {
            id: IiifImage::id(image_api, image_id, &image.format),
            format: image.format.media_type().to_owned(),
            service: ImageService2::new(ImageService2::id(image_api, image_id)),
            width: image.width,
            height: image.height,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct ImageService2 {
    id: Uri,
    profile: String,
}

impl ImageService2 {
    pub fn id(image_api: &str, image_id: &Id) -> Uri {
        Uri::new(format!("{}/{}", image_api, image_id.encoded))
    }
    fn new(id: Uri) -> ImageService2 {
        ImageService2 {
            id,
            profile: "level2".to_owned(),
        }
    }
}

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

        let mut collection = Collection::new();

        let mut directory_paths: Vec<_> = std::fs::read_dir(source_path)?
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        directory_paths.sort();

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
