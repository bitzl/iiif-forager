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
    #[serde(rename = "path sep")]
    path_sep: String,
    #[serde(rename = "image api")]
    image_api: String,
    #[serde(rename = "presentation api")]
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

#[cfg(test)]
mod tests {

    use crate::config::Config;
    use serde_yaml;

    const CONFIG: &str = "
    # Which directory shall be served where (host and port)?
    serving:
        path: samples
        host: localhost
        port: 7890

    # The urls part is important for the public facing user interaction
    # and will end up in the generated JSON.
    urls:
        path sep: '-'
        image api: http://localhost:1234/iiif/image/v2
        presentation api: http://localhost:1234/iiif/presentation/v2
    ";

    #[test]
    fn load_path() {
        let config: Config = serde_yaml::from_str(CONFIG).unwrap();
        assert_eq!(config.serving.path, "samples");
    }
    #[test]
    fn load_yaml() {
        let config: Config = serde_yaml::from_str(CONFIG).unwrap();
        assert_eq!(config.serving.host, "localhost");
        assert_eq!(config.serving.port, 7890);
        assert_eq!(config.urls.path_sep, "-");
        assert_eq!(config.urls.image_api, "http://localhost:1234/iiif/image/v2");
        assert_eq!(
            config.urls.presentation_api,
            "http://localhost:1234/iiif/presentation/v2"
        );
    }
}
