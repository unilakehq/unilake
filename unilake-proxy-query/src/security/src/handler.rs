use crate::policies::{
    PolicyCollectResult, PolicyFound, PolicyHitManager, PolicyLogger, PolicyType,
};
use crate::repository::RepoBackend;
use crate::HitRule;
use casbin::{Cache, CachedEnforcer, CoreApi};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;
use ulid::Ulid;
use unilake_common::error::{TdsWireError, TokenError};
use unilake_common::model::{EntityModel, SessionModel};
use unilake_sql::{
    run_scan_operation, run_secure_operation, run_transpile_operation, Catalog, ParserError,
    ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject, TranspilerInput, VisibleSchemaBuilder,
};

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
    cached_enforcer: Arc<Mutex<CachedEnforcer>>,
    session_model: SessionModel,
    repo_backend: Arc<RepoBackend>,
    cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
}

impl SecurityHandler {
    pub fn new(
        cached_enforcer: Arc<Mutex<CachedEnforcer>>,
        session_model: SessionModel,
        repo_backend: Arc<RepoBackend>,
        cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
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
            repo_backend,
            cached_rules,
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
            &self.repo_backend,
            &self.cached_enforcer,
            &self.session_model,
            &self.cached_rules,
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

enum AttributeAccess {
    Allowed,
    Hidden,
    Denied,
}

struct QueryPolicyDecision<'a> {
    repo_backend: &'a Arc<RepoBackend>,
    enforcer: &'a Arc<Mutex<CachedEnforcer>>,
    session_model: &'a SessionModel,
    policy_cache: &'a Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
}

impl<'a> QueryPolicyDecision<'a> {
    pub fn new(
        repo_backend: &'a Arc<RepoBackend>,
        enforcer: &'a Arc<Mutex<CachedEnforcer>>,
        session_model: &'a SessionModel,
        policy_cache: &'a Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    ) -> Self {
        QueryPolicyDecision {
            repo_backend,
            enforcer,
            session_model,
            policy_cache,
        }
    }

    // todo: we also need to check the intent, do you perform a select statement, update or delete etc... based on roles
    pub async fn process(
        &self,
        scan_output: &ScanOutput,
    ) -> Result<TranspilerInput, SecurityHandlerResult> {
        // enforcer should only be used in a single thread
        let mut enforcer = self.enforcer.lock().await;

        // reload policy (in case of changes)
        if let Err(err) = enforcer.load_policy().await {
            panic!("Failed to load policy: {}", err);
        }

        // set input (user and group)
        let user_model = self
            .repo_backend
            .get_user_model(&self.session_model.user_id)
            .await?;
        let group_model = self
            .repo_backend
            .get_group_model(&self.session_model.user_id)
            .await?;

        // walk through all scopes and attributes
        let mut masking_rules = Vec::new();
        let mut filter_rules = Vec::new();
        let mut entities = Vec::new();
        let mut exclude_attributes_visible_map = Vec::new();

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
            enforcer.set_logger(logger);
            enforcer.enable_log(true);

            // unpack stars
            let mut star_map = HashMap::new();
            // todo: fix unwrap
            for entity in items.iter().map(|(e, _, _)| e.unwrap()) {
                let found = Self::get_star_attributes(
                    &self.repo_backend,
                    entity.get_full_name(),
                    &entity.alias,
                )
                .await?;
                star_map.insert(entity.get_full_name(), found.1);
                entities.push(found.0);
            }
            Self::fill_star_attributes(items, &star_map).await?;

            // process all attributes for policy enforcement
            for (entity, attribute, is_starred) in items {
                // formulate entity name
                let entity = entity.unwrap();
                let entity_name = format!(
                    "{}.{}.{}.{}",
                    entity.catalog, entity.db, entity.name, attribute.name
                );

                // get input object model
                let object_model = self.repo_backend.get_object_model(&entity_name).await?;

                // check access based on policy
                match enforcer.enforce((
                    &user_model,
                    &group_model,
                    self.session_model,
                    &object_model,
                )) {
                    Ok(status) => {
                        if !status && !*is_starred {
                            return self.get_denied_access_result().await;
                        } else if !status {
                            exclude_attributes_visible_map.push(format!(
                                "{}:{}",
                                entity.get_full_name(),
                                attribute.name
                            ));
                        }
                    }
                    Err(err) => {
                        panic!("Failed to enforce policy: {}", err)
                    }
                }
            }

            let items = match pm
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
                        "Could not find any policy results".to_string(),
                    ));
                }
            };

            for (object_id, rules) in items {
                // todo: determine where we can get the stricter and looser rules priority from
                let masking_rule = if let Some(rule) =
                    PolicyHitManager::resolve_masking_policy_conflict(&rules, true)
                {
                    Some(rule.clone())
                } else {
                    None
                };
                let filter_rule = rules
                    .iter()
                    .filter(|p| p.policy_type == PolicyType::FilterRule);

                masking_rules.push((scope, object_id.clone(), masking_rule));
                filter_rule.for_each(|p| filter_rules.push((scope, object_id.clone(), p.clone())));
            }
        }

        // todo: build set to iterate over (scoped, per attribute)
        // todo: take into account stars, we need to to unstar them for all schema records one is allowed to access
        // (no need for deny except for full table deny, since a star is a request for all accessible attributes)

        // todo: on failure, an access request needs to be formulated (either for access or since a deny rule has been hit)

        // todo: the visible schema also needs to be built, we actually need this beforehand

        Ok(TranspilerInput {
            cause: None,
            query: scan_output.query.clone().unwrap(),
            request_url: None,
            rules: vec![],
            filters: vec![],
            visible_schema: Self::get_visible_schema(entities, exclude_attributes_visible_map),
        })
    }

    async fn get_denied_access_result(&self) -> Result<TranspilerInput, SecurityHandlerResult> {
        todo!()
    }

    /// Extracts the combinations entity and attribute from the given scope
    /// Returned tuple: Optional Entity, Attribute, Star flag (defaults false)
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
        for item in items.iter_mut().find(|(_, att, _)| att.name == "*") {
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
        exclude: Vec<String>,
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

    /// Requires the full entity name <catalog>.<schema>.<entity> and gets all its attributes
    /// uses these attributes to create an entity attribute mapping. Entity alias is required
    /// since we need to know which attributes belong to which entity within a given scope
    async fn get_star_attributes(
        backend: &Arc<RepoBackend>,
        full_entity_name: String,
        entity_alias: &str,
    ) -> Result<(EntityModel, Vec<ScanAttribute>), SecurityHandlerResult> {
        if let Ok(entity) = backend.get_entity_model(&full_entity_name).await {
            let mut attributes = Vec::new();
            for attr in &entity.attributes {
                let full_entity_name = format!("{}.{}", full_entity_name, attr.0);
                if let Ok(_) = backend.get_object_model(&full_entity_name).await {
                    attributes.push(ScanAttribute {
                        entity_alias: entity_alias.to_owned(),
                        name: attr.0.to_owned(),
                        alias: attr.0.to_owned(),
                    });
                } else {
                    return Err(SecurityHandlerResult::EntityNotFound(full_entity_name));
                }
            }

            Ok((entity, attributes))
        } else {
            Err(SecurityHandlerResult::EntityNotFound(full_entity_name))
        }
    }

    /// Check if we are allowed to access the given object attribute
    ///
    ///     Allowed: we can further process this attribute as is and use it in the visible schema
    ///     Hidden: we need to hide this attribute from the visible schema
    ///     Denied: we are trying to access an attribute we don't have access to
    fn get_attribute_access_status(policy: PolicyFound, from_star: bool) -> AttributeAccess {
        if policy.policy_name.is_some_and(|n| n == "hidden") {
            return if from_star {
                AttributeAccess::Hidden
            } else {
                AttributeAccess::Denied
            };
        }
        AttributeAccess::Allowed
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
        AccountType, EntityModel, GroupInstance, GroupModel, ObjectModel, SessionModel, UserModel,
    };
    use unilake_sql::{ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject};

    #[tokio::test]
    async fn test_query_policy_decision_single_object() {
        let (abac_model, scan_output, cache_container) = get_defaults().await;
        let (rules_cache, adapter) = get_default_rules_cache();
        let mut e = Arc::new(Mutex::new(
            CachedEnforcer::new(abac_model, adapter).await.unwrap(),
        ));
        let session_model = get_session_model_input();

        let sut = QueryPolicyDecision::new(&cache_container, e.clone(), &session_model, None);
        let result = sut.process(&scan_output).await;
    }

    #[test]
    fn test_get_entity_attributes_found() {
        let scan_output = get_scan_default_output();
        let found = QueryPolicyDecision::get_entity_attributes(&scan_output.objects, 0);
        assert_eq!(found.len(), 4);
        assert!(found.iter().map(|(x, _, _)| x.is_some()).all(|b| b));
        assert!(found.iter().all(|(_, _, x)| !*x));
    }

    #[test]
    fn test_get_entity_attributes_not_found() {
        let scan_output = get_scan_default_output();
        let found = QueryPolicyDecision::get_entity_attributes(&scan_output.objects, 1);
        assert_eq!(found.len(), 0);
    }

    #[tokio::test]
    async fn test_fill_star_attributes_found() {
        let scan_output = get_scan_star_output();
        let mut attributes = QueryPolicyDecision::get_entity_attributes(&scan_output.objects, 0);

        // initially we have a star
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
        QueryPolicyDecision::fill_star_attributes(&mut attributes, &star_map).await;

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
        let mut attributes = QueryPolicyDecision::get_entity_attributes(&scan_output.objects, 0);
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

    async fn get_defaults() -> (DefaultModel, ScanOutput, CacheContainer) {
        (
            DefaultModel::from_str(ABAC_MODEL).await.unwrap(),
            get_scan_default_output(),
            CacheContainer::new(
                get_user_model_cache(None),
                get_group_model_cache(None),
                get_object_model_cache(None),
                get_entity_model_cache(None),
                get_rules_cache(None),
            ),
        )
    }

    fn get_default_rules_cache() -> (Arc<MultiLayeredCache<u64, CacheEntity>>, CachedAdapter) {
        let rules_cache = Arc::new(MultiLayeredCache::new(
            10,
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
            Box::from(DummyBackendProvider::from(items)),
        )))
    }

    fn get_object_model_cache(
        items: Option<HashMap<String, ObjectModel>>,
    ) -> Arc<Box<MultiLayeredCache<String, ObjectModel>>> {
        let items = if let Some(items) = items {
            items
        } else {
            get_object_model_input()
        };
        Arc::new(Box::new(MultiLayeredCache::new(
            10,
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

    fn get_entity_model_input() -> HashMap<String, EntityModel> {
        let mut entity_model_input = HashMap::new();
        entity_model_input.insert(
            "entity_id".to_string(),
            EntityModel {
                id: "".to_string(),
                full_name: "".to_string(),
                attributes: vec![],
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
                id: "user_id_1".to_string(),
                principal_name: "user_principal_name_1".to_string(),
                role: "user_role_1".to_string(),
                tags: vec!["pii::email".to_string()],
            },
        );
        user_model_input
    }

    fn get_group_model_input() -> HashMap<String, GroupModel> {
        let mut group_model_input = HashMap::new();
        group_model_input.insert(
            "group_id".to_string(),
            GroupModel {
                user_id: "user_id_1".to_string(),
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
        let mut object_model_input = HashMap::new();
        object_model_input.insert(
            "object_id".to_string(),
            ObjectModel {
                id: "object_id".to_string(),
                full_name: "object_full_name_1".to_string(),
                is_aggregated: false,
                last_time_accessed: 0,
                tags: vec!["pii::lastname".to_string()],
            },
        );
        object_model_input
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
