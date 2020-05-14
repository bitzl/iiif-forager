use crate::iiif::Metadata;
use serde::Deserialize;
use serde_json;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Context {
    description: String,
    metadata: Vec<Metadata>,
}

impl Context {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Context, Box<Error>> {
        let json_file = File::open(path).expect("file not found");
        serde_json::from_reader(json_file)?
    }
}
