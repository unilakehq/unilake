use crate::sessions::{
    SessionVariable, SESSION_VARIABLE_BRANCH_NAME, SESSION_VARIABLE_CATALOG_NAME,
    SESSION_VARIABLE_DATABASE_NAME, SESSION_VARIABLE_DIALECT, SESSION_VARIABLE_SEND_TELEMETRY,
    SESSION_VARIABLE_TENANT_ID,
};
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use ulid::Ulid;
use unilake_common::error_code::ErrorCode;

pub struct SessionContext {
    /// Session ID
    pub(in crate::sessions) id: Ulid,
    /// Front-end session information
    pub frontend: RwLock<Box<dyn FeSessionContext>>,
    /// Back-end session information
    pub backend: RwLock<Box<dyn BeSessionContext>>,
    /// Current state of the session variables
    variables: RwLock<HashMap<String, SessionVariable>>,
}

impl SessionContext {
    pub fn new(
        id: Ulid,
        frontend: Box<dyn FeSessionContext>,
        backend: Box<dyn BeSessionContext>,
    ) -> Self {
        let default_variables = SessionContext::get_default_session_variable(&frontend, &backend);
        SessionContext {
            id,
            frontend: RwLock::new(frontend),
            backend: RwLock::new(backend),
            variables: RwLock::new(default_variables),
        }
    }

    fn set_session_variable(&self, name: String, value: SessionVariable) {
        self.variables.write().insert(name, value);
    }

    pub fn get_values_or_default<'a>(
        &self,
        default_values: &[&'a str],
        expected: bool,
    ) -> HashMap<&'a str, Arc<str>> {
        let mut values = HashMap::new();
        for v in default_values {
            let item = self.get_session_variable(v, expected);
            match item {
                None => self.report_value_not_set(*v),
                Some(item) => {
                    values.insert(*v, item);
                }
            }
        }
        values
    }

    fn report_value_not_set(&self, name: &str) {
        tracing::warn!(
            "Session variable '{}' not set for session with id {}",
            name,
            self.id
        );
    }

    pub fn get_session_variables(&self) -> HashMap<String, SessionVariable> {
        todo!()
    }

    fn get_session_variable(&self, name: &str, expected: bool) -> Option<Arc<str>> {
        if let Some(value) = self.variables.read().get(name) {
            return match value {
                SessionVariable::Some(v) | SessionVariable::Default(v) => Some(v.clone()),
                SessionVariable::None => None,
            };
        } else if expected {
            self.report_value_not_set(name);
        }
        None
    }

    fn get_default_session_variable(
        frontend: &Box<dyn FeSessionContext>,
        backend: &Box<dyn BeSessionContext>,
    ) -> HashMap<String, SessionVariable> {
        let mut variables = HashMap::new();
        variables.insert(
            SESSION_VARIABLE_CATALOG_NAME.to_string(),
            SessionVariable::new_default("default_catalog"),
        );
        variables.insert(
            SESSION_VARIABLE_DIALECT.to_string(),
            SessionVariable::new_default("tsql"),
        );
        variables.insert(
            SESSION_VARIABLE_DATABASE_NAME.to_string(),
            // todo: return back to default_schema when we have proper session variable support
            SessionVariable::new_default("dwh"),
        );
        variables.insert(
            SESSION_VARIABLE_SEND_TELEMETRY.to_string(),
            SessionVariable::new_default("false"),
        );

        frontend.set_default_variables(&mut variables);
        backend.set_default_variables(&mut variables);

        variables
    }

    pub fn quit(&self) {
        todo!()
    }

    pub fn kill(&self) {
        self.quit();
    }

    pub fn force_kill_session(&self) {
        self.force_kill_query(ErrorCode::ServerShutdown(
            "Query forcefully terminated due to server shutdown.",
        ));
        self.kill();
    }

    pub fn force_kill_query(&self, _cause: ErrorCode) {}

    pub fn get_current_catalog(&self) -> Option<Arc<str>> {
        self.get_session_variable(SESSION_VARIABLE_CATALOG_NAME, true)
    }

    pub fn set_current_catalog(&self, catalog: Arc<str>) {
        self.set_session_variable(
            SESSION_VARIABLE_CATALOG_NAME.to_string(),
            SessionVariable::Some(catalog),
        );
    }

    pub fn get_current_database(&self) -> Option<Arc<str>> {
        self.get_session_variable(SESSION_VARIABLE_DATABASE_NAME, true)
    }

    pub fn set_current_database(&self, database: Arc<str>) {
        self.set_session_variable(
            SESSION_VARIABLE_DATABASE_NAME.to_string(),
            SessionVariable::Some(database),
        );
    }

    pub fn get_current_sql_dialect(&self) -> Option<Arc<str>> {
        self.get_session_variable(SESSION_VARIABLE_DIALECT, true)
    }

    pub fn set_current_sql_dialect(&self, dialect: Arc<str>) {
        self.set_session_variable(
            SESSION_VARIABLE_DIALECT.to_string(),
            SessionVariable::Some(dialect),
        );
    }

    pub fn get_current_tenant_id(&self) -> Option<Arc<str>> {
        self.get_session_variable(SESSION_VARIABLE_TENANT_ID, true)
    }

    pub fn set_current_tenant_id(&self, tenant_id: Arc<str>) {
        self.set_session_variable(
            SESSION_VARIABLE_TENANT_ID.to_string(),
            SessionVariable::Some(tenant_id),
        );
    }

    pub fn set_current_branch_name(&self, branch_name: Arc<str>) {
        self.set_session_variable(
            SESSION_VARIABLE_BRANCH_NAME.to_string(),
            SessionVariable::Some(branch_name),
        );
    }
}

pub trait FeSessionContext: Send + Sync {
    /// Currently in use socket
    fn socket_addr(&self) -> SocketAddr;

    /// The username of the logged-in user
    fn get_username(&self) -> Option<Arc<str>>;

    /// Checks if the current user is logged in, determined by the presence of a username
    fn is_logged_in(&self) -> bool {
        self.get_username().is_some()
    }

    /// Return an any type of this instance which can be downcast to the specific type
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn set_default_variables(&self, variables: &mut HashMap<String, SessionVariable>);
}

pub trait BeSessionContext: Send + Sync {
    /// Return an any type of this instance which can be downcast to the specific type
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn set_default_variables(&self, variables: &mut HashMap<String, SessionVariable>);
}
