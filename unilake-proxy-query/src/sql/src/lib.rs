#![allow(dead_code)]
#![allow(unused_variables)]

use pyo3::prelude::*;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;

pub fn run_scan_operation(
    query: &str,
    dialect: &str,
    catalog: &str,
    database: &str,
) -> PyResult<ScanOutput> {
    let start_time = std::time::Instant::now();
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let builtins = PyModule::import_bound(py, "sqlparser")?;
        let result = builtins
            .getattr("scan")
            .unwrap()
            .call1((query, dialect, catalog, database))?;

        let elapsed_time = std::time::Instant::now().duration_since(start_time);
        println!("Elapsed time [Scan]: {:?}", elapsed_time);

        Ok(result.extract::<ScanOutput>()?)
    })
}

pub fn run_transpile_operation(
    input: &TranspilerInput,
    secure_output: bool,
) -> PyResult<TranspilerOutput> {
    pyo3::prepare_freethreaded_python();
    let start_time = std::time::Instant::now();
    Python::with_gil(|py| {
        let builtins = PyModule::import_bound(py, "sqlparser").unwrap();
        let result = builtins
            .getattr("transpile")
            .unwrap()
            // todo(mrhamburg): properly unwrap serde_json to avoid panic
            .call1((serde_json::to_string(input).unwrap(), secure_output))
            .unwrap();

        let elapsed_time = std::time::Instant::now().duration_since(start_time);
        println!("Elapsed time [Transpile]: {:?}", elapsed_time);

        Ok(result.extract::<TranspilerOutput>()?)
    })
}

pub fn run_secure_operation(input: &str) -> PyResult<String> {
    pyo3::prepare_freethreaded_python();
    let start_time = std::time::Instant::now();
    Python::with_gil(|py| {
        let builtins = PyModule::import_bound(py, "sqlparser").unwrap();
        let result = builtins
            .getattr("secure_query")
            .unwrap()
            // todo(mrhamburg): properly unwrap serde_json to avoid panic
            .call1((input,))
            .unwrap();

        let elapsed_time = std::time::Instant::now().duration_since(start_time);
        println!("Elapsed time [Secure]: {:?}", elapsed_time);

        Ok(result.extract::<String>()?)
    })
}

impl<'py> FromPyObject<'py> for ScanOutput {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(ScanOutput {
            objects: ob.getattr("objects")?.extract()?,
            dialect: ob.getattr("dialect")?.extract()?,
            query: Some(ob.getattr("query")?.to_string()),
            query_type: ob.getattr("type")?.extract()?,
            error: ob.getattr("error")?.extract()?,
            target_entity: ob.getattr("target_entity")?.extract()?,
        })
    }
}

#[derive(FromPyObject)]
pub struct TranspilerOutput {
    pub sql_transformed: String,
    pub error: Option<ParserError>,
}

pub struct ScanOutput {
    pub objects: Vec<ScanOutputObject>,
    pub dialect: String,
    pub query: Option<String>,
    /// Expect: SELECT, UPDATE, DELETE, INSERT, ALTER
    pub query_type: String,
    pub error: Option<ParserError>,
    /// Full target entity path, expressed in: "some_catalog"."some_schema"."some_table"
    pub target_entity: Option<String>,
}

impl ScanOutput {
    pub fn get_full_path_names(&self) -> (Option<&str>, Option<&str>, Option<&str>) {
        if let Some(entity) = &self.target_entity {
            let parts: Vec<&str> = entity.split('"').collect();
            return match parts.len() {
                1 => todo!(),
                2 => todo!(),
                3 => (Some(parts[0]), Some(parts[1]), Some(parts[2])),
                _ => (None, None, None),
            };
        }
        (None, None, None)
    }
}

#[derive(FromPyObject)]
pub struct ScanOutputObject {
    pub scope: i32,
    pub entities: Vec<ScanEntity>,
    pub attributes: Vec<ScanAttribute>,
    pub is_agg: bool,
}

#[derive(FromPyObject)]
pub struct ScanEntity {
    pub catalog: String,
    pub db: String,
    pub name: String,
    pub alias: String,
}

impl ScanEntity {
    /// Full name <catalog>.<db>.<entity_name>
    pub fn get_full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog, self.db, self.name)
    }
}

#[derive(FromPyObject)]
pub struct ScanAttribute {
    pub entity_alias: String,
    pub name: String,
    pub alias: String,
}

impl ScanAttribute {
    pub fn get_name(&self) -> String {
        format!("\"{}\".\"{}\"", self.entity_alias, self.name)
    }
}

#[derive(FromPyObject, Debug)]
pub struct ParserError {
    pub error_type: String,
    pub message: String,
    pub errors: Vec<ErrorMessage>,
}

#[derive(FromPyObject, Debug)]
pub struct ErrorMessage {
    pub description: String,
    pub line: u32,
    pub col: u32,
    pub start_context: String,
    pub highlight: String,
    pub end_context: String,
    pub into_expression: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct TranspilerDenyCause {
    pub scope: i32,
    pub attribute: String,
    pub policy_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct PolicyAccessRequestUrl {
    pub url: String,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct TranspilerInput {
    pub rules: Vec<TranspilerInputRule>,
    pub filters: Vec<TranspilerInputFilter>,
    pub visible_schema: Option<HashMap<String, Catalog>>,
    pub cause: Option<Vec<TranspilerDenyCause>>,
    pub query: String,
    pub request_url: Option<Vec<PolicyAccessRequestUrl>>,
}

impl TranspilerInput {
    pub fn is_approved(&self) -> bool {
        self.cause.is_none() && self.request_url.is_none()
    }
}

pub struct VisibleSchemaBuilder {
    pub catalog: HashMap<String, Catalog>,
}

impl VisibleSchemaBuilder {
    pub fn new() -> Self {
        VisibleSchemaBuilder {
            catalog: HashMap::new(),
        }
    }

    pub fn get_or_add_catalog(&mut self, name: String) -> &mut Catalog {
        self.catalog
            .entry(name)
            .or_insert(Catalog { db: HashMap::new() })
    }
}

#[derive(Serialize, Debug)]
pub struct Catalog {
    #[serde(flatten)]
    pub db: HashMap<String, Database>,
}

impl Catalog {
    pub fn get_or_add_database(&mut self, name: String) -> &mut Database {
        self.db.entry(name).or_insert(Database {
            table: HashMap::new(),
        })
    }
}

#[derive(Serialize, Debug)]
pub struct Database {
    #[serde(flatten)]
    pub table: HashMap<String, Table>,
}

impl Database {
    pub fn get_or_add_table(&mut self, name: String) -> &mut Table {
        self.table.entry(name).or_insert(Table {
            columns: HashMap::new(),
        })
    }
}

#[derive(Serialize, Debug)]
pub struct Table {
    #[serde(flatten)]
    pub columns: HashMap<String, String>,
}

impl Table {
    pub fn get_or_add_column(&mut self, name: String, data_type: String) -> &mut Table {
        self.columns.entry(name).or_insert(data_type);
        self
    }
}

#[derive(Serialize, Debug)]
pub struct TranspilerInputRule {
    pub scope: i32,
    pub attribute_id: String,
    pub attribute: String,
    pub policy_id: String,
    pub rule_definition: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct TranspilerInputFilter {
    pub scope: i32,
    pub attribute_id: String,
    pub attribute: String,
    pub policy_id: String,
    pub filter_definition: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use crate::{
        run_scan_operation, run_transpile_operation, TranspilerInput, TranspilerInputFilter,
        TranspilerInputRule, VisibleSchemaBuilder,
    };
    use pyo3::PyResult;
    use serde_json::json;
    use serde_json::Value::Null;

    #[test]
    fn test_scan_operation_happy_flow() {
        let sql = "select top 100 * from employees";
        let output = run_scan_operation(sql, "tsql", "catalog", "database").unwrap();
        assert_eq!("SELECT", output.query_type);
    }

    #[test]
    fn test_scan_operation_error_flow() {
        let sql = "SELECT foo FROM (SELECT baz FROM t";
        let output = run_scan_operation(sql, "tsql", "catalog", "database").unwrap();
        assert_eq!("UNKNOWN", output.query_type);
        assert_eq!(output.error.unwrap().errors.len(), 1);
    }

    #[test]
    fn test_transpile_operation_happy_flow() -> PyResult<()> {
        let sql = "select top 100 * from employees";
        let scan_result = run_scan_operation(sql, "tsql", "catalog", "database").unwrap();

        let mut builder = VisibleSchemaBuilder::new();
        let catalog = builder.get_or_add_catalog("catalog".to_string());
        let database = catalog.get_or_add_database("database".to_string());
        let table = database.get_or_add_table("employees".to_string());
        table.get_or_add_column("id".to_string(), "int".to_string());
        table.get_or_add_column("name".to_string(), "string".to_string());
        table.get_or_add_column("a".to_string(), "string".to_string());

        let output = run_transpile_operation(
            &TranspilerInput {
                cause: None,
                query: scan_result.query.unwrap(),
                request_url: None,
                rules: vec![TranspilerInputRule {
                    attribute_id: "".to_string(),
                    attribute: r#""employees"."a""#.to_string(),
                    scope: 0,
                    policy_id: "".to_string(),
                    rule_definition: json!({"name": "xxhash3", "properties": Null}),
                }],
                filters: vec![TranspilerInputFilter {
                    attribute_id: "".to_string(),
                    attribute: r#""employees"."id""#.to_string(),
                    scope: 0,
                    policy_id: "".to_string(),
                    filter_definition: json!({"expression": "? > 100"}),
                }],
                visible_schema: Some(builder.catalog),
            },
            false,
        )?;
        // todo(mrhamburg): the transformed query has a nondeterministic ordering, make sure this is deterministic instead
        // assert_eq!(
        //     output.sql_transformed,
        //     "SELECT `employees`.`id` AS `id`, `employees`.`name` AS `name`, XX_HASH3_128(`employees`.`a`) AS `a` FROM `catalog`.`database`.`employees` AS `employees` WHERE `employees`.`id` > 100 LIMIT 100"
        // );
        assert!(output.error.is_none());
        Ok(())
    }
}
