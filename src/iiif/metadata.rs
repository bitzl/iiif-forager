use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    Many(Vec<String>),
    Multilang(Vec<LocalizedValue>),
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LocalizedValue {
    #[serde(rename = "@value")]
    value: String,
    #[serde(rename = "@language")]
    language: String,
}

impl LocalizedValue {
    pub fn new<S: Into<String>>(value: S, language: S) -> LocalizedValue {
        LocalizedValue {
            value: value.into(),
            language: language.into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Metadata {
    pub label: String,
    pub value: Value,
}

impl Metadata {
    pub fn key_value<S: Into<String>>(label: S, value: S) -> Metadata {
        Metadata {
            label: label.into(),
            value: Value::Single(value.into()),
        }
    }
    pub fn list<S: Into<String>>(label: S, values: Vec<String>) -> Metadata {
        Metadata {
            label: label.into(),
            value: Value::Many(values),
        }
    }
    pub fn localized<S: Into<String>>(label: S, values: Vec<LocalizedValue>) -> Metadata {
        Metadata {
            label: label.into(),
            value: Value::Multilang(values),
        }
    }
}
