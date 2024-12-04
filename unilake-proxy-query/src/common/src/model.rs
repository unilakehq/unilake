use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Hash, Clone)]
#[allow(unused)]
pub enum AccountType {
    User,
    Group,
}

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct UserModel {
    /// User id
    pub id: String,
    /// The user's name (like: menno@something.com)
    #[serde(rename = "principalName")]
    pub principal_name: String,
    /// The user's role (as part of this connection)
    pub roles: Vec<String>,
    /// Tags associated with the user, a tag: some_category::some_tag
    pub tags: Vec<String>,
    /// The type of the user (User or Group)
    #[serde(rename = "accountType")]
    pub account_type: AccountType,
    /// The user's access policies. These are the access policies this user has.
    /// This can be used for checking the last accessed time and date for a policy
    /// so policies can be expired
    #[serde(rename = "accessPolicyIds")]
    pub access_policy_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct GroupModel {
    /// The user id these groups belong to
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "entityVersion")]
    pub entity_version: u32,
    /// The groups this user is member of
    pub groups: Vec<GroupInstance>,
}

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct GroupInstance {
    /// The group id
    pub id: String,
    /// Tags associated with the group, a tag: some_category::some_tag
    pub tags: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct SessionModel {
    /// Unique user id
    pub user_id: String,
    /// Optionally the user id you would like to impersonate
    pub impersonate_user_id: Option<String>,
    /// Unique session id
    pub id: String,
    /// The application id
    #[serde(rename = "appId")]
    pub app_id: u64,
    /// The application name
    #[serde(rename = "appName")]
    pub app_name: String,
    /// The application type
    #[serde(rename = "appType")]
    pub app_type: String,
    /// The application driver
    #[serde(rename = "appDriver")]
    pub app_driver: String,
    /// The source IP address of this session (v4)
    #[serde(rename = "sourceIpv4")]
    pub source_ipv4: String,
    /// The source country name (ISO 3166-1 alpha-2) of the source IP address
    #[serde(rename = "countryIso2")]
    pub country_iso2: String,
    /// The name of the continent (AF, AN, AS, EU, NA, OC, SA)
    pub continent: String,
    /// The source timezone name (e.g., "America/New_York") of the source IP address
    pub timezone: String,
    /// Current timestamp in UTC
    pub time: i64,
    /// Current day of the week (0 is Monday) in UTC
    #[serde(rename = "dayOfWeek")]
    pub day_of_week: u32,
    /// The branch name of the connection
    pub branch: String,
    /// The compute id of the connection
    #[serde(rename = "computeId")]
    pub compute_id: String,
    /// The policy id in use/currently active (by including this, all previously cached results will be invalidated on change)
    /// Policy id here refers to the current policy file for this tenant
    #[serde(rename = "policyId")]
    pub policy_id: String,
    /// The workspace id of the connection
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    /// The domain id of the connection
    #[serde(rename = "domainId")]
    pub domain_id: String,
}

impl Hash for SessionModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app_id.hash(state);
        self.app_name.hash(state);
        self.app_type.hash(state);
        self.app_driver.hash(state);
        self.source_ipv4.hash(state);
        self.day_of_week.hash(state);
        self.branch.hash(state);
        self.policy_id.hash(state);
        self.workspace_id.hash(state);
        self.domain_id.hash(state);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectModel {
    /// The object id
    pub id: String,
    /// The full name of the object (e.g., catalog.schema.table.column)
    pub full_name: String,
    /// Tags associated with the object, a tag: some_category::some_tag
    pub tags: Vec<String>,
    /// If true, this object is being aggregated (needs to be updated on execution)
    pub is_aggregated: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EntityModel {
    /// The entity id
    pub id: String,
    /// The full name of the object (e.g., catalog.schema.table)
    pub full_name: String,
    /// Attribute names and types of the object [(a, INT), (b, VARCHAR)]
    pub attributes: Vec<(String, String)>,
    /// Object models for this entity (full_name, object_model)
    pub objects: HashMap<String, ObjectModel>,
}

impl EntityModel {
    pub fn get_catalog_name(&self) -> Option<String> {
        Some(self.full_name.split('.').nth(0)?.to_string())
    }

    pub fn get_schema_name(&self) -> Option<String> {
        Some(self.full_name.split('.').nth(1)?.to_string())
    }

    pub fn get_table_name(&self) -> Option<String> {
        Some(self.full_name.split('.').nth(2)?.to_string())
    }
}

impl Hash for ObjectModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.tags.hash(state);
        self.is_aggregated.hash(state);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct AccessPolicyModel {
    /// The normalized name of the access policy (tenant_name + workspace_name + policy_name).
    /// Used as an attribute name when evaluating policy access times
    pub normalized_name: String,
    /// The access_policy id
    pub policy_id: String,
    /// if true, this policy priorities stricter rules when conflicting with other policies
    pub prio_strict: bool,
    /// Date and time when this policy will expire (in UTC unix timestamp).
    /// This value is maintained by the PIP endpoint (every time this object is updated, so will this value)
    pub expire_datetime_utc: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct IpInfoModel {
    pub ip_v4: String,
    pub country_iso2: String,
    pub continent: String,
    pub timezone: String,
    pub isp: String,
    pub city: String,
    pub country_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct AppInfoModel {
    pub app_id: u64,
    pub app_name: String,
    pub app_type: String,
    pub app_driver: String,
}

#[derive(Serialize)]
pub struct DataAccessRequest {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "securityPolicyId")]
    pub security_policy_id: String,
}

#[derive(Deserialize)]
pub struct DataAccessRequestResponse {
    pub message: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyRule {
    #[serde(rename = "t")]
    policy_type: String,
    #[serde(rename = "o")]
    object: String,
    #[serde(rename = "s")]
    sub_rule: String,
    #[serde(rename = "e")]
    eft: String,
    #[serde(rename = "f")]
    func: String,
    #[serde(rename = "i")]
    policy_id: String,
}

impl PolicyRule {
    pub fn new(
        policy_type: &str,
        object: &str,
        sub_rule: &str,
        eft: &str,
        func: &str,
        policy_id: &str,
    ) -> Self {
        PolicyRule {
            policy_type: policy_type.to_owned(),
            object: object.to_owned(),
            sub_rule: sub_rule.to_owned(),
            eft: eft.to_owned(),
            func: func.to_owned(),
            policy_id: policy_id.to_owned(),
        }
    }

    pub fn to_vec(self) -> Vec<String> {
        vec![
            self.policy_type,
            self.object,
            self.sub_rule,
            self.eft,
            self.func,
            self.policy_id,
        ]
    }
}
