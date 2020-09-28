use crate::iiif::metadata::Metadata;
use serde::Deserialize;
use serde_json;
use serde_yaml;
use std::error::Error;
use std::fs::File;
use std::path::Path;

enum Format {
    JSON,
    YAML,
}

// A context allows to add addditional metadata using a JSON file
// "context.json" in the same directory as the images.
#[derive(Debug, Deserialize)]
pub struct Meta {
    pub description: Option<String>,
    #[serde(default = "Vec::new")]
    pub metadata: Vec<Metadata>,
}

impl Meta {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Meta, Box<dyn Error>> {
        for (filename, format) in vec![
            ("meta.json", Format::JSON),
            ("meta.yml", Format::YAML),
            ("meta.yaml", Format::YAML),
        ] {
            let meta_path = path.as_ref().join(filename);
            if !meta_path.exists() {
                continue;
            }
            let json_file = File::open(meta_path)?;
            let context = match format {
                Format::JSON => serde_json::from_reader(&json_file)?,
                Format::YAML => serde_yaml::from_reader(&json_file)?,
            };
            return Ok(context);
        }
        Ok(Meta::empty())
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Meta {
        match Meta::load(path) {
            Ok(meta) => meta,
            Err(_) => Meta::empty(),
        }
    }

    pub const fn empty() -> Meta {
        Meta {
            description: None,
            metadata: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::iiif::metadata::{LocalizedValue, Metadata};
    use crate::meta::Meta;

    #[test]
    fn load_json() {
        let json = r#"
        {
            "description": "Expected description",
            "metadata": [
                {
                    "label": "size",
                    "value": "53 MB"
                },
                {
                    "label": "colors",
                    "value": ["red", "green", "blue"]
                },
                {
                    "label": "quality",
                    "value": [{"@value": "high", "@language": "en"}]
                }
            ]
        }"#;
        let actual: Meta = serde_json::from_str(json).unwrap();
        assert_eq!(actual.description, Some("Expected description".to_owned()));
        assert_eq!(actual.metadata[0], Metadata::key_value("size", "53 MB"));
        assert_eq!(
            actual.metadata[1],
            Metadata::list(
                "colors",
                vec!["red".to_owned(), "green".to_owned(), "blue".to_owned()]
            )
        );
        assert_eq!(
            actual.metadata[2],
            Metadata::localized("quality", vec![LocalizedValue::new("high", "en")])
        );
    }
}
