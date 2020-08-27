use serde::Deserialize;
use serde_yaml;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Config {
    pub serving: Serving,
    pub urls: Urls,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Serving {
    pub path: PathBuf,
    pub host: String,
    pub port: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Urls {
    #[serde(rename = "path sep")]
    pub path_sep: String,
    #[serde(rename = "image api")]
    pub image_api: String,
    #[serde(rename = "presentation api")]
    pub presentation_api: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
        let f = std::fs::File::open(path.as_ref())?;
        let config: Config = serde_yaml::from_reader(f)?;
        Ok(config)
    }
}

impl Serving {
    pub fn bind(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {

    use crate::config::Config;
    use serde_yaml;

    const FULL_CONFIG: &str = "
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
    fn load_yaml() {
        let config: Config = serde_yaml::from_str(FULL_CONFIG).unwrap();
        assert_eq!(config.serving.host, "localhost");
        assert_eq!(config.serving.port, 7890);
        assert_eq!(config.urls.path_sep, "-");
        assert_eq!(config.urls.image_api, "http://localhost:1234/iiif/image/v2");
        assert_eq!(
            config.urls.presentation_api,
            "http://localhost:1234/iiif/presentation/v2"
        );
    }

    #[test]
    fn load_minimal() {
        let config: Config = serde_yaml::from_str(FULL_CONFIG).unwrap();
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
