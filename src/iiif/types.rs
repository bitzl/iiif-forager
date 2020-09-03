use serde::Serialize;

pub struct Id {
    pub value: String,
    pub encoded: String,
}

impl Id {
    pub fn new<S: Into<String>>(value: S) -> Id {
        let value = value.into();
        let encoded = value.replace("/", "%2F");
        Id { value, encoded }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Uri {
    value: String,
}

impl Uri {
    pub fn new<S: Into<String>>(value: S) -> Uri {
        Uri {
            value: value.into(),
        }
    }
}
