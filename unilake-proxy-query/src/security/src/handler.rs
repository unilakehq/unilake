// todo: https://github.com/notken12/licensesnip
// todo: when auditing, we also need number of records processed, processing time and total time in proxy. We will do this as a seperate message (QueryTelemetry and use query id to link to this event)

use crate::adapter::cached_adapter::CachedAdapter;
use crate::effector::PdpEffector;
use crate::functions::add_functions;
use crate::policies::{
    PolicyCollectResult, PolicyFound, PolicyHitManager, PolicyLogger, PolicyType,
};
use crate::repository::{CacheContainer, RepoBackend};
use crate::{HitRule, ABAC_MODEL};
use casbin::{Cache, CachedEnforcer, CoreApi, DefaultModel};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use ulid::Ulid;
use unilake_common::error::{TdsWireError, TokenError};
use unilake_common::model::{
    AccessPolicyModel, EntityAttributeModel, EntityModel, GroupModel, SessionModel, UserModel,
};
use unilake_common::settings::settings_server_name;
use unilake_sql::{
    run_scan_operation, run_secure_operation, run_transpile_operation, Catalog, ParserError,
    PolicyAccessRequestUrl, ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject,
    TranspilerDenyCause, TranspilerInput, TranspilerInputFilter, TranspilerInputRule,
    VisibleSchemaBuilder,
};

const SELECT: &str = "SELECT";

pub enum HandleResult {
    /// The query that should be executed
    Query(Arc<str>),
    /// Contains causes and request urls
    AccessDenied(
        Vec<TranspilerDenyCause>,
        Option<Vec<PolicyAccessRequestUrl>>,
    ),
}

#[derive(Debug)]
pub enum SecurityHandlerResult {
    /// In case we cannot find an entity from the attribute
    EntityNotFoundFromAttribute(String),
    /// Happens when a requested entity <catalog>.<schema>.<entity> does not exist
    EntityNotFound(String),
    /// Happens when a requested entity <catalog>.<schema>.<entity> exists but is not allowed to be accessed
    EntityNotAllowed(String),
    /// Happens when the user groups cannot be found
    UserGroupsNotFound(String),
    /// Happens when the user cannot be found
    UserNotFound(String),
    /// Happens when there are issues with the policy being used
    PolicyError(String),
    /// Happens when the cache is in an invalid state, retry the process. Returns the current iteration count
    InvalidCacheError,
    /// Happens when the iteration limit is reached for processing the security checks
    IterationLimitReached(usize),
}

pub struct SecurityError {
    message: String,
    audit_only: bool,
}

pub enum SecurityHandlerError {
    /// Error code, TdsWireError
    WireError(u32, TdsWireError),
    /// Error code, Query Id, ParserError
    QueryError(u32, String, ParserError),
    /// Error code, Query Id, SecurityError
    SecurityError(u32, String, SecurityError),
}

impl From<SecurityHandlerError> for TokenError {
    fn from(value: SecurityHandlerError) -> Self {
        match value {
            SecurityHandlerError::WireError(code, e) => TokenError {
                code,
                line: 0,
                message: e.to_string(),
                class: 0,
                procedure: "".to_string(),
                server: settings_server_name(),
                state: 0,
            },
            SecurityHandlerError::QueryError(code, id, e) => {
                if let Some(err) = e.errors.first() {
                    return TokenError {
                        code,
                        message: format!(
                            "{}. Line: {}, Col: {}. {}. Query Id: {}",
                            err.start_context, err.line, err.col, err.description, id
                        ),
                        class: 0,
                        line: err.line,
                        procedure: "".to_string(),
                        server: settings_server_name(),
                        state: 1,
                    };
                }
                TokenError {
                    code,
                    line: 0,
                    message: format!("Parser error: {}. Query Id: {}", e.message, id),
                    class: 0,
                    procedure: "".to_string(),
                    server: settings_server_name(),
                    state: 0,
                }
            }
            SecurityHandlerError::SecurityError(code, id, s) => match s.audit_only {
                true => TokenError {
                    code,
                    line: 0,
                    message: format!(
                        "Unable to process query, check logs for more details. Query Id: {}",
                        id
                    ),
                    class: 0,
                    procedure: "".to_string(),
                    server: settings_server_name(),
                    state: 0,
                },
                false => TokenError {
                    code,
                    line: 0,
                    message: format!("{}. Query Id: {}", s.message, id),
                    class: 0,
                    procedure: "".to_string(),
                    server: settings_server_name(),
                    state: 0,
                },
            },
        }
    }
}

pub struct SecurityHandler {
    query_id: Ulid,
    scan_output: Option<ScanOutput>,
    transpiler_input: Option<TranspilerInput>,
    output_query: Option<Arc<str>>,
    output_query_secured: Option<Arc<str>>,
    input_query_secured: Option<Arc<str>>,
    input_query: Option<Arc<str>>,
    session_model: SessionModel,
    cached_adapter: Option<CachedAdapter>,
    cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    cached_backend: CacheContainer,
    repo_backend: Box<dyn RepoBackend>,
    abac_model: Option<DefaultModel>,
}

impl SecurityHandler {
    pub fn new(
        cached_adapter: CachedAdapter,
        session_model: SessionModel,
        cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
        cached_backend: CacheContainer,
        repo_backend: Box<dyn RepoBackend>,
        abac_model: Option<DefaultModel>,
    ) -> Self {
        SecurityHandler {
            query_id: Ulid::new(),
            scan_output: None,
            transpiler_input: None,
            output_query: None,
            output_query_secured: None,
            input_query_secured: None,
            input_query: None,
            cached_adapter: Some(cached_adapter),
            session_model,
            cached_rules,
            cached_backend,
            repo_backend,
            abac_model,
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
    ) -> Result<ScanOutput, SecurityHandlerError> {
        Ok(
            run_scan_operation(query, dialect, catalog, database).map_err(|e| {
                SecurityHandlerError::WireError(90102, TdsWireError::Input(e.to_string()))
            })?,
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
    /// * `dialect` - A reference to a string containing the dialect of the current session.
    /// * `catalog` - A reference to a string containing the catalog of the current session.
    /// * `database` - A reference to a string containing the database of the current session.
    ///
    /// # Returns
    ///
    /// * `Result<String, QueryHandlerError>` - On success, returns a `String` containing the
    ///   result of all scanning and transpilation, ready for execution. On error, returns a `QueryHandlerError` containing
    ///   information about what went wrong during analyses and transpilation.
    pub async fn handle_query(
        &mut self,
        query: &str,
        dialect: &str,
        catalog: &str,
        database: &str,
    ) -> Result<HandleResult, SecurityHandlerError> {
        // You can only handle a query once
        if let Some(ref query_result) = self.output_query {
            return Ok(HandleResult::Query(query_result.clone()));
        }

        self.input_query = Some(Arc::from(query.to_string()));
        let scan_output = self.scan(query, dialect, catalog, database)?;
        if let Some(error) = scan_output.error {
            self.close_handler();
            return Err(SecurityHandlerError::QueryError(
                90_200,
                self.query_id.to_string(),
                error,
            ));
        }

        // check access on entity and action level
        if !self.check_user_access(&scan_output).await {
            // todo: return information on user, entity and action combination not being allowed
            // make use of entitynotallowed?
            // Something like: Access to entity {} with action {} not allowed. Query ID: {}
        }

        let mut iterations = 2;
        let transpiler_input: Option<TranspilerInput>;
        loop {
            iterations -= 1;
            if iterations == 0 {
                tracing::error!(
                    "Failed to provide transpiler input for query with id: {}",
                    self.query_id
                );

                self.close_handler();
                return Err(
                    self.handle_error(SecurityHandlerResult::IterationLimitReached(iterations))
                );
            }

            match QueryPolicyDecision::new(
                &self.cached_backend,
                self.cached_adapter.take(),
                self.abac_model.take(),
                &self.session_model,
                self.cached_rules.clone(),
                &self.repo_backend,
            )
            .process(&scan_output)
            .await
            {
                Ok(ti) => {
                    transpiler_input = Some(ti);
                    break;
                }
                Err(e) => match e {
                    SecurityHandlerResult::InvalidCacheError => {
                        continue;
                    }
                    _ => {
                        tracing::error!(
                            "Error occurred while processing query policy decision: {:?}",
                            e
                        );

                        self.close_handler();
                        return Err(self.handle_error(e));
                    }
                },
            }
        }

        // unwrap: we can only get here if the above loop was successful
        let transpiler_input = transpiler_input.unwrap();

        // check for any security violations
        if let Some(cause) = transpiler_input.cause {
            self.close_handler();
            return Ok(HandleResult::AccessDenied(
                cause,
                transpiler_input.request_url,
            ));
        }
        let output_query = self.transpile_query(&transpiler_input, false)?;

        self.scan_output = Some(scan_output);
        self.transpiler_input = Some(transpiler_input);
        self.output_query = Some(Arc::from(output_query));

        Ok(HandleResult::Query(
            self.output_query.as_ref().unwrap().clone(),
        ))
    }

    /// Closes this queryhandler making sure that it cannot be reused.
    fn close_handler(&mut self) {
        self.output_query = Some(Arc::from("".to_string()));
    }

    /// Properly handle the security handler error
    fn handle_error(&mut self, error: SecurityHandlerResult) -> SecurityHandlerError {
        let (error_code, error) = match error {
            SecurityHandlerResult::EntityNotFoundFromAttribute(e) => (90300, SecurityError {
                message: format!("Could not find entity from attribute: {}.", e),
                audit_only: false,
            }),
            SecurityHandlerResult::EntityNotFound(e) => (90301, SecurityError {
                message: format!("Could not find requested entity: {}", e),
                audit_only: false,
            }),
            SecurityHandlerResult::EntityNotAllowed(e) => (90302, SecurityError {
                message: format!("Access to entity {} not allowed", e),
                audit_only: false,
            }),
            SecurityHandlerResult::UserGroupsNotFound(e) => (90303, SecurityError {
                message: format!("Could not find user groups for user: {}", e),
                audit_only: true,
            }),
            SecurityHandlerResult::UserNotFound(e) => (90304, SecurityError {
                message: format!("Could not find user: {}", e),
                audit_only: false,
            }),
            SecurityHandlerResult::PolicyError(e) => (90305, SecurityError {
                message: format!("Policy error: {}", e),
                audit_only: true,
            }),
            SecurityHandlerResult::InvalidCacheError => (90306, SecurityError {
                message: "Invalid cache error, corrupted cache?".to_string(),
                audit_only: true,
            }),
            SecurityHandlerResult::IterationLimitReached(e) => (90307, SecurityError {
                message: format!("Iteration limit of {} reached. Could not processes query correctly, please check logs.", e),
                audit_only: true,
            }),
        };
        SecurityHandlerError::SecurityError(error_code, self.query_id.to_string(), error)
    }

    /// Executes the transpile operation for transpiling an input query to an allowed executable SQL query.
    fn transpile_query(
        &self,
        scanned: &TranspilerInput,
        secure_output: bool,
    ) -> Result<String, SecurityHandlerError> {
        let transpiler_output = run_transpile_operation(scanned, secure_output).map_err(|e| {
            SecurityHandlerError::WireError(90101, TdsWireError::Input(e.to_string()))
        })?;
        if let Some(error) = transpiler_output.error {
            return Err(SecurityHandlerError::QueryError(
                90201,
                self.query_id.to_string(),
                error,
            ));
        }
        Ok(transpiler_output.sql_transformed)
    }

    /// Secure the generated output query by removing any sensitive information.
    pub fn secure_output_query(&mut self) -> Result<&str, SecurityHandlerError> {
        // You can only secure a query once
        if let Some(ref output_query_secured) = self.output_query_secured {
            return Ok(output_query_secured);
        } else if let Some(ref transpiler_input) = self.transpiler_input {
            self.output_query_secured =
                Some(Arc::from(self.transpile_query(transpiler_input, true)?));
        }

        Ok(self.output_query_secured.as_ref().unwrap())
    }

    /// Secure the input query by removing any sensitive information.
    pub fn secure_input_query(&mut self) -> Result<&str, SecurityHandlerError> {
        // You can only secure an input query once
        if let Some(ref input_query_secured) = self.input_query_secured {
            return Ok(input_query_secured);
        }

        self.input_query_secured = Some(Arc::from(
            run_secure_operation(self.input_query.as_ref().unwrap().as_ref()).map_err(|e| {
                SecurityHandlerError::WireError(90100, TdsWireError::Protocol(e.to_string()))
            })?,
        ));
        Ok(self.input_query_secured.as_ref().unwrap())
    }

    /// Returns the unique identifier of the current query.
    pub fn get_query_id(&self) -> Ulid {
        self.query_id
    }

    /// Checks if the current user has access to the entity involved with the given intent.
    async fn check_user_access(&self, scan_output: &ScanOutput) -> bool {
        if scan_output.query_type == SELECT {
            return true;
        }

        let (catalog, schema, table) = scan_output.get_full_path_names();
        let table = if let Some(table) = table {
            Some(table.to_owned())
        } else {
            None
        };

        // check if user has access to the entity involved with the given intent (gravitino api, select|update|delete|create|modify)
        // we handle select, create|update|modify|delete intents are done by gravitino api
        let result = self
            .repo_backend
            .get_access_by_action(
                catalog.unwrap_or("").to_string(),
                schema.unwrap_or("").to_string(),
                table.clone(),
                scan_output.query_type.clone(),
            )
            .await;

        result.unwrap_or_else(|e| {
            tracing::error!(
                e = e,
                action = scan_output.query_type,
                catalog = catalog,
                schema = schema,
                table = table,
                "Failed to check user access: {}",
                e
            );
            false
        })
    }
}

#[derive(PartialEq, Clone)]
enum AttributeAccess {
    Allowed,
    Hidden,
    Denied,
}

struct QueryPolicyDecision<'a> {
    /// Container for cached model input (entity model, user mode, group model, etc..)
    cached_backend: &'a CacheContainer,
    cached_adapter: Option<CachedAdapter>,
    abac_model: Option<DefaultModel>,
    /// Current session based information, required as context for the abac model
    session_model: &'a SessionModel,
    /// Policy hit cache, caches policy hits so we maintain context (works together with the enforcer)
    policy_hit_cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    /// Repository backend for interacting with the database and other data sources
    repo_backend: &'a Box<dyn RepoBackend>,
}

impl<'a> QueryPolicyDecision<'a> {
    pub fn new(
        cached_backend: &'a CacheContainer,
        cached_adapter: Option<CachedAdapter>,
        abac_model: Option<DefaultModel>,
        session_model: &'a SessionModel,
        policy_hit_cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
        repo_backend: &'a Box<dyn RepoBackend>,
    ) -> Self {
        QueryPolicyDecision {
            cached_backend,
            session_model,
            policy_hit_cache,
            repo_backend,
            cached_adapter,
            abac_model,
        }
    }

    fn get_object_model<'b>(
        &self,
        entity_model: &'b EntityModel,
        attribute_full_path_name: &str,
    ) -> Option<&'b EntityAttributeModel> {
        entity_model.attributes.get(attribute_full_path_name)
    }

    pub async fn process(
        &mut self,
        scan_output: &ScanOutput,
    ) -> Result<TranspilerInput, SecurityHandlerResult> {
        let abac_model = if let Some(abac_model) = self.abac_model.take() {
            // prefer to get it from the supplied value (quicker)
            abac_model
        } else {
            tracing::debug!("Recreating abac model - not preferred");
            DefaultModel::from_str(ABAC_MODEL).await.unwrap()
        };

        let mut enforcer = CachedEnforcer::new(abac_model, self.cached_adapter.take().unwrap())
            .await
            .unwrap();

        // set possible output
        let mut cause: Option<Vec<TranspilerDenyCause>> = None;
        let mut request_url: Option<Vec<PolicyAccessRequestUrl>> = None;

        // set input (user and group)
        let (user_model, group_model, impersonate_user_model, impersonate_group_model) =
            self.get_user_and_group_models().await?;

        // set input user policies
        let mut policies = BTreeMap::new();
        self.get_user_policies(&user_model, &mut policies).await?;
        if let Some(ref impersonate_user_model) = impersonate_user_model {
            self.get_user_policies(impersonate_user_model, &mut policies)
                .await?;
        }

        // input for enforcing policies
        let mut impersonate_session_model: Option<SessionModel> = None;
        let mut policy_enforce_context =
            [Some((&user_model, &group_model, self.session_model)), None];
        if self.session_model.impersonate_user_id.is_some() {
            let mut session_model = self.session_model.clone();
            session_model.user_id = session_model.impersonate_user_id.clone().unwrap();
            impersonate_session_model = Some(session_model);

            policy_enforce_context[1] = Some((
                impersonate_user_model.as_ref().unwrap(),
                impersonate_group_model.as_ref().unwrap(),
                impersonate_session_model.as_ref().unwrap(),
            ));
        }

        // walk through all scopes and attributes
        let mut masking_rules = Vec::new();
        let mut filter_rules = Vec::new();
        let mut entities = HashMap::new();
        let mut exclude_attributes_visible_map = HashSet::new();

        // set enforcer dependencies
        let effector = Box::new(PdpEffector);
        enforcer.set_effector(effector);
        add_functions(enforcer.get_mut_engine());

        // get all entity attributes by scope, for processing
        for (scope, ref mut items) in scan_output.objects.iter().map(|objects| {
            (
                objects.scope,
                Self::get_entity_attributes(&scan_output.objects),
            )
        }) {
            // set new logger instance (logger is scoped to a (query)scope)
            let mut pm = PolicyHitManager::new(self.policy_hit_cache.clone());
            let logger = Box::new(PolicyLogger::new(pm.get_sender()));

            // set logger to enforcer
            enforcer.set_logger(logger);
            enforcer.enable_log(true);

            // unpack stars
            let mut star_map = HashMap::new();
            for entity in items.iter().map(|(e, att, _)| {
                e.ok_or_else(|| {
                    SecurityHandlerResult::EntityNotFoundFromAttribute(att.get_name().to_owned())
                })
            }) {
                let entity = entity?;
                let found = self
                    .get_star_attributes(
                        &self.cached_backend,
                        entity.get_full_name(),
                        &entity.alias,
                    )
                    .await?;

                star_map.insert(entity.get_full_name(), found.1);
                entities.insert(entity.get_full_name(), found.0);
            }
            Self::fill_star_attributes(items, &star_map).await?;

            // process all attributes for policy enforcement
            let mut results: HashMap<&str, bool> = HashMap::new();
            for (scan_entity, attribute, is_starred) in items.iter() {
                let scan_entity = scan_entity.ok_or_else(|| {
                    SecurityHandlerResult::EntityNotFoundFromAttribute(
                        attribute.get_name().to_owned(),
                    )
                })?;
                let entity_name = scan_entity.get_full_name();
                let entity_model = entities.get(&entity_name).unwrap();
                let entity_attribute_name = format!("{}.{}", entity_name, attribute.name);

                // get input object model
                let object_model = match self.get_object_model(entity_model, &entity_attribute_name)
                {
                    None => {
                        return Err(SecurityHandlerResult::EntityNotFound(entity_attribute_name));
                    }
                    Some(om) => om,
                };

                // check access based on policy
                for context in policy_enforce_context {
                    if let Some((user_model, group_model, session_model)) = context {
                        match enforcer.enforce((
                            user_model,
                            group_model,
                            session_model,
                            &object_model,
                            &policies,
                        )) {
                            Ok(status) => {
                                if !status && !*is_starred {
                                    // explicitly requested entity attribute
                                    // will be handled later (when processing policies found)
                                    results.insert(object_model.id.as_ref(), false);
                                } else if !status {
                                    // implicitly requested starred entity attribute
                                    exclude_attributes_visible_map
                                        .insert(entity_attribute_name.clone());
                                    results.insert(object_model.id.as_ref(), true);
                                } else {
                                    results.insert(object_model.id.as_ref(), true);
                                }
                            }
                            Err(err) => {
                                panic!("Failed to enforce policy: {}", err)
                            }
                        }
                    }
                }
            }

            // this will process the results and gather their associated policies
            let policies_found = match pm
                .process_hits()
                .map_err(|e| SecurityHandlerResult::PolicyError(e))?
            {
                PolicyCollectResult::Found(f) => f,
                PolicyCollectResult::CacheInvalid => {
                    self.policy_hit_cache.clear();
                    return Err(SecurityHandlerResult::InvalidCacheError);
                }
                PolicyCollectResult::NotFound => {
                    // todo: this can also be a valid result, for example "select 1", we need to handle this scenario and a combination of expressions and literals correctly
                    return Err(SecurityHandlerResult::PolicyError(
                        "Could not find any policy results, are the policies loaded correctly?"
                            .to_string(),
                    ));
                }
            };

            let object_models = entities
                .values()
                .flat_map(|e| &e.attributes)
                .map(|(_, v)| v)
                .collect();

            for (object_id, rules) in policies_found {
                // check if we should prioritize stricter rules over less strict rules
                // in case this is an impersonate session, we always prioritize stricter rules
                let prio_stricter = self
                    .check_policy_rules_prio(&rules, impersonate_session_model.is_some())
                    .await?;

                let (object_name, att_name, from_star) =
                    Self::find_star_and_attribute_name(items, &object_models, &object_id)?;

                // check if there are any deny rules
                if !results.get(object_id.as_str()).unwrap_or(&false) {
                    let policy_id = Self::get_causing_policy(&rules).map(|r| r.policy_id.as_str());
                    self.set_deny_access_cause(&mut cause, scope, object_id.as_str(), policy_id);
                    continue;
                }

                // add masking rule
                if let Some(rule) =
                    PolicyHitManager::resolve_masking_policy_conflict(&rules, prio_stricter)
                {
                    match Self::get_attribute_access_status(rule, from_star) {
                        AttributeAccess::Allowed => masking_rules.push((
                            scope,
                            object_id.clone(),
                            att_name.clone(),
                            rule.clone(),
                        )),
                        AttributeAccess::Hidden => {
                            exclude_attributes_visible_map.insert(object_name.clone());
                            // no need to process filters, if we are not allowed to see this item.
                            // in case you need to filter an attribute and not allow to this attribute to be shown, you should use a different mask (replace_null, for example)
                            continue;
                        }
                        AttributeAccess::Denied => {
                            self.set_deny_access_cause(
                                &mut cause,
                                scope,
                                object_id.as_str(),
                                Some(rule.policy_id.as_str()),
                            );
                        }
                    }
                }

                // add all filter rules
                let filter_rule = rules
                    .iter()
                    .filter(|p| p.policy_type == PolicyType::FilterRule);

                filter_rule.for_each(|p| {
                    filter_rules.push((scope, object_id.clone(), att_name.clone(), p.clone()))
                });
            }
        }

        // get request url
        if let Some(ref cause) = cause {
            request_url = Some(self.get_deny_access_url(cause).await?);
        }

        // prepare transpiler input
        let entities = entities.into_iter().map(|(_, v)| v).collect();
        let query = match scan_output.query.as_ref() {
            None => {
                return Err(SecurityHandlerResult::PolicyError(
                    "Query not found".to_owned(),
                ))
            }
            Some(query) => query,
        };

        Ok(TranspilerInput {
            cause,
            request_url,
            query: query.clone(),
            rules: masking_rules
                .iter()
                .map(|(scope, att_id, att_name, rule)| TranspilerInputRule {
                    scope: *scope,
                    attribute_id: att_id.to_owned(),
                    attribute: att_name.to_owned(),
                    policy_id: rule.policy_id.to_owned(),
                    rule_definition: rule.rule_definition.clone(),
                })
                .collect(),
            filters: filter_rules
                .iter()
                .map(|(scope, att_id, att_name, rule)| TranspilerInputFilter {
                    scope: *scope,
                    attribute_id: att_id.to_owned(),
                    attribute: att_name.to_owned(),
                    policy_id: rule.policy_id.to_owned(),
                    filter_definition: rule.rule_definition.clone(),
                })
                .collect(),
            visible_schema: Self::get_visible_schema(entities, exclude_attributes_visible_map),
        })
    }

    async fn get_user_policies(
        &self,
        user_model: &UserModel,
        policies: &mut BTreeMap<String, AccessPolicyModel>,
    ) -> Result<(), SecurityHandlerResult> {
        for policy_id in user_model.access_policy_ids.iter() {
            let found = self
                .cached_backend
                .access_policy_model
                .get(policy_id)
                .await
                .ok_or_else(|| {
                    SecurityHandlerResult::PolicyError(format!(
                        "Could not find policy: {}",
                        policy_id
                    ))
                })?;
            policies.insert(found.normalized_name.clone(), found);
        }

        Ok(())
    }

    async fn get_user_and_group_models(
        &self,
    ) -> Result<(UserModel, GroupModel, Option<UserModel>, Option<GroupModel>), SecurityHandlerResult>
    {
        let user_model = self.get_user_model(&self.session_model.user_id).await?;
        let group_model = self.get_group_model(&self.session_model.user_id).await?;

        let (impersonate_user_model, impersonate_group_model) =
            if let Some(ref user_id) = self.session_model.impersonate_user_id {
                (
                    Some(self.get_user_model(user_id).await?),
                    Some(self.get_group_model(user_id).await?),
                )
            } else {
                (None, None)
            };

        Ok((
            user_model,
            group_model,
            impersonate_user_model,
            impersonate_group_model,
        ))
    }

    async fn get_user_model(&self, user_id: &String) -> Result<UserModel, SecurityHandlerResult> {
        self.cached_backend
            .user_model
            .get(user_id)
            .await
            .ok_or_else(|| {
                SecurityHandlerResult::UserNotFound(self.session_model.user_id.to_owned())
            })
    }

    async fn get_group_model(&self, user_id: &String) -> Result<GroupModel, SecurityHandlerResult> {
        self.cached_backend
            .group_model
            .get(user_id)
            .await
            .ok_or_else(|| SecurityHandlerResult::UserGroupsNotFound(user_id.to_owned()))
    }

    /// Checks if a stricter policy is preferred based on the policy rules found
    /// defaults to strict, if no prio strict is found, will return Ok(true)
    async fn check_policy_rules_prio(
        &self,
        policies: &[PolicyFound],
        is_impersonation: bool,
    ) -> Result<bool, SecurityHandlerResult> {
        if is_impersonation {
            return Ok(true); // always prioritize impersonate rules over regular rules
        }
        let ids: HashSet<_> = policies.iter().map(|p| &p.policy_id).collect();
        for policy_id in ids {
            let found = self.cached_backend.access_policy_model.get(policy_id).await;
            match found {
                None => {
                    return Err(SecurityHandlerResult::PolicyError(format!(
                        "Could not find policy with id: {}",
                        policy_id
                    )))
                }
                Some(p) => {
                    if !p.prio_strict {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    /// Gets the full object name, attribute name and whether it's a starred attribute
    fn find_star_and_attribute_name(
        entities_and_attributes: &[(Option<&ScanEntity>, &ScanAttribute, bool)],
        objects: &Vec<&EntityAttributeModel>,
        object_id: &str,
    ) -> Result<(String, String, bool), SecurityHandlerResult> {
        let object_model = objects
            .iter()
            .find(|object| object.id == object_id)
            .ok_or_else(|| {
                SecurityHandlerResult::PolicyError(
                    "Could not find object in entity object model".to_owned(),
                )
            })?;

        let (_, attribute, star) = entities_and_attributes
            .iter()
            .find_map(|(entity, attr, star)| {
                if let Some(entity) = entity {
                    let full_name = format!("{}.{}", entity.get_full_name(), attr.name);
                    if full_name == object_model.full_name {
                        return Some((entity, attr, star));
                    }
                }
                None
            })
            .ok_or_else(|| {
                SecurityHandlerResult::PolicyError(
                    "Could not find object attributes in entity object model".to_owned(),
                )
            })?;

        Ok((object_model.full_name.clone(), attribute.get_name(), *star))
    }

    fn set_deny_access_cause(
        &self,
        cause: &mut Option<Vec<TranspilerDenyCause>>,
        scope: i32,
        attribute: &str,
        policy_id: Option<&str>,
    ) -> () {
        if cause.is_none() {
            cause.replace(Vec::new());
        }

        // unwrap is safe here because we've made sure the vector exists (see above)
        cause.as_mut().unwrap().push(TranspilerDenyCause {
            scope,
            attribute: attribute.to_owned(),
            policy_id: policy_id.map(|a| a.to_owned()),
        });
    }

    /// Gets the request urls for each deny access cause policy (1 request url per policy)
    async fn get_deny_access_url(
        &self,
        cause: &Vec<TranspilerDenyCause>,
    ) -> Result<Vec<PolicyAccessRequestUrl>, SecurityHandlerResult> {
        let policy_ids: HashSet<_> = cause
            .iter()
            .map(|c| &c.policy_id)
            .filter_map(|i| i.as_ref())
            .collect();
        let mut requests = Vec::new();
        for policy_id in policy_ids {
            let found = self
                .repo_backend
                .generate_access_request(
                    self.session_model.workspace_id.clone(),
                    self.session_model.user_id.clone(),
                    policy_id.clone(),
                )
                .await
                .map_err(|e| {
                    SecurityHandlerResult::PolicyError(format!(
                        "Could not get policy access request due to error: {}",
                        e
                    ))
                })?;

            requests.push(PolicyAccessRequestUrl {
                url: found.url,
                message: found.message,
            })
        }

        Ok(requests)
    }

    /// Extracts the combinations entity and attribute from the given (ScanOutput) scope
    /// ## Returned tuple:
    /// - Optional Entity,
    /// - Attribute,
    /// - Star flag (defaults false)
    fn get_entity_attributes(
        input: &Vec<ScanOutputObject>,
    ) -> Vec<(Option<&ScanEntity>, &ScanAttribute, bool)> {
        input
            .iter()
            .flat_map(|i| {
                i.attributes
                    .iter()
                    .filter_map(|a| {
                        i.entities
                            .iter()
                            .find(|e| e.alias == a.entity_alias)
                            .map(|e| (Some(e), a, false))
                            .or_else(|| Some((None, a, false)))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Searches for star entities and replaces them with their known attributes
    async fn fill_star_attributes<'b>(
        items: &mut Vec<(Option<&ScanEntity>, &'b ScanAttribute, bool)>,
        found_attributes: &'b HashMap<String, Vec<ScanAttribute>>,
    ) -> Result<(), SecurityHandlerResult> {
        let mut star_attributes = Vec::new();
        if let Some(item) = items.iter_mut().find(|(_, att, _)| att.name == "*") {
            let (entity, _, _) = item;
            // create new elements based on the matching ones
            if let Some(entity) = entity {
                let full_name = entity.get_full_name();
                match found_attributes.get(&full_name) {
                    None => {
                        return Err(SecurityHandlerResult::EntityNotFound(full_name));
                    }
                    Some(found) => {
                        star_attributes.extend(
                            found
                                .iter()
                                .map(|i| (Some(*entity), i, true))
                                .collect::<Vec<_>>(),
                        );
                    }
                }
            }
        }

        // extend the original list with the new elements
        items.extend(star_attributes);

        // remove the original elements with the "*" attribute
        items.retain(|(_, attr, _)| attr.name != "*");
        Ok(())
    }

    fn get_visible_schema(
        entities: Vec<EntityModel>,
        exclude: HashSet<String>,
    ) -> Option<HashMap<String, Catalog>> {
        let mut builder = VisibleSchemaBuilder::new();
        for model in entities {
            let table = builder
                .get_or_add_catalog(model.get_catalog_name()?)
                .get_or_add_database(model.get_schema_name()?)
                .get_or_add_table(model.get_table_name()?);
            for (n, t) in &model.attributes {
                if !exclude.contains(n) {
                    table.get_or_add_column(t.name.to_owned(), t.data_type.to_owned());
                }
            }
        }
        Some(builder.catalog)
    }

    /// Requires the full entity name <catalog>.<schema>.<entity>/<fileset> and gets all its attributes
    /// uses these attributes to create an entity attribute mapping. Entity alias is required
    /// since we need to know which attributes belong to which entity within a given scope
    async fn get_star_attributes(
        &self,
        cached_backend: &CacheContainer,
        full_entity_name: String,
        entity_alias: &str,
    ) -> Result<(EntityModel, Vec<ScanAttribute>), SecurityHandlerResult> {
        if let Some(entity) = cached_backend.entity_model.get(&full_entity_name).await {
            let mut attributes = Vec::new();
            for (full_entity_name, _) in &entity.attributes {
                if let Some(obj) = self.get_object_model(&entity, &full_entity_name) {
                    attributes.push(ScanAttribute {
                        // todo: check if this works with spaces in the name
                        entity_alias: entity_alias.to_owned(),
                        name: obj.name.to_owned(),
                        alias: obj.name.to_owned(),
                    });
                } else {
                    return Err(SecurityHandlerResult::EntityNotFound(
                        full_entity_name.to_owned(),
                    ));
                }
            }

            Ok((entity, attributes))
        } else {
            Err(SecurityHandlerResult::EntityNotFound(full_entity_name))
        }
    }

    /// Check if we are allowed to access the given object attribute
    ///
    /// - Allowed: we can further process this attribute as is and use it in the visible schema
    /// - Hidden: we need to hide this attribute from the visible schema
    /// - Denied: we are trying to access an attribute we don't have access to
    fn get_attribute_access_status(policy: &PolicyFound, from_star: bool) -> AttributeAccess {
        if policy.get_name() == "hidden" {
            return if from_star {
                AttributeAccess::Hidden
            } else {
                AttributeAccess::Denied
            };
        }
        AttributeAccess::Allowed
    }

    /// Based on policies found, returns the most likely cause for denying access
    fn get_causing_policy(policies_found: &[PolicyFound]) -> Option<&PolicyFound> {
        policies_found
            .iter()
            .find(|p| p.effect == "deny")
            .or_else(|| policies_found.iter().find(|p| p.effect == "approve"))
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::cached_adapter::{CachedAdapter, CachedPolicyRules};
    use crate::caching::layered_cache::{BackendProvider, MultiLayeredCache};
    use crate::handler::{CacheContainer, QueryPolicyDecision, SecurityHandlerResult};
    use crate::repository::RepoBackend;
    use crate::{HitRule, ABAC_MODEL};
    use async_trait::async_trait;
    use casbin::{Cache, DefaultCache, DefaultModel};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::sync::Arc;
    use unilake_common::model::{
        AccessPolicyModel, AppInfoModel, DataAccessRequestResponse, EntityAttributeModel,
        EntityModel, GroupInstance, GroupModel, IpInfoModel, PolicyRule, SessionModel, UserModel,
    };
    use unilake_sql::{ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject, TranspilerInput};

    async fn run_default_test(
        rules: Vec<PolicyRule>,
        scan_output: Option<ScanOutput>,
        user_model_items: Option<HashMap<String, UserModel>>,
        group_model_items: Option<HashMap<String, GroupModel>>,
        entity_model_items: Option<HashMap<String, EntityModel>>,
        policy_model_items: Option<HashMap<String, AccessPolicyModel>>,
        session_model_input: Option<SessionModel>,
    ) -> Result<TranspilerInput, SecurityHandlerResult> {
        // get all defaults
        let (abac_model, default_scan_output, cache_container) = get_defaults(
            user_model_items,
            group_model_items,
            entity_model_items,
            policy_model_items,
        )
        .await;
        let (_, adapter) = get_default_policy_cache(rules);
        let session_model = if let Some(session_model_input) = session_model_input {
            session_model_input
        } else {
            get_session_model_input()
        };
        let policy_cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>> =
            Arc::new(Box::new(DefaultCache::new(10)));

        // set repo backend
        let fake_backend: Box<dyn RepoBackend> = Box::new(FakeRepoBackend {});

        // set sut
        let mut sut = QueryPolicyDecision::new(
            &cache_container,
            Some(adapter),
            Some(abac_model),
            &session_model,
            policy_cache.clone(),
            &fake_backend,
        );

        // process results
        let scan_output = if scan_output.is_some() {
            scan_output.unwrap()
        } else {
            default_scan_output
        };
        sut.process(&scan_output).await
    }

    #[tokio::test]
    async fn test_query_policy_decision_full_access() {
        let result =
            run_default_test(get_default_policy(), None, None, None, None, None, None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.filters.is_empty());
        assert_eq!(0, result.rules.len())
    }

    #[tokio::test]
    async fn test_query_policy_decision_masking_impersonation_access() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.customers.firstname",
                "TagExists(r.user, \"test::user_2\")",
                "allow",
                // {"name": "replace_null"}
                "eyJuYW1lIjogInJlcGxhY2VfbnVsbCJ9",
                "masked_id",
            ),
        ];

        let mut policy_models = get_policy_model_input();
        policy_models.insert(
            "masked_id".to_string(),
            AccessPolicyModel {
                policy_id: "masked_id".to_string(),
                prio_strict: true,
                expire_datetime_utc: 0,
                normalized_name: "masked_id".to_string(),
            },
        );

        let mut user_model = get_user_model_input();
        user_model.insert(
            "user_id_2".to_owned(),
            UserModel {
                id: "user_id_2".to_string(),
                principal_name: "".to_string(),
                roles: vec![],
                tags: vec!["test::user_2".to_string()],
                access_policy_ids: vec!["policy_id".to_string()],
            },
        );

        let mut session_model = get_session_model_input();
        session_model.impersonate_user_id = Some("user_id_2".to_owned());

        let mut group_model = get_group_model_input();
        group_model.insert(
            "user_id_2".to_owned(),
            GroupModel {
                user_id: "user_id_2".to_string(),
                groups: vec![],
            },
        );

        let result = run_default_test(
            policies,
            None,
            Some(user_model),
            Some(group_model),
            None,
            Some(policy_models),
            Some(session_model),
        )
        .await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.filters.is_empty());
        assert_eq!(result.rules.len(), 1);
    }

    #[tokio::test]
    async fn test_query_policy_decision_one_masked_column_access() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::id\")",
                "allow",
                // {"name": "replace_null"}
                "eyJuYW1lIjogInJlcGxhY2VfbnVsbCJ9",
                "masked_id",
            ),
        ];

        let mut policy_models = get_policy_model_input();
        policy_models.insert(
            "masked_id".to_string(),
            AccessPolicyModel {
                policy_id: "masked_id".to_string(),
                prio_strict: true,
                expire_datetime_utc: 0,
                normalized_name: "masked_id".to_string(),
            },
        );

        let result =
            run_default_test(policies, None, None, None, None, Some(policy_models), None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.visible_schema.is_some());
        assert!(result.query.len() > 0);

        assert!(result.filters.is_empty());
        assert_eq!(1, result.rules.len());
        let rule = &result.rules[0];
        assert_eq!(rule.policy_id, "masked_id");
        assert_eq!(rule.attribute_id, "object_id_1");
    }

    #[tokio::test]
    async fn test_query_policy_decision_one_masked_column_access_same_policy_id() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::id\")",
                "allow",
                // {"name": "replace_null"}
                "eyJuYW1lIjogInJlcGxhY2VfbnVsbCJ9",
                "policy_id",
            ),
        ];

        let mut policy_models = get_policy_model_input();
        policy_models.insert(
            "policy_id".to_string(),
            AccessPolicyModel {
                policy_id: "policy_id".to_string(),
                prio_strict: true,
                expire_datetime_utc: 0,
                normalized_name: "policy_id".to_string(),
            },
        );

        let result =
            run_default_test(policies, None, None, None, None, Some(policy_models), None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.visible_schema.is_some());
        assert!(result.query.len() > 0);

        assert!(result.filters.is_empty());
        assert_eq!(1, result.rules.len());
        let rule = &result.rules[0];
        assert_eq!(rule.policy_id, "policy_id");
        assert_eq!(rule.attribute_id, "object_id_1");
    }

    #[tokio::test]
    async fn test_query_policy_decision_multiple_policies_loose_policy_choice() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id_1",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::id\")",
                "allow",
                // {"name": "rounding", "properties": {"value": "2"}}
                "eyJuYW1lIjogInJvdW5kaW5nIiwgInByb3BlcnRpZXMiOiB7InZhbHVlIjogIjIifX0=",
                "policy_id_1",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::id\")",
                "allow",
                // {"name": "left", "properties": {"len": "3"}}
                "eyJuYW1lIjogImxlZnQiLCAicHJvcGVydGllcyI6IHsibGVuIjogIjMifX0=",
                "policy_id_2",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::id\")",
                "allow",
                // {"name": "xxhash3", "properties": null}
                "eyJuYW1lIjogInh4aGFzaDMiLCAicHJvcGVydGllcyI6IG51bGx9",
                "policy_id_2",
            ),
        ];

        let mut policy_models = get_policy_model_input();
        policy_models.insert(
            "policy_id_1".to_string(),
            AccessPolicyModel {
                policy_id: "policy_id_1".to_string(),
                prio_strict: false,
                expire_datetime_utc: 0,
                normalized_name: "policy_id_1".to_string(),
            },
        );

        policy_models.insert(
            "policy_id_2".to_string(),
            AccessPolicyModel {
                policy_id: "policy_id_2".to_string(),
                prio_strict: false,
                expire_datetime_utc: 0,
                normalized_name: "policy_id_2".to_string(),
            },
        );

        let result =
            run_default_test(policies, None, None, None, None, Some(policy_models), None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.visible_schema.is_some());
        assert!(result.query.len() > 0);

        assert!(result.filters.is_empty());
        assert_eq!(1, result.rules.len());
        let rule = &result.rules[0];
        assert_eq!(rule.policy_id, "policy_id_1");
        assert_eq!(rule.attribute_id, "object_id_1");
    }

    #[tokio::test]
    async fn test_query_policy_decision_one_filtered_column_access() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::firstname\")",
                "allow",
                // {"expression": "? > 0"}
                "eyJleHByZXNzaW9uIjogIj8gPiAwIn0=",
                "filter_id",
            ),
        ];

        let mut policy_models = get_policy_model_input();
        policy_models.insert(
            "filter_id".to_string(),
            AccessPolicyModel {
                policy_id: "filter_id".to_string(),
                prio_strict: true,
                expire_datetime_utc: 0,
                normalized_name: "filter_name".to_string(),
            },
        );

        let result =
            run_default_test(policies, None, None, None, None, Some(policy_models), None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.rules.is_empty());
        assert!(result.visible_schema.is_some());
        assert!(result.query.len() > 0);

        assert_eq!(1, result.filters.len());
        let filter = &result.filters[0];
        assert_eq!(filter.policy_id, "filter_id");
        assert_eq!(filter.attribute_id, "object_id_2");
    }

    #[tokio::test]
    async fn test_query_policy_decision_one_deny_hidden_column_access() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.*",
                "TagExists(r.object, \"pii::firstname\")",
                "allow",
                // {"name": "hidden"}
                "eyJuYW1lIjogImhpZGRlbiJ9",
                "hidden_id",
            ),
        ];

        let mut policy_models = get_policy_model_input();
        policy_models.insert(
            "hidden_id".to_string(),
            AccessPolicyModel {
                policy_id: "hidden_id".to_string(),
                prio_strict: true,
                expire_datetime_utc: 0,
                normalized_name: "hidden_name".to_string(),
            },
        );

        let result =
            run_default_test(policies, None, None, None, None, Some(policy_models), None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_some());
        assert!(result.cause.is_some());
        assert!(result.rules.is_empty());
        assert!(result.filters.is_empty());
    }

    #[tokio::test]
    async fn test_query_policy_decision_entity_not_found() {
        let policies = vec![PolicyRule::new(
            "p",
            "catalog.schema.customers.*",
            "true",
            "allow",
            // {"full_access": true}
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "policy_id",
        )];

        let mut scan_output = get_scan_default_output();
        let objects = scan_output.objects.first_mut().unwrap();
        objects.attributes.push(ScanAttribute {
            entity_alias: "b".to_string(),
            name: "id".to_string(),
            alias: "id".to_string(),
        });
        objects.entities.push(ScanEntity {
            catalog: "catalog".to_string(),
            db: "schema".to_string(),
            name: "orders".to_string(),
            alias: "b".to_string(),
        });
        let result =
            run_default_test(policies, Some(scan_output), None, None, None, None, None).await;

        // check results
        assert!(result.is_err());
        let result = result.err().unwrap();
        match result {
            SecurityHandlerResult::EntityNotFound(entity) => {
                assert_eq!(entity, "catalog.schema.orders");
            }
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_query_policy_decision_attribute_not_found() {
        let policies = vec![PolicyRule::new(
            "p",
            "catalog.schema.customers.*",
            "true",
            "allow",
            // {"full_access": true}
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "policy_id",
        )];

        let mut scan_output = get_scan_default_output();
        let objects = scan_output.objects.first_mut().unwrap();
        objects.attributes.push(ScanAttribute {
            entity_alias: "b".to_string(),
            name: "id".to_string(),
            alias: "id".to_string(),
        });
        objects.entities.push(ScanEntity {
            catalog: "catalog".to_string(),
            db: "schema".to_string(),
            name: "orders".to_string(),
            alias: "b".to_string(),
        });
        let mut entities = get_entity_model_input();
        let mut objects = HashMap::new();
        objects.insert(
            "catalog.schema.orders.unknown".to_owned(),
            EntityAttributeModel {
                id: "unknown_id".to_string(),
                name: "unknown".to_string(),
                full_name: "catalog.schema.orders.unknown".to_string(),
                tags: vec![],
                is_aggregated: false,
                data_type: "STRING".to_owned(),
            },
        );

        entities.insert(
            "catalog.schema.orders".to_owned(),
            EntityModel {
                id: "extra_entity".to_string(),
                full_name: "catalog.schema.orders".to_string(),
                attributes: objects,
            },
        );
        let result = run_default_test(
            policies,
            Some(scan_output),
            None,
            None,
            Some(entities),
            None,
            None,
        )
        .await;

        // check results
        assert!(result.is_err());
        let result = result.err().unwrap();
        match result {
            SecurityHandlerResult::EntityNotFound(entity) => {
                assert_eq!(entity, "catalog.schema.orders.id");
            }
            _ => panic!("Expected EntityNotFound"),
        }
    }
    #[tokio::test]
    async fn test_query_policy_decision_one_deny_column_access() {
        // test: star expand, deny access to a single attribute
        let policies = vec![
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "TagExists(r.object, \"pii::firstname\")",
                "deny",
                // {"name": "hidden"}
                "eyJuYW1lIjogImhpZGRlbiJ9",
                "policy_id",
            ),
        ];

        let scan_output = ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "a".to_string(),
                }],
                attributes: vec![
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "firstname".to_string(),
                        alias: "firstname".to_string(),
                    },
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "lastname".to_string(),
                        alias: "lastname".to_string(),
                    },
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "email".to_string(),
                        alias: "email".to_string(),
                    },
                ],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some(
                "SELECT firstname, lastname, email FROM catalog.schema.customers as a".to_string(),
            ),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        };

        let result =
            run_default_test(policies, Some(scan_output), None, None, None, None, None).await;

        // check results
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_some());
        assert!(result.cause.is_some());
    }

    #[tokio::test]
    async fn test_query_policy_decision_star_expand_allow_all() {
        // test: star expand, allow all stars
        let policies = vec![PolicyRule::new(
            "p",
            "catalog.schema.customers.*",
            "true",
            "allow",
            // {"full_access": true}
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "policy_id",
        )];

        let scan_output = ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "a".to_string(),
                }],
                attributes: vec![ScanAttribute {
                    entity_alias: "a".to_string(),
                    name: "*".to_string(),
                    alias: "".to_string(),
                }],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some("SELECT * FROM catalog.schema.customers as a".to_string()),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        };

        let result =
            run_default_test(policies, Some(scan_output), None, None, None, None, None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());

        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.rules.is_empty());
        assert!(result.filters.is_empty());
    }

    // todo: add test for visible schema (check if this is actually correct)

    #[tokio::test]
    async fn test_query_policy_decision_star_expand_deny_one() {
        // test: star expand, deny access to a single attribute
        let policies = vec![
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "TagExists(r.object, \"pii::firstname\")",
                "deny",
                // {"name": "hidden"}
                "eyJuYW1lIjogImhpZGRlbiJ9",
                "policy_id",
            ),
        ];

        let scan_output = ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "a".to_string(),
                }],
                attributes: vec![ScanAttribute {
                    entity_alias: "a".to_string(),
                    name: "*".to_string(),
                    alias: "".to_string(),
                }],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some("SELECT * FROM catalog.schema.customers as a".to_string()),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        };

        let result =
            run_default_test(policies, Some(scan_output), None, None, None, None, None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());

        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.rules.is_empty());
        assert!(result.filters.is_empty());
        let fields = result
            .visible_schema
            .unwrap()
            .get("catalog")
            .unwrap()
            .db
            .get("schema")
            .unwrap()
            .table
            .get("customers")
            .unwrap()
            .columns
            .len();
        // Input are 4 columns, but one of them is hidden (firstname)
        assert_eq!(fields, 3);
    }

    #[tokio::test]
    async fn test_query_policy_decision_star_expand_hidden_attribute() {
        // test: star expand, hide a column based on a policy (hidden)
        let policies = vec![
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "TagExists(r.object, \"pii::firstname\")",
                "allow",
                // {"name": "hidden", "properties": null}
                "eyJuYW1lIjogImhpZGRlbiIsICJwcm9wZXJ0aWVzIjogbnVsbH0=",
                "policy_id",
            ),
        ];

        let scan_output = ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "a".to_string(),
                }],
                attributes: vec![ScanAttribute {
                    entity_alias: "a".to_string(),
                    name: "*".to_string(),
                    alias: "".to_string(),
                }],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some("SELECT * FROM catalog.schema.customers as a".to_string()),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        };

        let result =
            run_default_test(policies, Some(scan_output), None, None, None, None, None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());

        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.rules.is_empty());
        assert!(result.filters.is_empty());
        let fields = result
            .visible_schema
            .unwrap()
            .get("catalog")
            .unwrap()
            .db
            .get("schema")
            .unwrap()
            .table
            .get("customers")
            .unwrap()
            .columns
            .len();
        // Input are 4 columns, but one of them is hidden (firstname)
        assert_eq!(fields, 3);
    }

    #[tokio::test]
    async fn test_query_policy_decision_deny_hidden_attribute() {
        // test: deny access to a hidden column based on the policy (hidden)
        let policies = vec![
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "true",
                "allow",
                // {"full_access": true}
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "policy_id",
            ),
            PolicyRule::new(
                "p",
                "catalog.schema.customers.*",
                "TagExists(r.object, \"pii::firstname\")",
                "allow",
                // {"name": "hidden", "properties": null}
                "eyJuYW1lIjogImhpZGRlbiIsICJwcm9wZXJ0aWVzIjogbnVsbH0=",
                "policy_id",
            ),
        ];

        let scan_output = ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "a".to_string(),
                }],
                attributes: vec![
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "firstname".to_string(),
                        alias: "firstname".to_string(),
                    },
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "lastname".to_string(),
                        alias: "lastname".to_string(),
                    },
                ],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some(
                "SELECT firstname, lastname FROM catalog.schema.customers as a".to_string(),
            ),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        };

        let result =
            run_default_test(policies, Some(scan_output), None, None, None, None, None).await;

        // check results
        if !result.is_ok() {
            println!("{:?}", result);
        }
        assert!(result.is_ok());

        let result = result.ok().unwrap();
        assert!(result.cause.is_some());
        assert!(result.request_url.is_some());
    }

    #[test]
    fn test_get_entity_attributes_found() {
        let scan_output = get_scan_default_output();
        let found = QueryPolicyDecision::get_entity_attributes(&scan_output.objects);
        assert_eq!(found.len(), 4);
        assert!(found.iter().map(|(x, _, _)| x.is_some()).all(|b| b));
        assert!(found.iter().all(|(_, _, x)| !*x));
    }

    #[tokio::test]
    async fn test_fill_star_attributes_found() {
        let scan_output = get_scan_star_output();
        let mut attributes = QueryPolicyDecision::get_entity_attributes(&scan_output.objects);

        // initially we have a star attribute
        assert_eq!(
            attributes
                .iter()
                .find(|(_, x, _)| x.name == "*")
                .iter()
                .count(),
            1
        );

        let star_attributes = vec![
            ScanAttribute {
                entity_alias: "customers".to_string(),
                name: "id".to_string(),
                alias: "a".to_string(),
            },
            ScanAttribute {
                entity_alias: "customers".to_string(),
                name: "first_name".to_string(),
                alias: "first_name".to_string(),
            },
            ScanAttribute {
                entity_alias: "customers".to_string(),
                name: "last_name".to_string(),
                alias: "last_name".to_string(),
            },
        ];

        let mut star_map = HashMap::new();
        star_map.insert("catalog.schema.customers".to_string(), star_attributes);
        let _ = QueryPolicyDecision::fill_star_attributes(&mut attributes, &star_map).await;

        assert_eq!(attributes.len(), 3);
        // now we don't have any stars anymore
        assert_eq!(
            attributes
                .iter()
                .find(|(_, x, _)| x.name == "*")
                .iter()
                .count(),
            0
        );
        // all stars are marked as star origins
        assert!(attributes.iter().all(|(_, _, x)| *x));
    }

    #[tokio::test]
    async fn test_fill_star_attributes_entry_not_found() {
        let scan_output = get_scan_star_output();
        let mut attributes = QueryPolicyDecision::get_entity_attributes(&scan_output.objects);
        let star_attributes = vec![ScanAttribute {
            entity_alias: "customers".to_string(),
            name: "id".to_string(),
            alias: "a".to_string(),
        }];

        let mut items = HashMap::new();
        items.insert("unknown".to_string(), star_attributes);
        let found_star = QueryPolicyDecision::fill_star_attributes(&mut attributes, &items).await;
        assert!(found_star.is_err());
        if let Some(e) = found_star.err() {
            match e {
                SecurityHandlerResult::EntityNotFound(v) => {
                    assert_eq!("catalog.schema.customers", v)
                }
                _ => unreachable!(), // Handle any other unexpected errors
            }
        }
    }

    fn get_default_policy() -> Vec<PolicyRule> {
        vec![PolicyRule::new(
            "p",
            "*",
            "true",
            "allow",
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "policy_id",
        )]
    }

    async fn get_defaults(
        user_model_items: Option<HashMap<String, UserModel>>,
        group_model_items: Option<HashMap<String, GroupModel>>,
        entity_model_items: Option<HashMap<String, EntityModel>>,
        policy_model_items: Option<HashMap<String, AccessPolicyModel>>,
    ) -> (DefaultModel, ScanOutput, CacheContainer) {
        (
            DefaultModel::from_str(ABAC_MODEL).await.unwrap(),
            get_scan_default_output(),
            CacheContainer::new(
                get_user_model_cache(user_model_items),
                get_group_model_cache(group_model_items),
                get_entity_model_cache(entity_model_items),
                get_policy_model_cache(policy_model_items),
            ),
        )
    }

    fn get_default_policy_cache(
        rules: Vec<PolicyRule>,
    ) -> (
        Arc<MultiLayeredCache<u64, CachedPolicyRules>>,
        CachedAdapter,
    ) {
        let mut cached = HashMap::new();
        cached.insert(0, CachedPolicyRules::PolicyId(100));
        cached.insert(100, CachedPolicyRules::Policy(rules));
        let rules_cache = Arc::new(MultiLayeredCache::new(
            10,
            Box::new(DummyBackendProvider::from(cached)),
            Box::new(DummyBackendProvider::new()),
        ));
        let adapter = CachedAdapter::new(rules_cache.clone());
        (rules_cache, adapter)
    }

    fn get_scan_default_output() -> ScanOutput {
        ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "a".to_string(),
                }],
                attributes: vec![
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "user_id".to_string(),
                        alias: "uid".to_string(),
                    },
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "firstname".to_string(),
                        alias: "firstname".to_string(),
                    },
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "lastname".to_string(),
                        alias: "lastname".to_string(),
                    },
                    ScanAttribute {
                        entity_alias: "a".to_string(),
                        name: "email".to_string(),
                        alias: "email".to_string(),
                    },
                ],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some("SELECT a.user_id as [uid], a.firstname as [firstname], a.lastname as [lastname], a.email as [email] FROM catalog.schema.customers as a".to_string()),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        }
    }

    fn get_scan_star_output() -> ScanOutput {
        ScanOutput {
            objects: vec![ScanOutputObject {
                scope: 0,
                entities: vec![ScanEntity {
                    catalog: "catalog".to_string(),
                    db: "schema".to_string(),
                    name: "customers".to_string(),
                    alias: "customers".to_string(),
                }],
                attributes: vec![ScanAttribute {
                    entity_alias: "customers".to_string(),
                    name: "*".to_string(),
                    alias: "".to_string(),
                }],
                is_agg: false,
            }],
            dialect: "tsql".to_string(),
            query: Some("SELECT * FROM catalog.schema.customers".to_string()),
            query_type: "SELECT".to_string(),
            error: None,
            target_entity: None,
        }
    }

    fn get_user_model_cache(
        items: Option<HashMap<String, UserModel>>,
    ) -> Arc<Box<MultiLayeredCache<String, UserModel>>> {
        let items = if let Some(items) = items {
            items
        } else {
            get_user_model_input()
        };
        Arc::new(Box::new(MultiLayeredCache::new(
            10,
            Box::from(DummyBackendProvider::from(items.clone())),
            Box::from(DummyBackendProvider::from(items)),
        )))
    }

    fn get_policy_model_cache(
        items: Option<HashMap<String, AccessPolicyModel>>,
    ) -> Arc<Box<MultiLayeredCache<String, AccessPolicyModel>>> {
        let items = if let Some(items) = items {
            items
        } else {
            get_policy_model_input()
        };
        Arc::new(Box::new(MultiLayeredCache::new(
            10,
            Box::from(DummyBackendProvider::from(items.clone())),
            Box::from(DummyBackendProvider::from(items)),
        )))
    }

    fn get_entity_model_cache(
        items: Option<HashMap<String, EntityModel>>,
    ) -> Arc<Box<MultiLayeredCache<String, EntityModel>>> {
        let items = if let Some(items) = items {
            items
        } else {
            get_entity_model_input()
        };
        Arc::new(Box::new(MultiLayeredCache::new(
            10,
            Box::from(DummyBackendProvider::from(items.clone())),
            Box::from(DummyBackendProvider::from(items)),
        )))
    }

    fn get_group_model_cache(
        items: Option<HashMap<String, GroupModel>>,
    ) -> Arc<Box<MultiLayeredCache<String, GroupModel>>> {
        let items = if let Some(items) = items {
            items
        } else {
            get_group_model_input()
        };
        Arc::new(Box::new(MultiLayeredCache::new(
            10,
            Box::from(DummyBackendProvider::from(items.clone())),
            Box::from(DummyBackendProvider::from(items)),
        )))
    }

    fn get_session_model_input() -> SessionModel {
        SessionModel {
            user_id: "user_id".to_string(),
            impersonate_user_id: None,
            id: "session_id".to_string(),
            app_id: 0,
            app_name: "app_name".to_string(),
            app_type: "app_type".to_string(),
            app_driver: "app_driver".to_string(),
            source_ipv4: "source_ipv4".to_string(),
            country_iso2: "country_iso2".to_string(),
            continent: "continent".to_string(),
            timezone: "timezone".to_string(),
            time: 0,
            day_of_week: 0,
            branch: "branch".to_string(),
            compute_id: "compute_id".to_string(),
            policy_id: "policy_id".to_string(),
            domain_id: "domain_id".to_string(),
            workspace_id: "workspace_id".to_string(),
        }
    }

    fn get_policy_model_input() -> HashMap<String, AccessPolicyModel> {
        let mut policy_model_input = HashMap::new();
        policy_model_input.insert(
            "policy_id".to_string(),
            AccessPolicyModel {
                normalized_name: "no_name".to_string(),
                policy_id: "policy_id".to_string(),
                prio_strict: true,
                expire_datetime_utc: 0,
            },
        );
        policy_model_input
    }

    fn get_entity_model_input() -> HashMap<String, EntityModel> {
        let mut entity_model_input = HashMap::new();
        entity_model_input.insert(
            "catalog.schema.customers".to_string(),
            EntityModel {
                id: "entity_model_id".to_string(),
                full_name: "catalog.schema.customers".to_string(),
                attributes: get_object_model_input(),
            },
        );
        entity_model_input
    }

    fn get_user_model_input() -> HashMap<String, UserModel> {
        let mut user_model_input = HashMap::new();
        user_model_input.insert(
            "user_id".to_string(),
            UserModel {
                id: "user_id".to_string(),
                principal_name: "user_principal_name_1".to_string(),
                roles: vec!["user_role_1".to_string()],
                tags: vec!["pii::email".to_string()],
                access_policy_ids: vec!["policy_id".to_string()],
            },
        );
        user_model_input
    }

    fn get_group_model_input() -> HashMap<String, GroupModel> {
        let mut group_model_input = HashMap::new();
        group_model_input.insert(
            "user_id".to_string(),
            GroupModel {
                user_id: "user_id".to_string(),
                groups: vec![GroupInstance {
                    id: "group_id".to_string(),
                    tags: vec!["pii::username".to_string()],
                }],
            },
        );
        group_model_input
    }

    fn get_object_model_input() -> HashMap<String, EntityAttributeModel> {
        let mut values = HashMap::new();
        values.insert(
            "catalog.schema.customers.user_id".to_owned(),
            EntityAttributeModel {
                id: "object_id_1".to_string(),
                name: "user_id".to_string(),
                full_name: "catalog.schema.customers.user_id".to_string(),
                is_aggregated: false,
                tags: vec!["pii::id".to_string()],
                data_type: "INT".to_string(),
            },
        );

        values.insert(
            "catalog.schema.customers.firstname".to_owned(),
            EntityAttributeModel {
                id: "object_id_2".to_string(),
                name: "firstname".to_string(),
                full_name: "catalog.schema.customers.firstname".to_string(),
                is_aggregated: false,
                tags: vec!["pii::firstname".to_string()],
                data_type: "STRING".to_string(),
            },
        );

        values.insert(
            "catalog.schema.customers.lastname".to_owned(),
            EntityAttributeModel {
                id: "object_id_3".to_string(),
                name: "lastname".to_string(),
                full_name: "catalog.schema.customers.lastname".to_string(),
                is_aggregated: false,
                tags: vec!["pii::lastname".to_string()],
                data_type: "STRING".to_string(),
            },
        );

        values.insert(
            "catalog.schema.customers.email".to_owned(),
            EntityAttributeModel {
                id: "object_id_4".to_string(),
                name: "email".to_string(),
                full_name: "catalog.schema.customers.email".to_string(),
                is_aggregated: false,
                tags: vec!["pii::email".to_string()],
                data_type: "STRING".to_string(),
            },
        );

        values
    }

    struct DummyBackendProvider<K, V>
    where
        K: Send + Hash + Clone + Sync + Eq,
        V: Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        items: HashMap<K, V>,
    }

    impl<K, V> DummyBackendProvider<K, V>
    where
        K: Send + Hash + Clone + Sync + Eq,
        V: Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        fn new() -> Self {
            DummyBackendProvider {
                items: HashMap::new(),
            }
        }

        fn from(items: HashMap<K, V>) -> Self {
            DummyBackendProvider { items }
        }
    }

    #[async_trait]
    impl<K, V> BackendProvider<K, V> for DummyBackendProvider<K, V>
    where
        K: Send + Hash + Clone + Sync + Eq,
        V: Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        async fn get(&self, key: &K) -> Result<Option<V>, String> {
            Ok(self.items.get(key).cloned())
        }

        async fn set(&self, _: &K, _: &V) -> Result<(), String> {
            unreachable!()
        }

        async fn has(&self, key: &K) -> Result<bool, String> {
            Ok(self.items.contains_key(key))
        }

        async fn evict(&self, _: &K) -> Result<(), String> {
            unreachable!()
        }

        fn generate_key(&self, _: &K) -> String {
            unreachable!()
        }
    }

    struct FakeRepoBackend {}

    #[async_trait]
    impl RepoBackend for FakeRepoBackend {
        async fn get_entity_model(&self, _: String) -> Result<Option<EntityModel>, String> {
            unreachable!()
        }

        async fn get_access_policy_model(
            &self,
            _: String,
        ) -> Result<Option<AccessPolicyModel>, String> {
            unreachable!()
        }

        async fn get_user_model(&self, _: String) -> Result<Option<UserModel>, String> {
            unreachable!()
        }

        async fn get_group_model(&self, _: String) -> Result<Option<GroupModel>, String> {
            unreachable!()
        }

        async fn generate_access_request(
            &self,
            _: String,
            _: String,
            _: String,
        ) -> Result<DataAccessRequestResponse, String> {
            Ok(DataAccessRequestResponse {
                message: "Some message".to_string(),
                url: "https://unilake.com".to_string(),
            })
        }

        async fn get_access_by_action(
            &self,
            _: String,
            _: String,
            _: Option<String>,
            _: String,
        ) -> Result<bool, String> {
            unreachable!()
        }

        async fn get_ip_info_model(&self, _: String) -> Result<Option<IpInfoModel>, String> {
            unreachable!()
        }

        async fn get_app_info_model(&self, _: String) -> Result<Option<AppInfoModel>, String> {
            unreachable!()
        }

        async fn get_active_policy_rules(
            &self,
            _: String,
        ) -> Result<Option<CachedPolicyRules>, String> {
            unreachable!()
        }
    }
}
