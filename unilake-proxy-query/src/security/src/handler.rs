// todo: https://github.com/notken12/licensesnip

use crate::effector::PdpEffector;
use crate::functions::add_functions;
use crate::policies::{
    PolicyCollectResult, PolicyFound, PolicyHitManager, PolicyLogger, PolicyType,
};
use crate::repository::CacheContainer;
use crate::HitRule;
use casbin::{Cache, CachedEnforcer, CoreApi};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use ulid::Ulid;
use unilake_common::error::{TdsWireError, TokenError};
use unilake_common::model::{EntityModel, ObjectModel, SessionModel};
use unilake_sql::{
    run_scan_operation, run_secure_operation, run_transpile_operation, Catalog, ParserError,
    ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject, TranspilerDenyCause, TranspilerInput,
    TranspilerInputFilter, TranspilerInputRule, VisibleSchemaBuilder,
};

#[derive(Debug)]
pub enum SecurityHandlerResult {
    /// Happens when a requested entity <catalog>.<schema>.<entity> does not exist
    EntityNotFound(String),
    /// Happens when a requested entity <catalog>.<schema>.<entity> exists but is not allowed to be accessed
    EntityNotAllowed(String),
    /// Happens when the user groups cannot be found
    UserGroupsNotFound(String),
    /// Happens when the user cannot be found
    UserNotFound(String),
    /// Happens when there are issues with the repository
    RepositoryError(String),
    /// Happens when there are issues with the policy being used
    PolicyError(String),
}

pub enum SecurityHandlerError {
    WireError(TdsWireError),
    QueryError(ParserError),
}

impl From<TdsWireError> for SecurityHandlerError {
    fn from(value: TdsWireError) -> Self {
        SecurityHandlerError::WireError(value)
    }
}

impl From<ParserError> for SecurityHandlerError {
    fn from(value: ParserError) -> Self {
        SecurityHandlerError::QueryError(value)
    }
}

impl From<SecurityHandlerError> for TokenError {
    fn from(value: SecurityHandlerError) -> Self {
        match value {
            SecurityHandlerError::WireError(e) => TokenError {
                line: 0,
                code: 0,
                message: e.to_string(),
                class: 0,
                procedure: "".to_string(),
                server: "".to_string(),
                state: 0,
            },
            SecurityHandlerError::QueryError(e) => {
                if let Some(err) = e.errors.first() {
                    return TokenError {
                        message: format!(
                            "{}. Line: {}, Col: {}. {}",
                            err.start_context, err.line, err.col, err.description
                        ),
                        class: 0,
                        line: err.line,
                        procedure: "".to_string(),
                        server: "".to_string(),
                        state: 1,
                        code: 0,
                    };
                }
                TokenError {
                    line: 0,
                    code: 0,
                    message: format!("Parser error: {}", e.message),
                    class: 0,
                    procedure: "".to_string(),
                    server: "".to_string(),
                    state: 0,
                }
            }
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
    cached_enforcer: Arc<Mutex<CachedEnforcer>>,
    cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    cached_backend: CacheContainer,
}

impl SecurityHandler {
    pub fn new(
        cached_enforcer: Arc<Mutex<CachedEnforcer>>,
        session_model: SessionModel,
        cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
        cached_backend: CacheContainer,
    ) -> Self {
        SecurityHandler {
            query_id: Ulid::new(),
            scan_output: None,
            transpiler_input: None,
            output_query: None,
            output_query_secured: None,
            input_query_secured: None,
            input_query: None,
            cached_enforcer,
            session_model,
            cached_rules,
            cached_backend,
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
    pub async fn handle_query(
        &mut self,
        query: &str,
        dialect: &str,
        catalog: &str,
        database: &str,
    ) -> Result<&str, SecurityHandlerError> {
        // You can only handle a query once
        if let Some(ref query_result) = self.output_query {
            return Ok(query_result);
        }

        self.input_query = Some(Arc::from(query.to_string()));
        let scan_output = self.scan(query, dialect, catalog, database)?;
        if scan_output.error.is_some() {
            return Err(SecurityHandlerError::QueryError(scan_output.error.unwrap()));
        }

        let transpiler_input = QueryPolicyDecision::new(
            &self.cached_backend,
            self.cached_enforcer.clone(),
            &self.session_model,
            self.cached_rules.clone(),
        )
        .process(&scan_output)
        .await
        .ok()
        .unwrap();
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
    ) -> Result<String, SecurityHandlerError> {
        let transpiler_output = run_transpile_operation(scanned, secure_output)
            .map_err(|e| TdsWireError::Protocol(e.to_string()))?;
        if let Some(error) = transpiler_output.error {
            return Err(SecurityHandlerError::QueryError(error));
        }
        Ok(transpiler_output.sql_transformed)
    }

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

    pub fn secure_input_query(&mut self) -> Result<&str, SecurityHandlerError> {
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

    pub fn get_query_id(&self) -> Ulid {
        self.query_id
    }
}

#[derive(PartialEq, Clone)]
enum AttributeAccess {
    Allowed,
    Hidden,
    Denied,
}

struct QueryPolicyDecision<'a> {
    cached_backend: &'a CacheContainer,
    enforcer: Arc<Mutex<CachedEnforcer>>,
    session_model: &'a SessionModel,
    policy_cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
}

impl<'a> QueryPolicyDecision<'a> {
    pub fn new(
        cached_backend: &'a CacheContainer,
        enforcer: Arc<Mutex<CachedEnforcer>>,
        session_model: &'a SessionModel,
        policy_cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    ) -> Self {
        QueryPolicyDecision {
            cached_backend,
            enforcer,
            session_model,
            policy_cache,
        }
    }

    fn get_object_model<'b>(
        &self,
        entity_model: &'b EntityModel,
        attribute_full_path_name: &str,
    ) -> Option<&'b ObjectModel> {
        entity_model.objects.get(attribute_full_path_name)
    }

    // todo: we also need to check the intent, do you perform a select statement, update or delete etc... based on roles. This is where gravitino api should help
    // todo: PAP should generate the necessary rules, it should however also generate the regular access rules where no filtering or masking should be applied, the actions below should honor that
    // todo: in order to cover both the cases above, we might as well use a select permission as a full transparent access approach and the below for consuming data via this security layer,
    // this will have to be checked before executing this part as no policies are applicable in a full access scenario
    pub async fn process(
        &self,
        scan_output: &ScanOutput,
    ) -> Result<TranspilerInput, SecurityHandlerResult> {
        // enforcer should only be used in a single thread (which normally should be the case)
        let mut enforcer = self.enforcer.lock().await;

        // reload policy (in case of changes)
        if let Err(err) = enforcer.load_policy().await {
            panic!("Failed to load policy: {}", err);
        }

        // set possible output
        let mut cause: Option<Vec<TranspilerDenyCause>> = None;
        let mut request_url: Option<String> = None;

        // set input (user and group)
        let user_model = self
            .cached_backend
            .user_model
            .get(&self.session_model.user_id)
            .await;

        let group_model = self
            .cached_backend
            .group_model
            .get(&self.session_model.user_id)
            .await;

        // walk through all scopes and attributes
        let mut masking_rules = Vec::new();
        let mut filter_rules = Vec::new();
        let mut entities = HashMap::new();
        let mut exclude_attributes_visible_map = HashSet::new();

        // set enforcer dependencies
        // todo(mrhamburg): would prefer that we do this only once, not every time. But this works for now (less efficient)
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
            // set new logger instance (logger is scoped to a scope)
            let mut pm = PolicyHitManager::new(self.policy_cache.clone());
            let logger = Box::new(PolicyLogger::new(pm.get_sender()));

            // set logger to enforcer
            enforcer.set_logger(logger);
            enforcer.enable_log(true);

            // unpack stars
            let mut star_map = HashMap::new();
            // todo: fix unwrap
            for entity in items.iter().map(|(e, _, _)| e.unwrap()) {
                let found = self
                    .get_star_attributes(
                        &self.cached_backend,
                        entity.get_full_name(),
                        &entity.alias,
                    )
                    .await?;

                // todo: set last time accessed (via access policy)
                star_map.insert(entity.get_full_name(), found.1);
                entities.insert(entity.get_full_name(), found.0);
            }
            Self::fill_star_attributes(items, &star_map).await?;

            // process all attributes for policy enforcement
            let mut results: HashMap<&str, bool> = HashMap::new();
            for (scan_entity, attribute, is_starred) in items.iter() {
                // todo: check this unwrap
                let scan_entity = scan_entity.unwrap();
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
                match enforcer.enforce((
                    &user_model,
                    &group_model,
                    self.session_model,
                    &object_model,
                )) {
                    Ok(status) => {
                        if !status && !*is_starred {
                            // explicitly requested entity attribute
                            // will be handled later (when processing policies found)
                            results.insert(object_model.id.as_ref(), false);
                        } else if !status {
                            // implicitly requested starred entity attribute
                            exclude_attributes_visible_map.insert(entity_attribute_name);
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

            // this will process the results and gather their associated policies
            let policies_found = match pm
                .process_hits()
                .map_err(|e| SecurityHandlerResult::PolicyError(e))?
            {
                PolicyCollectResult::Found(f) => f,
                PolicyCollectResult::CacheInvalid => {
                    // todo: invalidate cache and retry
                    todo!()
                }
                PolicyCollectResult::NotFound => {
                    return Err(SecurityHandlerResult::PolicyError(
                        "Could not find any policy results, are the policies loaded correctly?"
                            .to_string(),
                    ));
                }
            };

            let object_models = entities
                .values()
                .flat_map(|e| &e.objects)
                .map(|(_, v)| v)
                .collect();
            for (object_id, rules) in policies_found {
                let prio_stricter = self.check_policy_rules_prio(&rules).await?;
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
        if cause.is_some() {
            request_url = Some(self.get_deny_access_url().await?);
        }

        // prepare transpiler input
        let entities = entities.into_iter().map(|(_, v)| v).collect();
        let query = if scan_output.query.is_some() {
            scan_output.query.clone().unwrap()
        } else {
            return Err(SecurityHandlerResult::PolicyError(
                "Query not found".to_owned(),
            ));
        };
        Ok(TranspilerInput {
            cause,
            request_url,
            query,
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

    /// Checks if a stricter policy is preferred based on the policy rules found
    async fn check_policy_rules_prio(
        &self,
        policies: &[PolicyFound],
    ) -> Result<bool, SecurityHandlerResult> {
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
                    if p.prio_strict {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Gets the full object name, attribute name and whether it's a starred attribute
    fn find_star_and_attribute_name(
        entities_and_attributes: &[(Option<&ScanEntity>, &ScanAttribute, bool)],
        objects: &Vec<&ObjectModel>,
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

    async fn check_user_access(&self, scan_output: &ScanOutput) -> () {
        // check if user has access to the entity involved with the given intent (gravitino api, select|update|delete)
        // we handle select, create|modify|delete intents are done by gravitino api
        todo!()
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

    async fn get_deny_access_url(&self) -> Result<String, SecurityHandlerResult> {
        todo!()
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
                if !exclude.contains(&format!("{}.{}", model.full_name, n)) {
                    table.get_or_add_column(n.to_owned(), t.to_owned());
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
            for attr in &entity.attributes {
                let full_entity_name = format!("{}.{}", full_entity_name, attr.0);
                if let Some(_) = self.get_object_model(&entity, &full_entity_name) {
                    attributes.push(ScanAttribute {
                        entity_alias: entity_alias.to_owned(),
                        name: attr.0.to_owned(),
                        alias: attr.0.to_owned(),
                    });
                } else {
                    return Err(SecurityHandlerResult::EntityNotFound(format!(
                        "{}.{}",
                        full_entity_name, attr.0
                    )));
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
    use crate::adapter::cached_adapter::{CacheEntity, CachedAdapter};
    use crate::caching::layered_cache::{BackendProvider, MultiLayeredCache};
    use crate::handler::{CacheContainer, QueryPolicyDecision, SecurityHandlerResult};
    use crate::{HitRule, ABAC_MODEL};
    use async_trait::async_trait;
    use casbin::{Cache, CachedEnforcer, CoreApi, DefaultCache, DefaultModel};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use unilake_common::model::{
        AccessPolicyModel, AccountType, EntityModel, GroupInstance, GroupModel, ObjectModel,
        PolicyRule, SessionModel, UserModel,
    };
    use unilake_sql::{ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject, TranspilerInput};

    async fn run_default_test(
        rules: Vec<PolicyRule>,
        scan_output: Option<ScanOutput>,
        user_model_items: Option<HashMap<String, UserModel>>,
        group_model_items: Option<HashMap<String, GroupModel>>,
        entity_model_items: Option<HashMap<String, EntityModel>>,
        policy_model_items: Option<HashMap<String, AccessPolicyModel>>,
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
        let e = Arc::new(Mutex::new(
            CachedEnforcer::new(abac_model, adapter).await.unwrap(),
        ));
        let session_model = get_session_model_input();
        let policy_cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>> =
            Arc::new(Box::new(DefaultCache::new(10)));

        // set sut
        let sut = QueryPolicyDecision::new(
            &cache_container,
            e.clone(),
            &session_model,
            policy_cache.clone(),
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
        let result = run_default_test(get_default_policy(), None, None, None, None, None).await;

        // check results
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_none());
        assert!(result.cause.is_none());
        assert!(result.filters.is_empty());
        assert_eq!(0, result.rules.len())
    }

    #[tokio::test]
    async fn test_query_policy_decision_one_masked_column_access() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "no_id",
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
                usage: HashMap::new(),
            },
        );

        let result = run_default_test(policies, None, None, None, None, Some(policy_models)).await;

        // check results
        println!("{:?}", result);
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
    async fn test_query_policy_decision_one_filtered_column_access() {
        let policies = vec![
            PolicyRule::new(
                "p",
                "*",
                "true",
                "allow",
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "no_id",
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

        let result = run_default_test(policies, None, None, None, None, None).await;

        // check results
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
                "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
                "no_id",
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
        let result = run_default_test(policies, None, None, None, None, None).await;

        // check results
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
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "no_id",
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
        let result = run_default_test(policies, Some(scan_output), None, None, None, None).await;

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
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "no_id",
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
            ObjectModel {
                id: "unknown_id".to_string(),
                full_name: "catalog.schema.orders.unknown".to_string(),
                tags: vec![],
                last_time_accessed: 0,
                is_aggregated: false,
            },
        );

        entities.insert(
            "catalog.schema.orders".to_owned(),
            EntityModel {
                id: "extra_entity".to_string(),
                full_name: "catalog.schema.orders".to_string(),
                attributes: vec![("unknown".to_owned(), "INT".to_owned())],
                objects,
            },
        );
        let result = run_default_test(
            policies,
            Some(scan_output),
            None,
            None,
            Some(entities),
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
        todo!();
        let policies = vec![PolicyRule::new(
            "p",
            "catalog.schema.customers.*",
            "true",
            "allow",
            "eyJmdWxsX2FjY2VzcyI6IHRydWV9",
            "no_id",
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
        let result = run_default_test(policies, Some(scan_output), None, None, None, None).await;

        // check results
        assert!(result.is_ok());
        let result = result.ok().unwrap();
        assert!(result.request_url.is_some());
        assert!(result.cause.is_some());
        assert!(result.rules.is_empty());
        assert!(result.filters.is_empty());
    }

    #[tokio::test]
    async fn test_query_policy_decision_star_expand_allow_all() {
        // test: star expand, allow all stars
        todo!()
    }

    #[tokio::test]
    async fn test_query_policy_decision_star_expand_deny_one() {
        // test: star expand, deny access to a single attribute
        todo!()
    }

    #[tokio::test]
    async fn test_query_policy_decision_star_expand_hidden_attribute() {
        // test: star expand, hide a column based on a policy (hidden)
        todo!()
    }

    #[tokio::test]
    async fn test_query_policy_decision_deny_hidden_attribute() {
        // test: deny access to a hidden column based on the policy (hidden)
        todo!()
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
            "no_id",
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
    ) -> (Arc<MultiLayeredCache<u64, CacheEntity>>, CachedAdapter) {
        let mut cached = HashMap::new();
        cached.insert(0, CacheEntity::PolicyId(100));
        cached.insert(100, CacheEntity::Policy(rules));
        let rules_cache = Arc::new(MultiLayeredCache::new(
            10,
            Box::new(DummyBackendProvider::from(cached)),
            Box::new(DummyBackendProvider::new()),
        ));
        let adapter = CachedAdapter::new(rules_cache.clone());
        (rules_cache, adapter)
    }

    fn get_rules_cache(
        items: Option<HashMap<u64, (String, HitRule)>>,
    ) -> Arc<Box<dyn Cache<u64, (String, HitRule)>>> {
        let items = if let Some(items) = items {
            items
        } else {
            let mut input = HashMap::new();
            input.insert(
                0,
                (
                    "".to_string(),
                    HitRule {
                        id: 0,
                        rules: vec![],
                    },
                ),
            );
            input
        };
        Arc::new(Box::new(DefaultCache::new(10)))
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
            "no_id".to_string(),
            AccessPolicyModel {
                policy_id: "no_id".to_string(),
                prio_strict: true,
                usage: HashMap::new(),
            },
        );
        policy_model_input
    }

    fn get_entity_model_input() -> HashMap<String, EntityModel> {
        let mut entity_model_input = HashMap::new();
        entity_model_input.insert(
            "catalog.schema.customers".to_string(),
            EntityModel {
                id: "no_id".to_string(),
                full_name: "catalog.schema.customers".to_string(),
                attributes: vec![
                    ("user_id".to_string(), "INT".to_string()),
                    ("firstname".to_string(), "STRING".to_string()),
                    ("lastname".to_string(), "STRING".to_string()),
                    ("email".to_string(), "STRING".to_string()),
                ],
                objects: get_object_model_input(),
            },
        );
        entity_model_input
    }

    fn get_user_model_input() -> HashMap<String, UserModel> {
        let mut user_model_input = HashMap::new();
        user_model_input.insert(
            "user_id".to_string(),
            UserModel {
                account_type: AccountType::User,
                id: "user_id".to_string(),
                principal_name: "user_principal_name_1".to_string(),
                roles: vec!["user_role_1".to_string()],
                tags: vec!["pii::email".to_string()],
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
                entity_version: 0,
                groups: vec![GroupInstance {
                    id: "group_id".to_string(),
                    tags: vec!["pii::username".to_string()],
                }],
            },
        );
        group_model_input
    }

    fn get_object_model_input() -> HashMap<String, ObjectModel> {
        let mut values = HashMap::new();
        values.insert(
            "catalog.schema.customers.user_id".to_owned(),
            ObjectModel {
                id: "object_id_1".to_string(),
                full_name: "catalog.schema.customers.user_id".to_string(),
                is_aggregated: false,
                last_time_accessed: 0,
                tags: vec!["pii::id".to_string()],
            },
        );

        values.insert(
            "catalog.schema.customers.firstname".to_owned(),
            ObjectModel {
                id: "object_id_2".to_string(),
                full_name: "catalog.schema.customers.firstname".to_string(),
                is_aggregated: false,
                last_time_accessed: 0,
                tags: vec!["pii::firstname".to_string()],
            },
        );

        values.insert(
            "catalog.schema.customers.lastname".to_owned(),
            ObjectModel {
                id: "object_id_3".to_string(),
                full_name: "catalog.schema.customers.lastname".to_string(),
                is_aggregated: false,
                last_time_accessed: 0,
                tags: vec!["pii::lastname".to_string()],
            },
        );

        values.insert(
            "catalog.schema.customers.email".to_owned(),
            ObjectModel {
                id: "object_id_4".to_string(),
                full_name: "catalog.schema.customers.email".to_string(),
                is_aggregated: false,
                last_time_accessed: 0,
                tags: vec!["pii::email".to_string()],
            },
        );

        return values;
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

        fn set_items(&mut self, items: Vec<(K, V)>) {
            for (key, value) in items {
                self.items.insert(key.clone(), value.clone());
            }
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

        async fn set(&self, key: &K, value: &V) -> Result<(), String> {
            todo!()
        }

        async fn has(&self, key: &K) -> Result<bool, String> {
            Ok(self.items.contains_key(key))
        }

        async fn evict(&self, key: &K) -> Result<(), String> {
            todo!()
        }

        fn generate_key(&self, key: &K) -> String {
            todo!()
        }
    }
}
