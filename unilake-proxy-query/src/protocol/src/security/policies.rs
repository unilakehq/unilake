use casbin::{Cache, EventData, Logger};
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

pub(crate) struct PolicyHitLogger {
    sender: Sender<PolicyHit>,
    receiver: Receiver<PolicyHit>,
    logged: Option<Vec<PolicyHit>>,
}

impl PolicyHitLogger {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<PolicyHit>();
        PolicyHitLogger {
            sender,
            receiver,
            logged: None,
        }
    }

    pub fn get_sender(&self) -> Sender<PolicyHit> {
        self.sender.clone()
    }

    pub fn process_hits(&mut self) {
        if self.logged.is_some() {
            return;
        }

        let mut logged = Vec::new();
        while let Ok(hit) = self.receiver.try_recv() {
            logged.push(hit);
        }
        self.logged = Some(logged);
    }

    pub fn get_logged_hits(&mut self) -> Option<Vec<PolicyHit>> {
        self.logged.take()
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

        // todo: use regex instead?
        let object = rvals
            .last()
            .unwrap()
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

// TODO: the policy manager should be responsible for getting the transpiler output and generating output on the proxy endpoint
// features it should support:
// - resolve conflicts (set precedence based on settings)
// - generate request url if some assets are not found or denied (done via the API)
// - update last accessed time for assets
// - allow for impersonation (merge own rules with that of another user, least privileged takes precedence)
// - cache enforcer rules (using ids), cache evict enforcer cache if an enforced rule is missing

pub struct PolicyManager {
    cached: Box<dyn Cache<u64, (String, HitRule)>>, // repo: RepoRest,
}

pub struct PolicyFound {
    expression: String,
    policy_id: String,
}

pub enum PolicyCollectResult {
    /// String = object_id, PolicyRule contains all hit rules
    Found(Vec<(String, Vec<PolicyFound>)>),
    CacheInvalid,
    NotFound,
}

impl PolicyManager {
    pub fn new(cached: Box<dyn Cache<u64, (String, HitRule)>>) -> Self {
        Self { cached }
    }

    pub async fn process(
        &self,
        mut policy_logger: PolicyHitLogger,
    ) -> Result<PolicyCollectResult, String> {
        policy_logger.process_hits();
        let logged_hits = if let Some(logged_hits) = policy_logger.get_logged_hits() {
            logged_hits
        } else {
            return Ok(PolicyCollectResult::NotFound);
        };

        // process cache with new hits
        let mut items = logged_hits.iter().peekable();
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
                        return Ok(PolicyCollectResult::CacheInvalid);
                    }
                }
                _ => {}
            }
        }

        self.process_hits(object_ids).await;
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

    async fn process_hits(&self, object_ids: Vec<String>) {
        // TODO: send data-accessed post request to API for updating last accessed time, lets do this via the proxy channel and async processing
    }

    fn collect_policy(&self, id: &u64) -> Result<(String, Vec<PolicyFound>), String> {
        if let Some(hit) = self.cached.get(&id) {
            let mut policies = Vec::new();
            for rule in hit.1.rules {
                let rule = rule.split(',').collect::<Vec<&str>>();
                if rule.len() < 3 {
                    return Err("invalid policy format".to_string());
                }
                policies.push(PolicyFound {
                    expression: rule[rule.len() - 2].to_string(),
                    policy_id: rule[rule.len() - 1].to_string(),
                })
            }
            return Ok((hit.0.clone(), policies));
        }
        Err("policy not found in cache".to_string())
    }
}
