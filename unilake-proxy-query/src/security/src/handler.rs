use crate::caching::layered_cache::MultiLayeredCache;
use crate::policies::{PolicyHitManager, PolicyLogger};
use crate::HitRule;
use casbin::{Cache, CachedEnforcer, CoreApi};
use std::sync::Arc;
use tokio::sync::Mutex;
use ulid::Ulid;
use unilake_common::error::{TdsWireError, TokenError};
use unilake_common::model::{GroupModel, ObjectModel, SessionModel, UserModel};
use unilake_sql::{
    run_scan_operation, run_secure_operation, run_transpile_operation, ParserError, ScanAttribute,
    ScanEntity, ScanOutput, ScanOutputObject, TranspilerInput,
};

pub struct CacheContainer {
    user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
    group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
    object_model_cache: Arc<Box<MultiLayeredCache<String, ObjectModel>>>,
    cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
}

impl CacheContainer {
    pub fn new(
        user_model_cache: Arc<Box<MultiLayeredCache<String, UserModel>>>,
        group_model_cache: Arc<Box<MultiLayeredCache<String, GroupModel>>>,
        object_model_cache: Arc<Box<MultiLayeredCache<String, ObjectModel>>>,
        cached_rules: Arc<Box<dyn Cache<u64, (String, HitRule)>>>,
    ) -> Self {
        Self {
            user_model_cache,
            group_model_cache,
            object_model_cache,
            cached_rules,
        }
    }
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

struct QueryPolicyDecision {}

impl QueryPolicyDecision {
    pub async fn process(
        scan_output: &ScanOutput,
        cache_container: &CacheContainer,
        enforcer: Arc<Mutex<CachedEnforcer>>,
        session_model: &SessionModel,
    ) -> Result<TranspilerInput, SecurityHandlerError> {
        let mut pm = PolicyHitManager::new(cache_container.cached_rules.clone());
        let logger = Box::new(PolicyLogger::new(pm.get_sender()));
        let mut enforcer = enforcer.lock().await;

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

        scan_output
            .objects
            .iter()
            .map(|o| o.scope)
            .for_each(|scope| {
                let items = Self::get_entity_attributes(&scan_output.objects, scope);
                for item in items {
                    if let Some(entity) = item.0 {
                        let object_model = Self::get_object_model(
                            cache_container,
                            format!(
                                "{}.{}.{}.{}",
                                entity.catalog, entity.db, entity.name, item.1.name
                            ),
                        )
                        .await;
                    }
                    if let Err(err) = enforcer.enforce((
                        QueryPolicyDecision::get_user_model(cache_container),
                        QueryPolicyDecision::get_group_model(cache_container),
                        session_model,
                        QueryPolicyDecision::get_object_model(cache_container),
                    )) {
                        panic!("Failed to enforce policy: {}", err);
                    }
                }
            });

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

    fn get_entity_attributes(
        input: &Vec<ScanOutputObject>,
        scope: i32,
    ) -> Vec<(Option<&ScanEntity>, &ScanAttribute)> {
        input
            .iter()
            .filter(|o| o.scope == scope)
            .flat_map(|i| {
                i.attributes
                    .iter()
                    .filter_map(|a| {
                        i.entities
                            .iter()
                            .find(|e| e.name == a.entity)
                            .map(|e| (Some(e), a))
                            // todo: this does not work for star
                            .or_else(|| Some((None, a)))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    async fn get_object_model(cache_container: &CacheContainer, object_key: String) -> ObjectModel {
        cache_container.object_model_cache.get(&object_key).await;
        todo!()
    }

    fn get_group_model(cache_container: &CacheContainer) -> GroupModel {
        todo!()
    }

    fn get_user_model(cache_container: &CacheContainer) -> UserModel {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::cached_adapter::{CacheEntity, CachedAdapter};
    use crate::caching::layered_cache::{BackendProvider, MultiLayeredCache};
    use crate::handler::{CacheContainer, QueryPolicyDecision};
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
        AccountType, GroupInstance, GroupModel, ObjectModel, SessionModel, UserModel,
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

    async fn get_defaults() -> (DefaultModel, ScanOutput, CacheContainer) {
        (
            DefaultModel::from_str(ABAC_MODEL).await.unwrap(),
            get_scan_default_output(),
            CacheContainer::new(
                get_user_model_cache(None),
                get_group_model_cache(None),
                get_object_model_cache(None),
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
                        entity: "customers".to_string(),
                        name: "user_id".to_string(),
                        alias: "uid".to_string(),
                    },
                    ScanAttribute {
                        entity: "customers".to_string(),
                        name: "firstname".to_string(),
                        alias: "firstname".to_string(),
                    },
                    ScanAttribute {
                        entity: "customers".to_string(),
                        name: "lastname".to_string(),
                        alias: "lastname".to_string(),
                    },
                    ScanAttribute {
                        entity: "customers".to_string(),
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
