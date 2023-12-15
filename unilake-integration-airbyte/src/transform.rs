// TODO: implementation of: https://github.com/transferwise/pipelinewise-transform-field
// Functionalities
// 1. Transform to NULL
// 2. Transform to static value
// 3. Hash value
// 4. Regex replace

use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

//{
//  "tap_stream_name": "<stream ID>",
//  "field_id": "<Name of the field to transform in the record>",
//  "type": "<Transformation type>",
//  "when": [
//    {"column": "string_col_1", "equals": "some value"},
//    {"column": "string_col_2", "regex_match": ".*PII.*"},
//    {"column": "numeric_col_1", "equals": 33},
//    {"column": "json_column", "field_path": "metadata/comment", "regex_match": "sensitive"}
//  ]
//}
struct Transformer {
    transformations: Vec<TransformItem>,
}

#[derive(Deserialize, Debug)]
struct TransformItem {
    stream_name: String,
    field_id: String,
    #[serde(alias = "type")]
    t: String,
    when: Vec<TransformWhen>,
}

#[derive(Deserialize, Debug)]
struct TransformWhen {
    column: String,
    equals: Option<serde_json::Value>,
    regex_match: Option<String>,
    field_path: Option<String>,
}

impl Transformer {
    pub fn from_config(config: &serde_json::Value) -> Option<Self> {
        // TODO: transform input config to self struct (input will contain the transform settings)
        Some(Transformer {
            transformations: vec![],
        })
    }
    pub fn transform(&self, value: &mut Value) {
        // TODO: recursively check the value for properties that needs to be checked and apply transformations, return the transformed value
        *value.get_mut("a").unwrap() = json!("");
        if let Some(v) = value.get_mut("b") {
            *v = json!(12);
        }
        todo!();
    }
    fn transform_to_null(&self, value: &str) -> Option<String> {
        todo!();
    }
    fn transform_to_static(&self, value: &str) -> Option<String> {
        todo!();
    }
    fn hash_value(&self, value: &str) -> Option<String> {
        todo!();
    }
    fn regex_replace(&self, value: &str) -> Option<String> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_config_success() {
        todo!();
    }
    #[test]
    fn from_config_failed_wrong_input() {
        todo!();
    }
    #[test]
    fn no_transform_succeeds() {
        todo!();
    }

    #[test]
    fn no_transformations() {
        todo!();
    }

    #[test]
    fn transform_to_null() {
        todo!();
    }
    #[test]
    fn transform_to_static() {
        todo!();
    }
    #[test]
    fn transform_hash_value() {
        todo!();
    }
    #[test]
    fn regex_replace_value() {
        todo!();
    }
    #[test]
    fn regex_replace_based_on_column() {
        todo!();
    }
}
