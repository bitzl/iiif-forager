use serde::Deserialize;
use serde_yaml;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    serving: Serving,
    urls: Urls,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Serving {
    path: String,
    host: String,
    port: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Urls {
    path_sep: String,
    image_api: String,
    presentation_api: String,
}

impl Config {
    fn load<P: AsRef<Path>>(path: P) -> Result<Config, Box<std::error::Error>> {
        let f = std::fs::File::open(path.as_ref())?;
        let config: Config = serde_yaml::from_reader(f)?;
        Ok(config)
    }

    fn bind(&self) -> String {
        format!("http://{}:{}", self.serving.host, self.serving.port)
    }
}
