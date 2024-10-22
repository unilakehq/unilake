use crate::frontend::error::TdsWireError;
use crate::frontend::TokenError;
use crate::session::{
    SessionInfo, SESSION_VARIABLE_CATALOG, SESSION_VARIABLE_DATABASE, SESSION_VARIABLE_DIALECT,
};
use unilake_sql::{
    run_scan_operation, run_transpile_operation, ParserError, ScanOutput, TranspilerInput,
};

// TODO: implementation of the main PDP functionality
impl From<ParserError> for TokenError {
    fn from(value: ParserError) -> Self {
        if let Some(value) = value.errors.first() {
            TokenError {
                line: value.line as u32,
                code: 0,
                message: value.description.to_string(),
                class: 0,
                procedure: "".to_string(),
                server: "".to_string(),
                state: 0,
            }
        } else {
            TokenError {
                code: 0,
                state: 0,
                class: 0,
                message: value.message,
                server: "".to_string(),
                procedure: "".to_string(),
                line: 0,
            }
        }
    }
}

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
    output_query: Option<String>,
    output_query_secured: Option<String>,
}

impl QueryHandler {
    pub fn new() -> Self {
        QueryHandler {
            scan_output: None,
            transpiler_input: None,
            output_query: None,
            output_query_secured: None,
        }
    }

    /// Scan the provided query for context (which catalogs, databases, tables and columns)
    /// are used by this query. Can be used by downstream processes to determine further actions.
    ///
    /// # Parameters
    ///
    /// * `query` - A reference to a string containing the SQL query to be scanned.
    /// * `session_info` - A reference to a trait object implementing the `SessionInfo` trait,
    ///   which provides session-specific information such as dialect, catalog, and database.
    ///
    /// # Returns
    ///
    /// * `Result<ScanOutput, QueryHandlerError>` - On success, returns a `ScanOutput` containing the
    ///   result of the scan operation and all contextual information. On error, returns a `QueryHandlerError`
    ///   containing the error that occurred during the scan operation.
    fn scan(
        &self,
        query: &str,
        session_info: &dyn SessionInfo,
    ) -> Result<ScanOutput, QueryHandlerError> {
        let dialect = session_info
            .get_session_variable(SESSION_VARIABLE_DIALECT)
            .get_value_or_default();
        let catalog = session_info
            .get_session_variable(SESSION_VARIABLE_CATALOG)
            .get_value_or_default();
        let database = session_info
            .get_session_variable(SESSION_VARIABLE_DATABASE)
            .get_value_or_default();

        Ok(
            run_scan_operation(query, dialect.as_ref(), catalog.as_ref(), database.as_ref())
                .map_err(|e| TdsWireError::Protocol(e.to_string()))?,
        )
    }

    /// Handles a query by applying all necessary transformations and rules to the query.
    ///
    /// This function takes a SQL query and session information, processes the query,
    /// and returns the resulting query ready for execution.
    ///
    /// # Parameters
    ///
    /// * `query` - SQL query to be executed, as received from the client
    /// * `session_info` - A reference to a trait object implementing the `SessionInfo` trait,
    ///   which provides session-specific information such as dialect, catalog, and database.
    ///
    /// # Returns
    ///
    /// * `Result<String, QueryHandlerError>` - On success, returns a `String` containing the
    ///   result of all scanning and transpilation, ready for execution. On error, returns a `QueryHandlerError` containing
    ///   information about what went wrong during analyses and transpilation.
    pub fn handle_query(
        &mut self,
        query: &str,
        session_info: &dyn SessionInfo,
    ) -> Result<&str, QueryHandlerError> {
        // You can only handle a query once
        if let Some(ref query_result) = self.output_query {
            return Ok(query_result);
        }

        let scan_output = self.scan(query, session_info)?;
        // todo(mrhamburg): pdp has state and caching and all, needs to be improved
        let pdp = QueryPolicyDecision::new();
        let transpiler_input = pdp.from_scan_output(&scan_output)?;
        let output_query = self.transpile_query(&transpiler_input, false)?;

        self.scan_output = Some(scan_output);
        self.transpiler_input = Some(transpiler_input);
        self.output_query = Some(output_query);

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

    pub fn secure_query(&mut self) -> Result<&str, QueryHandlerError> {
        // You can only secure a query once
        if let Some(ref output_query) = self.output_query_secured {
            return Ok(output_query);
        } else if let Some(ref transpiler_input) = self.transpiler_input {
            self.output_query_secured = Some(self.transpile_query(transpiler_input, true)?);
        }

        Ok(self.output_query_secured.as_ref().unwrap())
    }
}

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
