use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use casbin::{Cache, EventData, Logger};
use std::cmp::PartialEq;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub(crate) enum PolicyHit {
    HitEnforce(HitEnforce),
    HitRule(HitRule),
}

#[derive(Debug)]
pub(crate) struct HitEnforce {
    id: u64,
    object_id: String,
    authorized: bool,
    cached: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct HitRule {
    id: u64,
    rules: Vec<String>,
}

#[derive(PartialEq)]
pub enum PolicyType {
    MaskingRule,
    FilterRule,
    None,
}

pub struct PolicyFound {
    rule_definition: serde_json::Value,
    policy_name: Option<String>,
    policy_type: PolicyType,
    policy_id: String,
}

pub enum PolicyCollectResult {
    /// String = object_id, PolicyRule contains all hit rules
    Found(Vec<(String, Vec<PolicyFound>)>),
    CacheInvalid,
    NotFound,
}

pub(crate) struct PolicyHitManager {
    sender: Sender<PolicyHit>,
    receiver: Receiver<PolicyHit>,
    cached: Box<dyn Cache<u64, (String, HitRule)>>, // repo: RepoRest,
}

impl PolicyHitManager {
    pub fn new(cached: Box<dyn Cache<u64, (String, HitRule)>>) -> Self {
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
                    } else {
                        // if this happens, clear cache and retry
                        return Ok(PolicyCollectResult::CacheInvalid);
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

                policies.push(PolicyFound {
                    policy_id: rule[len - 1].to_string(),
                    rule_definition,
                    policy_type,
                    policy_name,
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
                "Error while parsing rule definition to json: {}",
                e.to_string()
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
        }
        Ok((None, PolicyType::None))
    }

    pub fn resolve_masking_policy_conflict(
        policies: &Vec<PolicyFound>,
        prio_stricter: bool,
    ) -> Option<&PolicyFound> {
        let mut chosen = (if prio_stricter { u16::MAX } else { u16::MIN }, None);
        for policy in policies
            .iter()
            .filter(|policy| policy.policy_type == PolicyType::MaskingRule)
        {
            let name = policy
                .policy_name
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("");

            let prio = match name {
                // highly constraint
                "hidden" => 0, // todo: technically hidden column is also a form of masking, check how to add
                "replace_null" => 10,
                "replace_string" => 20,
                "replace_char" => 30,

                // masking based
                "xxhash3" => 110,
                "mask_except_last" => 120,
                "mask_except_first" => 130,

                // Masking with preserving information
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

                // Somewhat constraint
                "left" => 310,
                "right" => 320,

                // Randomized
                "random_number" => 410,
                "random_multiplication" => 420,

                // less constraint
                "rounding" => 510,
                "date_year_only" => 520,
                "date_month_only" => 530,
                &_ => continue,
            };

            if prio < chosen.0 && prio_stricter {
                chosen = (prio, Some(policy));
            } else if prio > chosen.0 && !prio_stricter {
                chosen = (prio, Some(policy));
            }
        }
        chosen.1
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

        let object = rvals
            .last()
            .unwrap_or(&",,unknown".to_string())
            .split(',')
            .skip(1)
            .take(1)
            .collect::<String>()
            .split(':')
            .last()
            .unwrap()
            .replace("\"", "")
            .trim()
            .to_string();

        self.sender
            .send(PolicyHit::HitEnforce(HitEnforce {
                authorized,
                cached,
                id: hasher.finish(),
                object_id: object,
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
        PolicyCollectResult, PolicyFound, PolicyHitManager, PolicyLogger, PolicyType,
    };
    use casbin::Logger;

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
        let cache = Box::new(casbin::DefaultCache::new(400));
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

    fn send_test_data(mut logger: PolicyLogger) {
        logger.enable_log(true);

        // send log events
        // Masking example
        let rvals = vec![
            r##"#{"accountType": "User", "id": "some_id", "principalName": "alice", "role": "", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"entityVersion": 0, "groups": [#{"id": "some_id", "tags": ["pii::username", "pii::email"]}], "userId": "some_id"}"##.to_string(),
            r##"#{"appDriver": "", "appId": 0, "appName": "", "appType": "", "branch": "", "computeId": "", "continent": "", "countryIso2": "", "dayOfWeek": 0, "domainId": "", "id": "some_id", "policyId": "", "sourceIpv4": "", "time": 0, "timezone": "", "workspaceId": ""}"##.to_string(),
            r##"#{"full_name": "some.schema.column", "id": "some_object_id", "is_aggregated": false, "last_time_accessed": 0, "tags": ["pii::username", "pii::email"]}"##.to_string()
        ];
        logger.print_enforce_log(&rvals, true, false);
        logger.print_explain_log(&rvals, vec!["p, some.*, TagExists(r.user, \"Hello\") && 1 == 1 && TagExists(r.group, \"Something\"), allow, eyJuYW1lIjogInh4aGFzaDMiLCAicHJvcGVydGllcyI6IG51bGx9, policy_id_1".to_string()]);

        // Filter example
        let rvals = vec![
            r##"#{"accountType": "User", "id": "some_id", "principalName": "alice", "role": "", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"entityVersion": 0, "groups": [#{"id": "some_id", "tags": ["pii::username", "pii::email"]}], "userId": "some_id"}"##.to_string(),
            r##"#{"appDriver": "", "appId": 0, "appName": "", "appType": "", "branch": "", "computeId": "", "continent": "", "countryIso2": "", "dayOfWeek": 0, "domainId": "", "id": "some_id", "policyId": "", "sourceIpv4": "", "time": 0, "timezone": "", "workspaceId": ""}"##.to_string(),
            r##"#{"full_name": "some.schema.column", "id": "another_object_id", "is_aggregated": false, "last_time_accessed": 0, "tags": ["pii::username", "pii::email"]}"##.to_string()
        ];
        logger.print_enforce_log(&rvals, true, false);
        logger.print_explain_log(&rvals, vec!["p, some.*, TagExists(r.user, \"Hello\") && 1 == 1 && TagExists(r.group, \"Something\"), allow, eyJleHByZXNzaW9uIjogIj8gPCAxMDAwIn0=, policy_id_2".to_string()]);

        // Allow example
        let rvals = vec![
            r##"#{"accountType": "User", "id": "some_id", "principalName": "alice", "role": "", "tags": ["pii::username", "pii::email"]}"##.to_string(),
            r##"#{"entityVersion": 0, "groups": [#{"id": "some_id", "tags": ["pii::username", "pii::email"]}], "userId": "some_id"}"##.to_string(),
            r##"#{"appDriver": "", "appId": 0, "appName": "", "appType": "", "branch": "", "computeId": "", "continent": "", "countryIso2": "", "dayOfWeek": 0, "domainId": "", "id": "some_id", "policyId": "", "sourceIpv4": "", "time": 0, "timezone": "", "workspaceId": ""}"##.to_string(),
            r##"#{"full_name": "some.schema.column", "id": "allow_object_id", "is_aggregated": false, "last_time_accessed": 0, "tags": ["pii::username", "pii::email"]}"##.to_string()
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
            },
            PolicyFound {
                policy_id: "1".to_string(),
                policy_name: Some("mail_mask_username".to_string()),
                policy_type: PolicyType::MaskingRule,
                rule_definition: serde_json::value::Value::Null,
            },
            PolicyFound {
                policy_id: "2".to_string(),
                policy_name: Some("random_number".to_string()),
                policy_type: PolicyType::MaskingRule,
                rule_definition: serde_json::value::Value::Null,
            },
        ]
    }
}
