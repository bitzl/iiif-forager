use crate::iiif::metadata::Metadata;
use serde::Deserialize;
use serde_json;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Context {
    pub description: Option<String>,
    #[serde(default = "Vec::new")]
    pub metadata: Vec<Metadata>,
}

impl Context {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Context, Box<dyn Error>> {
        for extension in vec!["json"] {
            let filename = format!("context.{}", extension);
            let context_path = path.as_ref().join(filename);
            if context_path.exists() {
                let json_file = File::open(context_path)?;
                let context = match extension {
                    "json" => serde_json::from_reader(&json_file)?,
                    _ => Context::empty(), // not possible
                };
                return Ok(context);
            }
        }
        Ok(Context::empty())
    }
    pub const fn empty() -> Context {
        Context {
            description: None,
            metadata: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::context::Context;
    use crate::iiif::metadata::{LocalizedValue, Metadata};

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
        let actual: Context = serde_json::from_str(json).unwrap();
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
