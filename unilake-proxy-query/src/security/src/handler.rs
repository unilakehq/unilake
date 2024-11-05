use std::sync::Arc;
use unilake_common::error::TdsWireError;
use unilake_sql::{
    run_scan_operation, run_secure_operation, run_transpile_operation, ParserError, ScanOutput,
    TranspilerInput,
};

// todo: needs to be placed in the protocol section instead

pub enum QueryHandlerError {
    WireError(TdsWireError),
    QueryError(ParserError),
}

impl From<TdsWireError> for QueryHandlerError {
    fn from(value: TdsWireError) -> Self {
        QueryHandlerError::WireError(value)
    }
}

impl From<ParserError> for QueryHandlerError {
    fn from(value: ParserError) -> Self {
        QueryHandlerError::QueryError(value)
    }
}

pub struct QueryHandler {
    scan_output: Option<ScanOutput>,
    transpiler_input: Option<TranspilerInput>,
    output_query: Option<Arc<str>>,
    output_query_secured: Option<Arc<str>>,
    input_query_secured: Option<Arc<str>>,
    input_query: Option<Arc<str>>,
}

impl QueryHandler {
    pub fn new() -> Self {
        QueryHandler {
            scan_output: None,
            transpiler_input: None,
            output_query: None,
            output_query_secured: None,
            input_query_secured: None,
            input_query: None,
        }
    }

    /// Scan the provided query for context (which catalogs, databases, tables and columns)
    /// are used by this query. Can be used by downstream processes to determine further actions.
    ///
    /// # Parameters
    ///
    /// * `query` - A reference to a string containing the SQL query to be scanned.
    /// * `dialect` - A reference to a string containing the dialect of the current session.
    /// * `catalog` - A reference to a string containing the catalog of the current session.
    /// * `database` - A reference to a string containing the database of the current session.
    ///
    /// # Returns
    ///
    /// * `Result<ScanOutput, QueryHandlerError>` - On success, returns a `ScanOutput` containing the
    ///   result of the scan operation and all contextual information. On error, returns a `QueryHandlerError`
    ///   containing the error that occurred during the scan operation.
    fn scan(
        &self,
        query: &str,
        dialect: &str,
        catalog: &str,
        database: &str,
    ) -> Result<ScanOutput, QueryHandlerError> {
        Ok(run_scan_operation(query, dialect, catalog, database)
            .map_err(|e| TdsWireError::Protocol(e.to_string()))?)
    }

    /// Handles a query by applying all necessary transformations and rules to the query.
    ///
    /// This function takes a SQL query and session information, processes the query,
    /// and returns the resulting query ready for execution.
    ///
    /// # Parameters
    ///
    /// * `query` - SQL query to be executed, as received from the client
    /// * `dialect` - A reference to a string containing the dialect of the current session.
    /// * `catalog` - A reference to a string containing the catalog of the current session.
    /// * `database` - A reference to a string containing the database of the current session.
    ///
    /// # Returns
    ///
    /// * `Result<String, QueryHandlerError>` - On success, returns a `String` containing the
    ///   result of all scanning and transpilation, ready for execution. On error, returns a `QueryHandlerError` containing
    ///   information about what went wrong during analyses and transpilation.
    pub fn handle_query(
        &mut self,
        query: &str,
        dialect: &str,
        catalog: &str,
        database: &str,
    ) -> Result<&str, QueryHandlerError> {
        // You can only handle a query once
        if let Some(ref query_result) = self.output_query {
            return Ok(query_result);
        }

        self.input_query = Some(Arc::from(query.to_string()));
        let scan_output = self.scan(query, dialect, catalog, database)?;
        // todo(mrhamburg): pdp has state and caching and all, needs to be improved
        let pdp = QueryPolicyDecision::new();
        let transpiler_input = pdp.from_scan_output(&scan_output)?;
        let output_query = self.transpile_query(&transpiler_input, false)?;

        self.scan_output = Some(scan_output);
        self.transpiler_input = Some(transpiler_input);
        self.output_query = Some(Arc::from(output_query));

        Ok(self.output_query.as_ref().unwrap())
    }

    fn transpile_query(
        &self,
        scanned: &TranspilerInput,
        secure_output: bool,
    ) -> Result<String, QueryHandlerError> {
        let transpiler_output = run_transpile_operation(scanned, secure_output)
            .map_err(|e| TdsWireError::Protocol(e.to_string()))?;
        if let Some(error) = transpiler_output.error {
            return Err(QueryHandlerError::QueryError(error));
        }
        Ok(transpiler_output.sql_transformed)
    }

    pub fn secure_output_query(&mut self) -> Result<&str, QueryHandlerError> {
        // You can only secure a query once
        if let Some(ref output_query_secured) = self.output_query_secured {
            return Ok(output_query_secured);
        } else if let Some(ref transpiler_input) = self.transpiler_input {
            self.output_query_secured =
                Some(Arc::from(self.transpile_query(transpiler_input, true)?));
        }

        Ok(self.output_query_secured.as_ref().unwrap())
    }

    pub fn secure_input_query(&mut self) -> Result<&str, QueryHandlerError> {
        // You can only secure an input query once
        if let Some(ref input_query_secured) = self.input_query_secured {
            return Ok(input_query_secured);
        }

        self.input_query_secured = Some(Arc::from(
            run_secure_operation(self.input_query.as_ref().unwrap().as_ref())
                .map_err(|e| TdsWireError::Protocol(e.to_string()))?,
        ));
        Ok(self.input_query_secured.as_ref().unwrap())
    }
}

// TODO: implement instead of PolicyManager?
struct QueryPolicyDecision {}

impl QueryPolicyDecision {
    pub fn new() -> Self {
        QueryPolicyDecision {}
    }

    pub fn from_scan_output(
        &self,
        scan_output: &ScanOutput,
    ) -> Result<TranspilerInput, QueryHandlerError> {
        Ok(TranspilerInput {
            cause: None,
            query: scan_output.query.clone().unwrap(),
            request_url: None,
            rules: vec![],
            filters: vec![],
            visible_schema: None,
        })
    }
}
