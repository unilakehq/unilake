use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use casbin::{Cache, EventData, Logger};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum PolicyHit {
    HitEnforce(HitEnforce),
    HitRule(HitRule),
}

#[derive(Debug)]
pub(crate) struct HitEnforce {
    id: u64,
    object_id: String,
    #[allow(dead_code)]
    authorized: bool,
    #[allow(dead_code)]
    cached: bool,
}

#[derive(Debug, Clone)]
pub struct HitRule {
    pub id: u64,
    pub rules: Vec<String>,
}

#[derive(PartialEq, Clone)]
pub enum PolicyType {
    MaskingRule,
    FilterRule,
    FullAccess,
    None,
}

#[derive(Clone)]
pub struct PolicyFound {
    pub rule_definition: serde_json::Value,
    pub policy_name: Option<String>,
    pub policy_type: PolicyType,
    pub policy_id: String,
    pub effect: String,
}

impl PolicyFound {
    pub fn get_name(&self) -> &str {
        self.policy_name.as_ref().map(|s| s.as_str()).unwrap_or("")
    }
}

pub enum PolicyCollectResult {
    /// String = object_id, PolicyRule contains all hit rules
    Found(Vec<(String, Vec<PolicyFound>)>),
    /// The current cache is invalid, clear cache and try again
    CacheInvalid,
    /// No item found in the cache
    NotFound,
}

pub(crate) struct PolicyHitManager {
    sender: Sender<PolicyHit>,
    receiver: Receiver<PolicyHit>,
    cached: Arc<Box<dyn Cache<u64, (String, HitRule)>>>, // repo: RepoRest,
}

impl PolicyHitManager {
    pub fn new(cached: Arc<Box<dyn Cache<u64, (String, HitRule)>>>) -> Self {
        let (sender, receiver) = channel::<PolicyHit>();
        PolicyHitManager {
            sender,
            receiver,
            cached,
        }
    }

    pub fn get_sender(&self) -> Sender<PolicyHit> {
        self.sender.clone()
    }

    pub fn process_hits(&mut self) -> Result<PolicyCollectResult, String> {
        let mut logged = Vec::new();
        while let Ok(hit) = self.receiver.try_recv() {
            logged.push(hit);
        }

        if logged.is_empty() {
            return Ok(PolicyCollectResult::NotFound);
        }

        // process cache with new hits
        let mut items = logged.iter().peekable();
        let mut object_ids = Vec::new();
        let mut hits = Vec::with_capacity(items.len());
        while let Some(hit) = items.next() {
            match (hit, items.peek()) {
                (PolicyHit::HitEnforce(enforce), Some(PolicyHit::HitRule(ref rule))) => {
                    self.register_rule(enforce, rule.clone());
                    hits.push(rule.id);
                    object_ids.push(enforce.object_id.clone());
                }
                (PolicyHit::HitEnforce(enforce), _) => {
                    if let Some((object_id, rule)) = self.cached.get(&enforce.id) {
                        hits.push(rule.id);
                        object_ids.push(object_id);
                    } else if enforce.authorized {
                        // if this happens, clear cache and retry
                        return Ok(PolicyCollectResult::CacheInvalid);
                    } else {
                        // todo: authorized is false, we need to treat this as an unauthorized access request
                        // todo: this should then be an automated hidden policy?
                        // or just report no access to this attribute id
                    }
                }
                _ => {}
            }
        }

        let mut toreturn = Vec::with_capacity(hits.len());
        for id in hits {
            let policy = self.collect_policy(&id)?;
            toreturn.push(policy);
        }

        Ok(PolicyCollectResult::Found(toreturn))
    }

    fn register_rule(&self, enforce: &HitEnforce, rule: HitRule) {
        if enforce.id != rule.id {
            panic!("rule id mismatch");
        }
        self.cached.set(rule.id, (enforce.object_id.clone(), rule));
    }

    fn collect_policy(&self, id: &u64) -> Result<(String, Vec<PolicyFound>), String> {
        if let Some(hit) = self.cached.get(&id) {
            let mut policies = Vec::new();
            for rule in hit.1.rules {
                let rule = rule.split(',').collect::<Vec<&str>>();
                let len = rule.len();
                if len < 3 {
                    return Err("invalid policy format".to_string());
                }

                let rule_definition = self.parse_rule_definition(rule[len - 2])?;
                let (policy_name, policy_type) =
                    self.get_policy_type_and_name_from_rule_definition(&rule_definition)?;
                let effect = rule[len - 3].trim().to_string();

                policies.push(PolicyFound {
                    policy_id: rule[len - 1].trim_start().to_string(),
                    rule_definition,
                    policy_type,
                    policy_name,
                    effect,
                })
            }
            return Ok((hit.0.clone(), policies));
        }
        // this can technically never happen, since the cache is validated beforehand
        Err("policy not found in cache".to_string())
    }

    fn parse_rule_definition(&self, rule_definition: &str) -> Result<serde_json::Value, String> {
        let rule = BASE64_STANDARD
            .decode(rule_definition.trim())
            .map_err(|e| format!("Error while base64 decoding string: {}", e.to_string()))?;
        let rule = String::from_utf8(rule).map_err(|e| {
            format!(
                "Error while parsing string from decoded base64 message {}",
                e.to_string()
            )
        })?;
        Ok(serde_json::from_str(&rule).map_err(|e| {
            format!(
                "Error while parsing rule definition to json: {}, input: {}",
                e.to_string(),
                rule_definition
            )
        })?)
    }

    fn get_policy_type_and_name_from_rule_definition(
        &self,
        rule_definition: &serde_json::Value,
    ) -> Result<(Option<String>, PolicyType), String> {
        if let Some(name) = rule_definition.get("name") {
            if let serde_json::Value::String(name) = name {
                return Ok((Some(name.clone()), PolicyType::MaskingRule));
            }
            return Err("policy name is not a string".to_string());
        } else if let Some(_) = rule_definition.get("expression") {
            return Ok((None, PolicyType::FilterRule));
        } else if let Some(is_full_access) = rule_definition.get("full_access") {
            if is_full_access.as_bool().unwrap_or(false) {
                return Ok((Some("full_access".to_owned()), PolicyType::FullAccess));
            }
        }
        Ok((None, PolicyType::None))
    }

    /// Resolves any conflicts when multiple policies are found, returns None in case no policies are provided or
    /// when the only policies found are those that provide full access.
    pub fn resolve_masking_policy_conflict(
        policies: &Vec<PolicyFound>,
        prio_stricter: bool,
    ) -> Option<&PolicyFound> {
        fn resolve_masking_prio<'b>(
            policies: &Vec<&'b PolicyFound>,
            prio_stricter: bool,
        ) -> Option<&'b PolicyFound> {
            let mut chosen = (if prio_stricter { u16::MAX } else { u16::MIN }, None);
            for policy in policies.iter().filter(|policy| {
                policy.policy_type == PolicyType::MaskingRule
                    || policy.policy_type == PolicyType::FullAccess
            }) {
                let name = policy
                    .policy_name
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let prio = match name {
                    // highly constraint
                    "hidden" => 0,
                    "replace_null" => 10,
                    "replace_string" => 20,
                    "replace_char" => 30,

                    // masking based
                    "xxhash3" => 110,
                    "mask_except_last" => 120,
                    "mask_except_first" => 130,

                    // masking with preserving information
                    "ip_anonymize" => 210,
                    "mail_hash_pres" => 220,
                    "mail_mask_pres" => 230,
                    "cc_hash_pres" => 240,
                    "cc_mask_pres" => 250,
                    "cc_last_four" => 260,
                    "ip_hash_pres" => 270,
                    "ip_mask_pres" => 280,
                    "mail_mask_username" => 290,
                    "mail_mask_domain" => 291,

                    // somewhat constraint
                    "left" => 310,
                    "right" => 320,

                    // less constraint
                    "rounding" => 410,
                    "date_year_only" => 420,
                    "date_month_only" => 430,

                    // unknown constraint level
                    "custom" => 998,
                    "full_access" => 999,
                    &_ => continue,
                };

                if prio < chosen.0 && prio_stricter {
                    chosen = (prio, Some(policy));
                } else if prio > chosen.0 && !prio_stricter {
                    chosen = (prio, Some(policy));
                }
            }

            match chosen.1 {
                Some(policy) if policy.policy_type == PolicyType::FullAccess => None, // for full access, no masking to apply
                Some(policy) => Some(policy),
                None => None,
            }
        }

        // resolution per policy_id
        resolve_masking_prio(
            &policies
                .iter()
                .fold(HashMap::new(), |mut map, policy| {
                    let item = map.entry(policy.policy_id.clone()).or_insert(Vec::new());
                    item.push(policy);
                    map
                })
                .iter()
                // for policy conflicts within a policy_id, prioritize the policy with higher priority
                .filter_map(|(_, i)| resolve_masking_prio(i, true))
                .collect(),
            prio_stricter,
        )
    }
}

pub(crate) struct PolicyLogger {
    enabled: bool,
    sender: Sender<PolicyHit>,
}

impl PolicyLogger {
    pub fn new(sender: Sender<PolicyHit>) -> Self {
        Self {
            enabled: false,
            sender,
        }
    }

    fn get_object_id(&self, rvals: &Vec<String>) -> String {
        rvals
            .get(rvals.len() - 2)
            .unwrap_or(&",,,unknown".to_string())
            .split(',')
            .skip(2)
            .take(1)
            .collect::<String>()
            .split(':')
            .last()
            .unwrap()
            .replace("\"", "")
            .trim()
            .to_string()
    }
}

impl Logger for PolicyLogger {
    fn enable_log(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn print_enforce_log(&self, rvals: &Vec<String>, authorized: bool, cached: bool) {
        let mut hasher = DefaultHasher::new();
        rvals.hash(&mut hasher);

        let object_id = self.get_object_id(rvals);
        self.sender
            .send(PolicyHit::HitEnforce(HitEnforce {
                authorized,
                cached,
                id: hasher.finish(),
                object_id,
            }))
            .unwrap();
    }

    fn print_mgmt_log(&self, _d: &EventData) {
        // not implemented
    }

    fn print_explain_log(&self, rvals: &Vec<String>, rules: Vec<String>) {
        let mut hasher = DefaultHasher::new();
        rvals.hash(&mut hasher);

        self.sender
            .send(PolicyHit::HitRule(HitRule {
                id: hasher.finish(),
                rules,
            }))
            .unwrap();
    }

    fn print_status_log(&self, _enabled: bool) {
        // not implemented
    }
}

#[cfg(test)]
mod tests {
    use crate::policies::{
        PolicyCollectResult, PolicyFound, PolicyHit, PolicyHitManager, PolicyLogger, PolicyType,
    };
    use crate::HitRule;
    use casbin::{Cache, Logger};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn test_policy_hit_evals_extract_object_id() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let sut = PolicyLogger::new(sender);
        send_test_data(sut);

        // check captured logs
        let mut hits = Vec::new();
        while let Ok(hit) = receiver.recv() {
            hits.push(hit);
        }

        let testcase = hits.first().unwrap();
        if let PolicyHit::HitEnforce(hit) = testcase {
            assert_eq!(hit.object_id, "some_object_id".to_string());
        } else {
            panic!("Expected HitEnforce")
        }
    }

    #[test]
    fn test_policy_logger_capture_hit_and_enforce() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let sut = PolicyLogger::new(sender);
        send_test_data(sut);

        // check captured logs
        let mut hits = Vec::new();
        while let Ok(hit) = receiver.recv() {
            hits.push(hit);
        }

        assert_eq!(hits.len(), 6);
    }

    #[test]
    fn test_policy_hit_manager_results() {
        let cache: Arc<Box<dyn Cache<u64, (String, HitRule)>>> =
            Arc::new(Box::new(casbin::DefaultCache::new(400)));
        let mut sut = PolicyHitManager::new(cache);
        {
            let logger = PolicyLogger::new(sut.get_sender());
            send_test_data(logger);
        }

        let start = std::time::Instant::now();
        let results = sut.process_hits().unwrap();
        let end = std::time::Instant::now();
        println!("Processing hits took: {:?}", end.duration_since(start));
        if let PolicyCollectResult::Found(policies) = results {
            assert_eq!(policies.len(), 3);
        } else {
            panic!("Expected PolicyCollectResult::Found");
        }
    }

    #[test]
    fn test_policy_conflict_precedent_loosely() {
        let items = get_test_data_policies_found();
        let found = PolicyHitManager::resolve_masking_policy_conflict(&items, false);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.policy_id, "2");
    }

    #[test]
    fn test_policy_conflict_precedent_strict() {
        let items = get_test_data_policies_found();
        let found = PolicyHitManager::resolve_masking_policy_conflict(&items, true);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.policy_id, "0");
    }

    #[test]
    fn test_policy_conflict_precedent_full_access() {
        let mut items = Vec::new();
        items.push(PolicyFound {
            policy_id: "99".to_string(),
            policy_type: PolicyType::FullAccess,
            policy_name: None,
            rule_definition: json!({}),
            effect: "allow".to_string(),
        });
        let found = PolicyHitManager::resolve_masking_policy_conflict(&items, true);
        assert!(found.is_none());
    }

    #[test]
    fn test_policy_conflict_precedent_no_full_access() {
        let mut items = get_test_data_policies_found();
        items.push(PolicyFound {
            policy_id: "99".to_string(),
            policy_type: PolicyType::FullAccess,
            policy_name: None,
            rule_definition: json!({}),
            effect: "allow".to_string(),
        });
        let found = PolicyHitManager::resolve_masking_policy_conflict(&items, true);
        assert!(found.is_some());
        assert_ne!(found.unwrap().policy_id, "99");
        assert!(found.unwrap().policy_type != PolicyType::FullAccess);
    }

    fn send_test_data(mut logger: PolicyLogger) {
        logger.enable_log(true);

        // send log events
        // Masking example
        let rvals = vec![
            r##"#{"accountType": "User", "id": "some_id", "principalName": "alice", "role": "", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"entityVersion": 0, "groups": [#{"id": "some_id", "tags": ["pii::username", "pii::email"]}], "userId": "some_id"}"##.to_string(),
            r##"#{"appDriver": "", "appId": 0, "appName": "", "appType": "", "branch": "", "computeId": "", "continent": "", "countryIso2": "", "dayOfWeek": 0, "domainId": "", "id": "some_id", "policyId": "", "sourceIpv4": "", "time": 0, "timezone": "", "workspaceId": ""}"##.to_string(),
            r##"#{"dataType": "INT", "fullName": "some.schema.table.column", "id": "some_object_id", "isAggregated": false, "name":"column", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"no_name": #{"expire_datetime_utc": 0, "normalized_name": "no_name", "policy_id": "policy_id_2", "prio_strict": true}}"##.to_string()
        ];
        logger.print_enforce_log(&rvals, true, false);
        logger.print_explain_log(&rvals, vec!["p, some.*, TagExists(r.user, \"Hello\") && 1 == 1 && TagExists(r.group, \"Something\"), allow, eyJuYW1lIjogInh4aGFzaDMiLCAicHJvcGVydGllcyI6IG51bGx9, policy_id_1".to_string()]);

        // Filter example
        let rvals = vec![
            r##"#{"accountType": "User", "id": "some_id", "principalName": "alice", "role": "", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"entityVersion": 0, "groups": [#{"id": "some_id", "tags": ["pii::username", "pii::email"]}], "userId": "some_id"}"##.to_string(),
            r##"#{"appDriver": "", "appId": 0, "appName": "", "appType": "", "branch": "", "computeId": "", "continent": "", "countryIso2": "", "dayOfWeek": 0, "domainId": "", "id": "some_id", "policyId": "", "sourceIpv4": "", "time": 0, "timezone": "", "workspaceId": ""}"##.to_string(),
            r##"#{"dataType": "INT", "fullName": "some.schema.table.column", "id": "another_object_id", "isAggregated": false, "name":"column", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"no_name": #{"expire_datetime_utc": 0, "normalized_name": "no_name", "policy_id": "policy_id_2", "prio_strict": true}}"##.to_string()
        ];
        logger.print_enforce_log(&rvals, true, false);
        logger.print_explain_log(&rvals, vec!["p, some.*, TagExists(r.user, \"Hello\") && 1 == 1 && TagExists(r.group, \"Something\"), allow, eyJleHByZXNzaW9uIjogIj8gPCAxMDAwIn0=, policy_id_2".to_string()]);

        // Allow example
        let rvals = vec![
            r##"#{"accountType": "User", "id": "some_id", "principalName": "alice", "role": "", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"entityVersion": 0, "groups": [#{"id": "some_id", "tags": ["pii::username", "pii::email"]}], "userId": "some_id"}"##.to_string(),
            r##"#{"appDriver": "", "appId": 0, "appName": "", "appType": "", "branch": "", "computeId": "", "continent": "", "countryIso2": "", "dayOfWeek": 0, "domainId": "", "id": "some_id", "policyId": "", "sourceIpv4": "", "time": 0, "timezone": "", "workspaceId": ""}"##.to_string(),
            r##"#{"dataType": "INT", "fullName": "some.schema.table.column", "id": "allow_object_id", "isAggregated": false, "name":"column", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"no_name": #{"expire_datetime_utc": 0, "normalized_name": "no_name", "policy_id": "policy_id_2", "prio_strict": true}}"##.to_string()
        ];
        logger.print_enforce_log(&rvals, true, false);
        logger.print_explain_log(&rvals, vec!["p, some.*, TagExists(r.user, \"Hello\") && 1 == 1 && TagExists(r.group, \"Something\"), allow, e30=, policy_id_2".to_string()]);
    }

    fn get_test_data_policies_found() -> Vec<PolicyFound> {
        vec![
            PolicyFound {
                policy_id: "0".to_string(),
                policy_name: Some("xxhash3".to_string()),
                policy_type: PolicyType::MaskingRule,
                rule_definition: serde_json::value::Value::Null,
                effect: "allow".to_string(),
            },
            PolicyFound {
                policy_id: "1".to_string(),
                policy_name: Some("mail_mask_username".to_string()),
                policy_type: PolicyType::MaskingRule,
                rule_definition: serde_json::value::Value::Null,
                effect: "allow".to_string(),
            },
            PolicyFound {
                policy_id: "2".to_string(),
                policy_name: Some("rounding".to_string()),
                policy_type: PolicyType::MaskingRule,
                rule_definition: serde_json::value::Value::Null,
                effect: "allow".to_string(),
            },
        ]
    }
}
