use crate::caching::layered_cache::MultiLayeredCache;
use crate::policies::{PolicyHitManager, PolicyLogger};
use crate::repository::RepoRest;
use crate::HitRule;
use casbin::{Cache, CachedEnforcer, CoreApi};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;
use ulid::Ulid;
use unilake_common::error::{TdsWireError, TokenError};
use unilake_common::model::{EntityModel, GroupModel, ObjectModel, SessionModel, UserModel};
use unilake_sql::{
    run_scan_operation, run_secure_operation, run_transpile_operation, Catalog, ParserError,
    ScanAttribute, ScanEntity, ScanOutput, ScanOutputObject, TranspilerInput, VisibleSchemaBuilder,
};

pub struct CacheContainer {
    user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
    group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
    entity_model_cache: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
    object_model_cache: Arc<Box<MultiLayeredCache<String, ObjectModel>>>,
    cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
}

impl CacheContainer {
    pub fn new(
        user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
        group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
        object_model_cache: Arc<Box<MultiLayeredCache<String, ObjectModel>>>,
        entity_model_cache: Arc<Box<MultiLayeredCache<String, EntityModel>>>,
        cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    ) -> Self {
        Self {
            user_model_cache,
            group_model_cache,
            object_model_cache,
            cached_rules,
            entity_model_cache,
        }
    }
}

pub enum SecurityHandlerResults {
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
    cache_container: CacheContainer,
    session_model: SessionModel,
}

impl SecurityHandler {
    pub fn new(
        cached_enforcer: Arc<Mutex<CachedEnforcer>>,
        cache_container: CacheContainer,
        session_model: SessionModel,
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
            cache_container,
            session_model,
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

        // todo(mrhamburg): pdp has state and caching and all, needs to be improved
        let transpiler_input = QueryPolicyDecision::process(
            &scan_output,
            &self.cache_container,
            self.cached_enforcer.clone(),
            &self.session_model,
        )
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

struct QueryPolicyDecision<'a> {
    cache_container: &'a CacheContainer,
    enforcer: Arc<Mutex<CachedEnforcer>>,
    session_model: &'a SessionModel,
    repo_rest: RepoRest,
}

impl<'a> QueryPolicyDecision<'a> {
    pub fn new(
        cache_container: &'a CacheContainer,
        enforcer: Arc<Mutex<CachedEnforcer>>,
        session_model: &'a SessionModel,
        repo_rest: RepoRest,
    ) -> Self {
        QueryPolicyDecision {
            cache_container,
            enforcer,
            session_model,
            repo_rest,
        }
    }

    pub async fn process(
        &self,
        scan_output: &ScanOutput,
    ) -> Result<TranspilerInput, SecurityHandlerError> {
        let mut pm = PolicyHitManager::new(self.cache_container.cached_rules.clone());
        let logger = Box::new(PolicyLogger::new(pm.get_sender()));
        let mut enforcer = self.enforcer.lock().await;

        // add dependencies and reload policy
        enforcer.set_logger(logger);
        enforcer.enable_log(true);
        if let Err(err) = enforcer.load_policy().await {
            panic!("Failed to load policy: {}", err);
        }

        // todo: build set to iterate over (scoped, per attribute)
        // todo: take into account stars, we need to to unstar them for all schema records one is allowed to access
        // (no need for deny except for full table deny, since a star is a request for all accessible attributes)

        // todo: on failure, an access request needs to be formulated (either for access or since a deny rule has been hit)

        // todo: the visible schema also needs to be built, we actually need this beforehand

        // scan_output
        //     .objects
        //     .iter()
        //     .map(|o| o.scope)
        //     .for_each(|scope| {
        //         let items = Self::get_entity_attributes(&scan_output.objects, scope);
        //         for item in items {
        //             if let Some(entity) = item.0 {
        //                 let object_model = Self::get_object_model(
        //                     cache_container,
        //                     format!(
        //                         "{}.{}.{}.{}",
        //                         entity.catalog, entity.db, entity.name, item.1.name
        //                     ),
        //                 )
        //                 .await;
        //             }
        //             if let Err(err) = enforcer.enforce((
        //                 QueryPolicyDecision::get_user_model(cache_container),
        //                 QueryPolicyDecision::get_group_model(cache_container),
        //                 session_model,
        //                 QueryPolicyDecision::get_object_model(cache_container),
        //             )) {
        //                 panic!("Failed to enforce policy: {}", err);
        //             }
        //         }
        //     });

        pm.process_hits();

        Ok(TranspilerInput {
            cause: None,
            query: scan_output.query.clone().unwrap(),
            request_url: None,
            rules: vec![],
            filters: vec![],
            visible_schema: None,
        })
    }

    /// Extracts the combinations entity and attribute from the given scope
    /// Returned tuple: Optional Entity, Attribute, Star flag (defaults false)
    fn get_entity_attributes(
        input: &Vec<ScanOutputObject>,
        scope: i32,
    ) -> Vec<(Option<&ScanEntity>, &ScanAttribute, bool)> {
        input
            .iter()
            .filter(|o| o.scope == scope)
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
    ) -> Result<(), SecurityHandlerResults> {
        let mut star_attributes = Vec::new();
        for item in items.iter_mut().find(|(_, att, _)| att.name == "*") {
            let (entity, _, _) = item;
            // create new elements based on the matching ones
            if let Some(entity) = entity {
                let full_name = entity.get_full_name();
                match found_attributes.get(&full_name) {
                    None => {
                        return Err(SecurityHandlerResults::EntityNotFound(full_name));
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

    async fn get_star_attributes(
        cache: &CacheContainer,
        full_entity_name: String,
        entity_alias: &str,
    ) -> Result<(EntityModel, Vec<ScanAttribute>), SecurityHandlerResults> {
        //todo: entitymodel can be used for building the schema and scanattributes is for knowing which attributes to check (casbin check)
        if let Some(entity) = cache.entity_model_cache.get(&full_entity_name).await {
            let mut attributes = Vec::new();
            for attr in &entity.attributes {
                let full_entity_name = format!("{}.{}", full_entity_name, attr.0);
                if let Some(_) = cache.object_model_cache.get(&full_entity_name).await {
                    attributes.push(ScanAttribute {
                        entity_alias: entity_alias.to_owned(),
                        name: attr.0.to_owned(),
                        alias: attr.0.to_owned(),
                    });
                } else {
                    return Err(SecurityHandlerResults::EntityNotFound(full_entity_name));
                }
            }

            Ok((entity, attributes))
        } else {
            Err(SecurityHandlerResults::EntityNotFound(full_entity_name))
        }
    }

    fn get_visible_schema(entities: &Vec<EntityModel>) -> Option<HashMap<String, Catalog>> {
        let mut builder = VisibleSchemaBuilder::new();
        for model in entities {
            let table = builder
                .get_or_add_catalog(model.get_catalog_name()?)
                .get_or_add_database(model.get_schema_name()?)
                .get_or_add_table(model.get_table_name()?);
            for (n, t) in &model.attributes {
                table.get_or_add_column(n.to_owned(), t.to_owned());
            }
        }
        Some(builder.catalog)
    }

    async fn get_object_model(
        &self,
        object_key: String,
    ) -> Result<ObjectModel, SecurityHandlerResults> {
        let mut found = self
            .cache_container
            .object_model_cache
            .get(&object_key)
            .await;
        if found.is_none() {
            found = Some(
                self.repo_rest
                    .fetch_objectmodel(object_key.as_str())
                    .await
                    .map_err(|e| SecurityHandlerResults::RepositoryError(e))?,
            );
        }
        match found {
            Some(model) => Ok(model),
            None => Err(SecurityHandlerResults::EntityNotFound(object_key)),
        }
    }

    async fn get_entity_model(
        &self,
        entity_key: String,
    ) -> Result<EntityModel, SecurityHandlerResults> {
        let mut found = self
            .cache_container
            .entity_model_cache
            .get(&entity_key)
            .await;
        if found.is_none() {
            found = Some(
                self.repo_rest
                    .fetch_entitymodel(entity_key.as_str())
                    .await
                    .map_err(|e| SecurityHandlerResults::RepositoryError(e))?,
            );
        }
        match found {
            Some(model) => Ok(model),
            None => Err(SecurityHandlerResults::UserGroupsNotFound(entity_key)),
        }
    }

    async fn get_group_model(
        &self,
        group_user_key: String,
    ) -> Result<GroupModel, SecurityHandlerResults> {
        let mut found = self
            .cache_container
            .group_model_cache
            .get(&group_user_key)
            .await;
        if found.is_none() {
            found = Some(
                self.repo_rest
                    .fetch_groupmodel(group_user_key.as_str())
                    .await
                    .map_err(|e| SecurityHandlerResults::RepositoryError(e))?,
            );
        }
        match found {
            Some(model) => Ok(model),
            None => Err(SecurityHandlerResults::UserGroupsNotFound(group_user_key)),
        }
    }

    async fn get_user_model(
        &self,
        user_object_key: String,
    ) -> Result<UserModel, SecurityHandlerResults> {
        let mut found = self
            .cache_container
            .user_model_cache
            .get(&user_object_key)
            .await;
        if found.is_none() {
            found = Some(
                self.repo_rest
                    .fetch_usermodel(user_object_key.as_str())
                    .await
                    .map_err(|e| SecurityHandlerResults::RepositoryError(e))?,
            );
        }
        match found {
            Some(model) => Ok(model),
            None => Err(SecurityHandlerResults::UserGroupsNotFound(user_object_key)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::cached_adapter::{CacheEntity, CachedAdapter};
    use crate::caching::layered_cache::{BackendProvider, MultiLayeredCache};
    use crate::handler::{CacheContainer, QueryPolicyDecision, SecurityHandlerResults};
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

        let result =
            QueryPolicyDecision::process(&scan_output, &cache_container, e.clone(), &session_model)
                .await;
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
                SecurityHandlerResults::EntityNotFound(v) => {
                    assert_eq!("catalog.schema.customers", v)
                }
                SecurityHandlerResults::EntityNotAllowed(_) => unreachable!(),
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
