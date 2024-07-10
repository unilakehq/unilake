use std::collections::HashMap;

use serde_json::value::Value;
use serde_json::Number;

pub use crate::error::FlattenError;
use crate::utils;

pub struct Flattener {
    /// Name of the base object stream
    base_stream: String,
    /// UTC timestamp of when this object was flattened
    normalized_at: usize,
}

impl Flattener {
    /// Creates a JSON object flattener with the default configuration.
    #[must_use]
    pub fn new(base_stream: String, normalized_at: Option<usize>) -> Self {
        Self {
            base_stream,
            normalized_at: normalized_at.unwrap_or(utils::get_current_time_in_seconds() as usize),
        }
    }

    /// Flattens the provided JSON object (`to_flatten`).
    /// Provided object will be consumed in the process.
    /// Result object is the flattened version of `to_flatten`.
    pub fn flatten(&self, to_flatten: Value) -> Result<HashMap<String, Vec<Value>>, FlattenError> {
        let mut flat: HashMap<String, Vec<Value>> = HashMap::new();
        self.flatten_value(&self.base_stream, to_flatten, None, &mut flat)?;
        Ok(flat)
    }

    /// Flattens the passed JSON value (`value`). The result is stored in the JSON object `flattened`.
    fn flatten_value(
        &self,
        key: &str,
        mut value: Value,
        base_obj: Option<(&str, &mut Value)>,
        flattened: &mut HashMap<String, Vec<Value>>,
    ) -> Result<(), FlattenError> {
        if value.is_object() && value.as_object().is_some() {
            let stream;
            let parent_obj = if let Some(b) = base_obj {
                stream = Self::remove_invalid_characters(&format!("{}_{}", b.0, key));
                Option::from(&*b.1)
            } else {
                stream = key.to_string();
                None
            };

            self.flatten_object(&stream, value, parent_obj, flattened)?;
        } else if let Some(current) = value.as_array_mut() {
            if !current.is_empty() {
                let stream;
                let parent_obj = if let Some(b) = base_obj {
                    stream = Self::remove_invalid_characters(&format!("{}_{}", b.0, key));
                    Option::from(&*b.1)
                } else {
                    stream = key.to_string();
                    None
                };

                self.flatten_array(&stream, current, parent_obj, flattened)?;
            }
        } else if let Some((_, current)) = base_obj {
            let current = current.as_object_mut().unwrap();
            let mut key = Self::remove_invalid_characters(key);
            while current.contains_key(&key) {
                key.push('_');
            }

            current.insert(key, value);
        }

        Ok(())
    }

    /// Flattens the passed object (`current`)
    fn flatten_object(
        &self,
        stream: &str,
        mut current: Value,
        parent_obj: Option<&Value>,
        flattened: &mut HashMap<String, Vec<Value>>,
    ) -> Result<(), FlattenError> {
        let mut base_obj = self.create_object(stream, Self::get_foreign_hash(parent_obj), &current);
        for (s, v) in current.as_object_mut().unwrap().iter_mut() {
            self.flatten_value(s, v.take(), Some((stream, &mut base_obj)), flattened)?;
        }
        Self::add_return_value(flattened, stream, base_obj);
        Ok(())
    }

    fn flatten_array(
        &self,
        stream: &str,
        current: &mut [Value],
        parent_obj: Option<&Value>,
        flattened: &mut HashMap<String, Vec<Value>>,
    ) -> Result<(), FlattenError> {
        current
            .iter_mut()
            .filter(|obj| !obj.is_null())
            .map(|obj| match obj {
                Value::Array(a) => self.flatten_array(stream, a, parent_obj, flattened),
                Value::Object(_) => self.flatten_object(stream, obj.take(), parent_obj, flattened),
                _ => {
                    let v = self.create_array_object(
                        stream,
                        Self::get_foreign_hash(parent_obj),
                        obj.take(),
                    );
                    Self::add_return_value(flattened, stream, v);
                    Ok(())
                }
            })
            .collect::<Result<Vec<()>, FlattenError>>()?;
        Ok(())
    }

    /// Create a new array based on the passed value
    fn create_array_object(
        &self,
        stream: &str,
        foreign_hash_id: Option<(String, String)>,
        data: Value,
    ) -> Value {
        let mut v = self.create_object(stream, foreign_hash_id, &data);
        v.as_object_mut().unwrap().insert("data".to_string(), data);
        v
    }

    /// Create a new object based on the passed value
    fn create_object(
        &self,
        stream: &str,
        foreign_hash_id: Option<(String, String)>,
        v: &Value,
    ) -> Value {
        let mut map = serde_json::map::Map::new();
        map.insert(
            format!("_{stream}_unilake_hashid"),
            Self::generate_hash_id(v),
        );
        if let Some(foreign_hash_id) = foreign_hash_id {
            map.insert(foreign_hash_id.0, Value::String(foreign_hash_id.1));
        }
        map.insert(
            "_unilake_normalized_at".to_string(),
            Value::Number(Number::from(self.normalized_at)),
        );
        Value::Object(map)
    }

    /// Add new value to the return object
    fn add_return_value(flattened: &mut HashMap<String, Vec<Value>>, stream: &str, v: Value) {
        //println!("Adding value ({:?}): {:?}", stream, &v);
        if !flattened.contains_key(stream) {
            flattened.insert(stream.to_string(), vec![v]);
            return;
        }
        flattened.get_mut(stream).unwrap().push(v);
    }

    /// Returns the hash of a parent object which can be used as foreign hash id
    fn get_foreign_hash(parent_obj: Option<&Value>) -> Option<(String, String)> {
        parent_obj.and_then(|obj| obj.as_object()).and_then(|obj| {
            obj.iter()
                .find(|(k, _v)| {
                    k.ends_with("_unilake_hashid") && !k.ends_with("_foreign_unilake_hashid")
                })
                .map(|(k, v)| {
                    (
                        k.replace("_unilake_hashid", "_foreign_unilake_hashid"),
                        v.as_str().unwrap().to_string(),
                    )
                })
        })
    }

    /// Generates a hash id based on the passed value, value returned is a json string value
    fn generate_hash_id(v: &Value) -> Value {
        let mut m = sha1_smol::Sha1::new();
        fn add_value(v: &Value, m: &mut sha1_smol::Sha1, header: bool) {
            match v {
                Value::Bool(b) => m.update(b.to_string().as_bytes()),
                Value::Number(n) => m.update(n.to_string().as_bytes()),
                Value::String(sv) => m.update(sv.as_bytes()),
                Value::Object(o) if header => {
                    for (_k, v) in o.iter() {
                        add_value(v, m, false);
                    }
                }
                _ => {}
            }
        }

        add_value(v, &mut m, true);

        Value::String(m.digest().to_string())
    }

    /// Consumes the passed in value and returns a new string containing no invalid characters
    fn remove_invalid_characters(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' => c.to_ascii_lowercase(),
                'a'..='z' => c,
                '0'..='9' => c,
                _ => '_',
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn single_object() {
        let stream = "cars";
        let obj = json!({"make": "alfa romeo", "model": "4C coupe"});
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert_eq!(
            result[stream],
            vec![
                json!({"_cars_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "make": "alfa romeo", "model": "4C coupe"})
            ]
        );
    }

    #[test]
    fn single_array() {
        let stream = "cars";
        let obj = json!([{"make": "alfa romeo", "model": "4C coupe"}]);
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert_eq!(
            result[stream],
            vec![
                json!({"_cars_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "make": "alfa romeo", "model": "4C coupe"})
            ]
        );
    }

    #[test]
    fn nested_object_array() {
        let stream = "cars";
        let obj = json!({"make": "alfa romeo", "model": "4C coupe", "limited_editions": [{"name": "4C spider", "release_year": 2013}, {"name": "4C spider italia", "release_year": 2018}]});
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert_eq!(
            result[stream],
            vec![
                json!({"_cars_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "make": "alfa romeo", "model": "4C coupe"})
            ]
        );
        assert_eq!(
            result[&format!("{}_limited_editions", stream)],
            vec![
                json!({"_cars_limited_editions_unilake_hashid": "ee0af87d7ded3d88477689d50f84e4f63919cbef", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "name": "4C spider", "release_year": 2013}),
                json!({"_cars_limited_editions_unilake_hashid": "3a713f7aff04414a4e6df62b9ade11f3bcbcddad", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "name": "4C spider italia", "release_year": 2018})
            ]
        );
    }

    #[test]
    fn double_nested_object_array() {
        let stream = "cars";
        let obj = json!({"make":"alfa romeo","model":"4C coupe","limited_editions":[{"name":"4C spider","release_year":2013,"price":{"base":125000,"margin":12},"manufactured_locations":["USA","JPN","NLD"]},{"name":"4C spider italia","release_year":2018,"price":{"base":125000,"margin":12},"manufactured_locations":["USA","JPN","GER"]}]});
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert_eq!(
            result[stream],
            vec![
                json!({"_cars_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "make": "alfa romeo", "model": "4C coupe"})
            ]
        );
        assert_eq!(
            result[&format!("{}_limited_editions", stream)],
            vec![
                json!({"_cars_limited_editions_unilake_hashid": "ee0af87d7ded3d88477689d50f84e4f63919cbef", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "name": "4C spider", "release_year": 2013}),
                json!({"_cars_limited_editions_unilake_hashid": "3a713f7aff04414a4e6df62b9ade11f3bcbcddad", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "name": "4C spider italia", "release_year": 2018})
            ]
        );
        assert_eq!(
            result[&format!("cars_limited_editions_manufactured_locations")],
            vec![
                json!({"_cars_limited_editions_manufactured_locations_unilake_hashid": "18bc5956dbf1cc6c9a5baeb624c9e7e472a04c2e", "_cars_limited_editions_foreign_unilake_hashid": "ee0af87d7ded3d88477689d50f84e4f63919cbef", "_unilake_normalized_at": 0, "data": "USA"}),
                json!({"_cars_limited_editions_manufactured_locations_unilake_hashid": "1d63f427f9147dbe3cc0af8c36ec4250cfa6902f", "_cars_limited_editions_foreign_unilake_hashid": "ee0af87d7ded3d88477689d50f84e4f63919cbef", "_unilake_normalized_at": 0, "data": "JPN"}),
                json!({"_cars_limited_editions_manufactured_locations_unilake_hashid": "f71350788f93db126432b5a36cee51a246f7cd2e", "_cars_limited_editions_foreign_unilake_hashid": "ee0af87d7ded3d88477689d50f84e4f63919cbef", "_unilake_normalized_at": 0, "data": "NLD"}),
                json!({"_cars_limited_editions_manufactured_locations_unilake_hashid": "18bc5956dbf1cc6c9a5baeb624c9e7e472a04c2e", "_cars_limited_editions_foreign_unilake_hashid": "3a713f7aff04414a4e6df62b9ade11f3bcbcddad", "_unilake_normalized_at": 0, "data": "USA"}),
                json!({"_cars_limited_editions_manufactured_locations_unilake_hashid": "1d63f427f9147dbe3cc0af8c36ec4250cfa6902f", "_cars_limited_editions_foreign_unilake_hashid": "3a713f7aff04414a4e6df62b9ade11f3bcbcddad", "_unilake_normalized_at": 0, "data": "JPN"}),
                json!({"_cars_limited_editions_manufactured_locations_unilake_hashid": "871f3a8eaf86c3ea64b2a87603ffea44913f2ee1", "_cars_limited_editions_foreign_unilake_hashid": "3a713f7aff04414a4e6df62b9ade11f3bcbcddad", "_unilake_normalized_at": 0, "data": "GER"}),
            ]
        );
    }

    #[test]
    fn nested_object_array_conflicted_name() {
        let stream = "cars";
        let obj = json!({"make": "alfa romeo", "model": "4C coupe", "cars": [{"name": "4C spider", "release_year": 2013}, {"name": "4C spider italia", "release_year": 2018}]});
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert_eq!(
            result[stream],
            vec![
                json!({"_cars_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "make": "alfa romeo", "model": "4C coupe"})
            ]
        );
        assert_eq!(
            result[&format!("{}_cars", stream)],
            vec![
                json!({"_cars_cars_unilake_hashid": "ee0af87d7ded3d88477689d50f84e4f63919cbef", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "name": "4C spider", "release_year": 2013}),
                json!({"_cars_cars_unilake_hashid": "3a713f7aff04414a4e6df62b9ade11f3bcbcddad", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "name": "4C spider italia", "release_year": 2018})
            ]
        );
    }

    #[test]
    fn nested_value_array() {
        let stream = "cars";
        let obj = json!({"make": "alfa romeo", "model": "4C coupe", "limited_editions": ["4C spider", "4C spider italia"]});
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert_eq!(
            result[stream],
            vec![
                json!({"_cars_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "make": "alfa romeo", "model": "4C coupe"})
            ]
        );
        assert_eq!(
            result[&format!("{}_limited_editions", stream)],
            vec![
                json!({"_cars_limited_editions_unilake_hashid": "e1892352b3766fa07d40c3cacf3362c635091c4f", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "data": "4C spider"}),
                json!({"_cars_limited_editions_unilake_hashid": "fa068e93b8ed7f42195a6a1065fe02c7c2962298", "_cars_foreign_unilake_hashid": "f06899b4185aaef6afac1e93a9c3b0020309c5b1", "_unilake_normalized_at": 0, "data": "4C spider italia"})
            ]
        );
    }

    #[test]
    fn sanity_check() {
        let stream = "items";
        let obj = json!({"data":[{"id":1,"name":"JohnDoe","age":35,"email":"johndoe@example.com","phone_numbers":["555-555-1234","555-555-5678"],"address":{"street":"123MainSt.","city":"SanFrancisco","state":"CA","zipcode":"94117"}},{"id":2,"name":"JaneDoe","age":31,"email":"janedoe@example.com","phone_numbers":["555-555-4321","555-555-9876"],"address":{"street":"456MainSt.","city":"SanJose","state":"CA","zipcode":"95123"}},{"id":3,"name":"JimSmith","age":42,"email":"jimsmith@example.com","phone_numbers":["555-555-6789","555-555-1235"],"address":{"street":"789MainSt.","city":"LosAngeles","state":"CA","zipcode":"90001"}}],"employees":[{"employee_id":101,"name":"JohnSmith","department":"Sales","salary":65000,"projects":["ProjectA","ProjectB"],"performance_reviews":[{"year":2020,"rating":"Outstanding"},{"year":2019,"rating":"ExceedsExpectations"}]},{"employee_id":102,"name":"JaneLee","department":"Marketing","salary":55000,"projects":["ProjectC","ProjectD"],"performance_reviews":[{"year":2020,"rating":"MeetsExpectations"},{"year":2019,"rating":"ExceedsExpectations"}]},{"employee_id":103,"name":"JimBrown","department":"Engineering","salary":75000,"projects":["ProjectE","ProjectF"],"performance_reviews":[{"year":2020,"rating":"Outstanding"},{"year":2019,"rating":"MeetsExpectations"}]}],"inventory":[{"item_id":201,"name":"Laptop","quantity":50,"price":1000,"manufacturer":"Apple"},{"item_id":202,"name":"Smartphone","quantity":100,"price":700,"manufacturer":"Samsung"},{"item_id":203,"name":"Tablet","quantity":75,"price":800,"manufacturer":"Amazon"}],"orders":[{"order_id":301,"customer_id":1,"items":[{"item_id":201,"quantity":2},{"item_id":202,"quantity":1}],"total_amount":1800,"order_date":"2022-01-01"},{"order_id":302,"customer_id":2,"items":[{"item_id":203,"quantity":3}],"total_amount":2400,"order_date":"2022-02-01"},{"order_id":303,"customer_id":3,"items":[{"item_id":201,"quantity":1},{"item_id":203,"quantity":2}],"total_amount":1700,"order_date":"2022-03-01"}]});
        let _result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();
    }

    #[test]
    fn double_nested_objects() {
        let stream = "items";
        let obj = json!({"data":[{"MainId":1111,"firstName":"Sherlock","lastName":"Homes","categories":[{"CategoryID":1,"CategoryName":"Example"}]},{"MainId":122,"firstName":"James","lastName":"Watson","categories":[{"CategoryID":2,"CategoryName":"Example2"}]}],"messages":[],"success":true});
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert!(result.contains_key(&format!("{}_data_categories", stream)));
    }

    #[test]
    fn special_characters() {
        let stream = "items";
        let obj = json!([{"id": 3, "currency": "GBP", "NZD": 3.14, "HKD@spéçiäl & characters": 9.2, "hkd_sp__i_l___characters": "column name collision?"},
        {"id": 3, "currency": "GBP", "NZD": 3.14, "hkd_sp__i_l___characters": "column name collision?", "HKD@spéçiäl & characters": 3.4}]);
        let result = Flattener::new(stream.to_string(), 0.into())
            .flatten(obj)
            .unwrap();

        assert!(result.contains_key("items"));
    }
}
