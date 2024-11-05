use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Hash)]
#[allow(unused)]
pub enum AccountType {
    User,
    Group,
}

#[derive(Serialize, Deserialize, Hash)]
pub struct UserModel<'a> {
    /// User id
    pub id: &'a str,
    /// The user's name (like: menno@something.com)
    #[serde(rename = "principalName")]
    pub principal_name: &'a str,
    /// The user's role (as part of this connection)
    pub role: &'a str,
    /// Tags associated with the user, a tag: some_category::some_tag
    pub tags: Vec<&'a str>,
    /// The type of the user (User or Group)
    #[serde(rename = "accountType")]
    pub account_type: AccountType,
}

#[derive(Serialize, Deserialize, Hash)]
pub struct GroupModel<'a> {
    /// The user id these groups belong to
    #[serde(rename = "userId")]
    pub user_id: &'a str,
    #[serde(rename = "entityVersion")]
    pub entity_version: u32,
    /// The groups this user is member of
    pub groups: Vec<GroupInstance<'a>>,
}

#[derive(Serialize, Deserialize, Hash)]
pub struct GroupInstance<'a> {
    /// The group id
    pub id: &'a str,
    /// Tags associated with the group, a tag: some_category::some_tag
    pub tags: Vec<&'a str>,
}

#[derive(Serialize)]
pub struct SessionModel<'a> {
    /// Unique session id
    pub id: &'a str,
    /// The application id
    #[serde(rename = "appId")]
    pub app_id: u64,
    /// The application name
    #[serde(rename = "appName")]
    pub app_name: &'a str,
    /// The application type
    #[serde(rename = "appType")]
    pub app_type: &'a str,
    /// The application driver
    #[serde(rename = "appDriver")]
    pub app_driver: &'a str,
    /// The source IP address of this session (v4)
    #[serde(rename = "sourceIpv4")]
    pub source_ipv4: &'a str,
    /// The source country name (ISO 3166-1 alpha-2) of the source IP address
    #[serde(rename = "countryIso2")]
    pub country_iso2: &'a str,
    /// The name of the continent (AF, AN, AS, EU, NA, OC, SA)
    pub continent: &'a str,
    /// The source timezone name (e.g., "America/New_York") of the source IP address
    pub timezone: &'a str,
    /// Current timestamp in UTC
    pub time: &'a u32,
    /// Current day of the week (0 is Monday) in UTC
    #[serde(rename = "dayOfWeek")]
    pub day_of_week: &'a u8,
    /// The branch name of the connection
    pub branch: &'a str,
    /// The compute id of the connection
    #[serde(rename = "computeId")]
    pub compute_id: &'a str,
    /// The policy id in use (by including this, all previously cached results will be invalidated on change)
    #[serde(rename = "policyId")]
    pub policy_id: &'a str,
    /// The workspace id of the connection
    #[serde(rename = "workspaceId")]
    pub workspace_id: &'a str,
    /// The domain id of the connection
    #[serde(rename = "domainId")]
    pub domain_id: &'a str,
}

impl<'a> Hash for SessionModel<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app_id.hash(state);
        self.app_name.hash(state);
        self.app_type.hash(state);
        self.app_driver.hash(state);
        self.source_ipv4.hash(state);
        self.country_iso2.hash(state);
        self.continent.hash(state);
        self.timezone.hash(state);
        self.day_of_week.hash(state);
        self.branch.hash(state);
        self.policy_id.hash(state);
        self.workspace_id.hash(state);
        self.domain_id.hash(state);
    }
}

#[derive(Serialize, Deserialize)]
pub struct ObjectModel<'a> {
    /// The object id
    pub id: &'a str,
    /// The full name of the object (e.g., catalog.schema.table.column)
    pub full_name: &'a str,
    /// Tags associated with the object, a tag: some_category::some_tag
    pub tags: Vec<&'a str>,
    /// The last time this object was accessed by the current user
    pub last_time_accessed: u64,
    /// If true, this object is being aggregated
    pub is_aggregated: bool,
}

impl<'a> Hash for ObjectModel<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.tags.hash(state);
        self.is_aggregated.hash(state);
    }
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
