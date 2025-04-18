use std::sync::Arc;

pub const SESSION_VARIABLE_DIALECT: &str = "proxy_dialect";
pub const SESSION_VARIABLE_CATALOG_NAME: &str = "proxy_catalog";
pub const SESSION_VARIABLE_DATABASE_NAME: &str = "proxy_database";
pub const SESSION_VARIABLE_SECURITY_IMPERSONATE: &str = "proxy_security_impersonate";
pub const SESSION_VARIABLE_SEND_TELEMETRY: &str = "proxy_send_telemetry";
pub const SESSION_VARIABLE_BRANCH_NAME: &str = "proxy_branch_name";
pub const SESSION_VARIABLE_TENANT_ID: &str = "proxy_tenant_id";
pub const SESSION_VARIABLE_DOMAIN_ID: &str = "proxy_domain_id";
pub const SESSION_VARIABLE_WORKSPACE_ID: &str = "proxy_workspace_id";
pub const SESSION_VARIABLE_DIALECT_NAME: &str = "proxy_dialect_name";

#[derive(Clone)]
pub enum SessionVariable {
    Some(Arc<str>),
    Default(Arc<str>),
    None,
}

impl SessionVariable {
    pub fn new(value: &str) -> Self {
        SessionVariable::Some(Arc::from(value))
    }

    pub fn new_default(value: &str) -> Self {
        SessionVariable::Default(Arc::from(value))
    }

    pub fn new_none() -> Self {
        SessionVariable::None
    }

    pub fn get_value_or_default(&self) -> Arc<str> {
        match self {
            SessionVariable::Some(value) => value.clone(),
            SessionVariable::Default(default_value) => default_value.clone(),
            SessionVariable::None => Arc::from(""),
        }
    }
}
