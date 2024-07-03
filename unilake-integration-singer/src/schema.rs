use std::collections::HashMap;

use arrow2::datatypes::DataType;
use arrow2::datatypes::Field;
use arrow2::datatypes::Schema;
use log::debug;

use crate::parquet::ParquetFilePart;

pub const UNILAKE_AB_ID: &str = "_unilake_ab_id";
pub const UNILAKE_EMITTED_AT: &str = "_unilake_emitted_at";
pub const UNILAKE_DATA: &str = "_unilake_data";

pub struct SchemaGenerator {
    configured_catalog: serde_json::Value,
}

impl SchemaGenerator {
    pub fn new(configured_catalog: serde_json::Value) -> SchemaGenerator {
        SchemaGenerator { configured_catalog }
    }

    pub fn get_configured_schema(&self, stream: &str) -> Option<Schema> {
        debug!("Trying to get configured schema for stream {}", stream);

        let streams = self.configured_catalog.get("streams")?.as_array()?;
        let stream_obj = streams.iter().find(|i| i["name"] == stream)?;

        let schema_obj = stream_obj["json_schema"].as_object()?;
        if let Some("object") = schema_obj.get("type").and_then(|t| t.as_str()) {
            let props_obj = schema_obj["properties"].as_object()?;
            let fields = props_obj
                .iter()
                .map(|(key, value)| {
                    Field::new(
                        key,
                        SchemaGenerator::get_data_type_from_catalog_value(value),
                        true,
                    )
                })
                .collect::<Vec<Field>>();
            Some(Schema::from(
                vec![
                    Field::new(UNILAKE_AB_ID, DataType::Utf8, false),
                    Field::new(UNILAKE_EMITTED_AT, DataType::UInt64, false),
                ]
                .into_iter()
                .chain(fields.into_iter())
                .collect::<Vec<Field>>(),
            ))
        } else {
            None
        }
    }

    fn get_data_type_from_catalog_value(value: &serde_json::Value) -> DataType {
        let data_type_str = value["type"]
            .as_array()
            .and_then(|t| t.get(0).and_then(|t| t.as_str()))
            .or_else(|| value["type"].as_str())
            .expect("Value type is missing");

        match data_type_str {
            "string" => DataType::Utf8,
            "number" => DataType::Float64,
            "integer" => DataType::Int64,
            "boolean" => DataType::Boolean,
            "object" => {
                let fields = value["properties"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(key, value)| {
                        Field::new(key, Self::get_data_type_from_catalog_value(value), true)
                    })
                    .collect();
                DataType::Struct(fields)
            }
            "array" => {
                let item_data_type = Self::get_data_type_from_catalog_value(&value["properties"]);
                DataType::List(Box::new(Field::new("", item_data_type, true)))
            }
            other => unimplemented!("{} is not a valid data type that we support", other),
        }
    }

    fn get_data_type_from_inferred_value(v: &serde_json::Value) -> Option<DataType> {
        match v {
            serde_json::Value::Bool(_) => Some(DataType::Boolean),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Some(DataType::Int64)
                } else if n.is_u64() {
                    Some(DataType::UInt64)
                } else {
                    Some(DataType::Float64)
                }
            }
            serde_json::Value::String(_) => Some(DataType::Utf8),
            serde_json::Value::Array(t) if !t.is_empty() => {
                Self::get_data_type_from_inferred_value(&t[0])
            }
            serde_json::Value::Object(o) => {
                let fields = o
                    .iter()
                    .map(|(key, value)| {
                        Field::new(
                            key,
                            Self::get_data_type_from_inferred_value(value).unwrap(),
                            true,
                        )
                    })
                    .collect();
                Some(DataType::Struct(fields))
            }
            _ => None,
        }
    }

    pub fn get_inferred_schema(f: &ParquetFilePart) -> Schema {
        let mut schema = HashMap::<&str, DataType>::new();
        schema.insert(UNILAKE_AB_ID, DataType::Utf8);
        schema.insert(UNILAKE_EMITTED_AT, DataType::Float64);

        for r in f.records.iter() {
            let datapoint = r.data.as_object().unwrap();
            for (k, v) in datapoint.iter() {
                if let Some(dt) = Self::get_data_type_from_inferred_value(v) {
                    if v.is_number() {
                        schema.entry(k).and_modify(|st| {
                            if dt == DataType::Float64 {
                                match st {
                                    DataType::Float64 => st,
                                    _ => &mut DataType::Float64,
                                };
                            } else if dt == DataType::Int64 {
                                match st {
                                    DataType::Int64 => st,
                                    DataType::Float64 => &mut DataType::Float64,
                                    _ => &mut DataType::Int64,
                                };
                            } else if dt == DataType::UInt64 {
                                match st {
                                    DataType::UInt64 => st,
                                    DataType::Float64 => &mut DataType::Float64,
                                    DataType::Int64 => &mut DataType::Int64,
                                    _ => &mut DataType::UInt64,
                                };
                            };
                        });
                    }
                    schema.entry(k).or_insert(dt);
                }
            }
        }

        let schema: Vec<Field> = schema
            .iter()
            .map(|(k, v)| Field::new(k.to_owned(), v.clone(), true))
            .collect();

        Schema::from(schema)
    }

    pub fn get_raw_schema() -> Schema {
        Schema::from(vec![
            Field::new(UNILAKE_AB_ID, DataType::Utf8, false),
            Field::new(UNILAKE_EMITTED_AT, DataType::UInt64, false),
            Field::new(UNILAKE_DATA, DataType::Utf8, false),
        ])
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn get_raw_schema_len_should_be_3() {
        assert_eq!(SchemaGenerator::get_raw_schema().fields.len(), 3);
    }

    #[test]
    fn get_configured_schema_dynamic_streams() {
        let configured_catalog = json!({
            "streams": [
                {
                    "name": "TSLA",
                    "supported_sync_modes": ["full_refresh","incremental"],
                    "source_defined_cursor": false,
                    "json_schema": {
                        "type": "object",
                        "properties": {
                            "symbol": {"type": "string"},
                            "price": {"type": "number"},
                            "date": {"type": "string"}
                        }
                    }
                },
                {
                    "name": "FB",
                    "supported_sync_modes": ["full_refresh","incremental"],
                    "source_defined_cursor": false,
                    "json_schema": {
                        "type": "object",
                        "properties": {
                            "symbol": {"type": "string"},
                            "price": {"type": "number"},
                            "date": {"type": "string"}
                        }
                    }
                }
            ]
        });
        let obj = SchemaGenerator::new(configured_catalog);

        let tsla = obj.get_configured_schema("TSLA").unwrap();
        assert_eq!(tsla.fields.len(), 5);
        assert_eq!(tsla.fields[2], Field::new("date", DataType::Utf8, true));
        assert_eq!(tsla.fields[3], Field::new("price", DataType::Float64, true));
        assert_eq!(tsla.fields[4], Field::new("symbol", DataType::Utf8, true));

        let fb = obj.get_configured_schema("FB").unwrap();
        assert_eq!(fb.fields.len(), 5);
        assert_eq!(fb.fields[2], Field::new("date", DataType::Utf8, true));
        assert_eq!(fb.fields[3], Field::new("price", DataType::Float64, true));
        assert_eq!(fb.fields[4], Field::new("symbol", DataType::Utf8, true));
    }

    #[test]
    fn get_configured_schema_nested_stream_flights_objects() {
        let configured_catalog = json!({
            "streams": [{
                "name": "flights",
                "supported_sync_modes": ["full_refresh"],
                "source_defined_cursor": false,
                "json_schema": {
                    "type": "object",
                    "properties": {
                        "airline": {"type": "string"},
                        "origin": {
                            "type": "object",
                            "properties": {
                                "airport_code": {"type": "string"},
                                "terminal": {"type": "string"},
                                "gate": {"type": "string"}
                            }
                        },
                        "destination": {
                            "type": "object",
                            "properties": {
                                "airport_code": {"type": "string"},
                                "terminal": {"type": "string"},
                                "gate": {"type": "string"}
                            }
                        }
                    }
                }
            }]
        });

        let obj = SchemaGenerator::new(configured_catalog);
        let flights = obj.get_configured_schema("flights").unwrap();

        assert_eq!(flights.fields.len(), 5);
        assert_eq!(
            flights.fields[2],
            Field::new("airline", DataType::Utf8, true)
        );
        assert_eq!(
            flights.fields[3],
            Field::new(
                "destination",
                DataType::Struct(vec![
                    Field::new("airport_code", DataType::Utf8, true),
                    Field::new("gate", DataType::Utf8, true),
                    Field::new("terminal", DataType::Utf8, true),
                ]),
                true
            )
        );
        assert_eq!(
            flights.fields[4],
            Field::new(
                "origin",
                DataType::Struct(vec![
                    Field::new("airport_code", DataType::Utf8, true),
                    Field::new("gate", DataType::Utf8, true),
                    Field::new("terminal", DataType::Utf8, true),
                ]),
                true
            )
        );
    }

    #[test]
    fn get_configured_schema_nested_stream_flights_array() {
        let configured_catalog = json!({
          "streams": [
            {
              "name": "flights",
              "supported_sync_modes": ["full_refresh"],
              "source_defined_cursor": false,
              "json_schema": {
                "type": "object",
                "properties": {
                  "airline": {"type": "string"},
                  "airports": {
                    "type": "array",
                    "properties": {
                      "type": "string"
                    }
                  }
                }
              }
            }
          ]
        });

        let obj = SchemaGenerator::new(configured_catalog);
        let flights = obj.get_configured_schema("flights").unwrap();

        assert_eq!(flights.fields.len(), 4);
        assert_eq!(
            flights.fields[2],
            Field::new("airline", DataType::Utf8, true)
        );
        assert_eq!(
            flights.fields[3],
            Field::new(
                "airports",
                DataType::List(Box::new(Field::new("", DataType::Utf8, true))),
                true
            )
        );
    }

    // TODO: set tests for inferred schema
}
