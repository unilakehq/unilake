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
    pub query_type: String,
    pub error: Option<ParserError>,
    pub target_entity: Option<String>,
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

#[derive(FromPyObject)]
pub struct ScanAttribute {
    pub entity: String,
    pub name: String,
    pub alias: String,
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
    pub line: i32,
    pub col: i32,
    pub start_context: String,
    pub highlight: String,
    pub end_context: String,
    pub into_expression: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct TranspilerInput {
    pub rules: Vec<TranspilerInputRule>,
    pub filters: Vec<TranspilerInputFilter>,
    pub visible_schema: Option<HashMap<String, Catalog>>,
    pub cause: Option<serde_json::Value>,
    pub query: String,
    pub request_url: Option<String>,
}

struct VisibleSchemaBuilder {
    pub catalog: HashMap<String, Catalog>,
}

impl VisibleSchemaBuilder {
    fn new() -> Self {
        VisibleSchemaBuilder {
            catalog: HashMap::new(),
        }
    }
    fn get_or_add_catalog(&mut self, name: String) -> &mut Catalog {
        self.catalog
            .entry(name)
            .or_insert(Catalog { db: HashMap::new() })
    }
    fn get_or_add_database(catalog: &mut Catalog, name: String) -> &mut Database {
        catalog.db.entry(name).or_insert(Database {
            table: HashMap::new(),
        })
    }
    fn get_or_add_table(database: &mut Database, name: String) -> &mut Table {
        database.table.entry(name).or_insert(Table {
            columns: HashMap::new(),
        })
    }
    fn get_or_add_column(table: &mut Table, name: String, data_type: String) -> &mut Table {
        table.columns.entry(name).or_insert(data_type);
        table
    }
}

#[derive(Serialize, Debug)]
pub struct Catalog {
    #[serde(flatten)]
    db: HashMap<String, Database>,
}

#[derive(Serialize, Debug)]
struct Database {
    #[serde(flatten)]
    table: HashMap<String, Table>,
}

#[derive(Serialize, Debug)]
struct Table {
    #[serde(flatten)]
    columns: HashMap<String, String>,
}

#[derive(Serialize, Debug)]
pub struct TranspilerInputRule {
    pub scope: i32,
    pub attribute: String,
    pub rule_id: String,
    pub rule_definition: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct TranspilerInputFilter {
    pub scope: i32,
    pub attribute: String,
    pub filter_id: String,
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
        let database = VisibleSchemaBuilder::get_or_add_database(catalog, "database".to_string());
        let table = VisibleSchemaBuilder::get_or_add_table(database, "employees".to_string());
        VisibleSchemaBuilder::get_or_add_column(table, "id".to_string(), "int".to_string());
        VisibleSchemaBuilder::get_or_add_column(table, "name".to_string(), "string".to_string());
        VisibleSchemaBuilder::get_or_add_column(table, "a".to_string(), "string".to_string());

        let output = run_transpile_operation(
            &TranspilerInput {
                cause: None,
                query: scan_result.query.unwrap(),
                request_url: None,
                rules: vec![TranspilerInputRule {
                    attribute: r#""employees"."a""#.to_string(),
                    scope: 0,
                    rule_id: "".to_string(),
                    rule_definition: json!({"name": "xxhash3", "properties": Null}),
                }],
                filters: vec![TranspilerInputFilter {
                    attribute: r#""employees"."id""#.to_string(),
                    scope: 0,
                    filter_id: "".to_string(),
                    filter_definition: json!({"expression": "? > 100"}),
                }],
                visible_schema: Some(builder.catalog),
            },
            false,
        )?;
        assert!(output.sql_transformed.ne(""));
        assert!(output.error.is_none());
        Ok(())
    }
}