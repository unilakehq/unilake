use casbin::rhai;
use casbin::rhai::{Array, ImmutableString, Map, INT};
use chrono::{DateTime, Utc};

#[allow(dead_code)]
pub(crate) fn add_functions(engine: &mut rhai::Engine) {
    engine.register_fn("TagExists", tag_exists_1);
    engine.register_fn("TagExists", tag_exists_2);
    engine.register_fn("MemberOfGroup", member_of_group);
    engine.register_fn("NotMemberOfGroup", not_member_of_group);
    engine.register_fn("ConnectedDomain", connected_domain);
    engine.register_fn("ConnectedWorkspace", connected_workspace);
    engine.register_fn("TimeBetween", time_between);
}

#[allow(dead_code)]
fn connected_domain(s: Map, domain_id: ImmutableString) -> bool {
    s.get("domainId").is_some_and(|v| {
        v.clone()
            .try_cast::<ImmutableString>()
            .is_some_and(|v| v == domain_id)
    })
}

#[allow(dead_code)]
fn connected_workspace(s: Map, workspace_id: ImmutableString) -> bool {
    s.get("workspaceId").is_some_and(|v| {
        v.clone()
            .try_cast::<ImmutableString>()
            .is_some_and(|v| v == workspace_id)
    })
}

// Inner get (helper function)
#[allow(dead_code)]
fn inner_tag_exists(s: Array, tag_name: ImmutableString) -> bool {
    let starred = tag_name.ends_with("*");
    s.iter().any(|v| {
        if let Some(v) = v.clone().try_cast::<ImmutableString>() {
            return if starred {
                v.split_once("::").unwrap().0 == tag_name.split_once("::").unwrap().0
            } else {
                v == tag_name
            };
        }
        false
    })
}

// Check if a tag exists in a user or object
#[allow(dead_code)]
fn tag_exists_1(s: Map, tag_name: ImmutableString) -> bool {
    s.get("tags")
        .and_then(|v| Some(v.clone().cast::<Array>()))
        .iter()
        .any(|v| inner_tag_exists(v.clone(), tag_name.clone()))
}

// Check if a tag exists in a group
#[allow(dead_code)]
fn tag_exists_2(s: Map, group_id: ImmutableString, tag_name: ImmutableString) -> bool {
    get_group_maps(s)
        .filter_map(|group| {
            if let Some(id) = group.get("guid") {
                if let Some(id) = id.clone().try_cast::<ImmutableString>() {
                    if id != group_id {
                        return None;
                    }
                }
            }
            group
                .get("tags")
                .map(|v| v.clone().try_cast::<Array>())
                .unwrap()
        })
        .any(|tags| inner_tag_exists(tags, tag_name.clone()))
}

// Check if a user is a member of a group
fn member_of_group(s: Map, group_id: ImmutableString) -> bool {
    get_group_maps(s).any(|group| {
        if let Some(id) = group.get("id") {
            if let Some(id) = id.clone().try_cast::<ImmutableString>() {
                return id == group_id;
            }
        }
        false
    })
}

#[allow(dead_code)]
fn get_group_maps(s: Map) -> impl Iterator<Item = Map> {
    s.get("groups")
        .and_then(|v| v.clone().try_cast::<Array>())
        .into_iter()
        .flatten()
        .filter_map(|group| group.clone().try_cast::<Map>())
}

// Check if a user is not a member of a group
#[allow(dead_code)]
fn not_member_of_group(s: Map, group_name: ImmutableString) -> bool {
    !member_of_group(s, group_name)
}

#[allow(dead_code)]
fn time_between(t: ImmutableString, from: INT, to: INT) -> INT {
    let from = DateTime::<Utc>::from_timestamp(from as i64, 0).unwrap_or_default();
    let to = DateTime::<Utc>::from_timestamp(to as i64, 0).unwrap_or_default();

    let x: i64 = match t.as_str() {
        "weeks" => (to - from).num_weeks(),
        "days" => (to - from).num_days(),
        "hours" => (to - from).num_hours(),
        "minutes" => (to - from).num_minutes(),
        _ => 0,
    };

    INT::try_from(x).unwrap_or_default().abs()
}

#[cfg(test)]
mod tests {
    use crate::functions::add_functions;
    use casbin::rhai;
    use casbin::rhai::serde::to_dynamic;
    use casbin::rhai::{Scope, INT};
    use unilake_common::model::{
        AccountType, GroupInstance, GroupModel, ObjectModel, SessionModel, UserModel,
    };

    #[test]
    fn test_tag_exists_static_object() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let object_model = ObjectModel {
            id: "some_id".to_string(),
            full_name: "".to_string(),
            tags: vec!["pii:username".to_string(), "pii::email".to_string()],
            last_time_accessed: 0,
            is_aggregated: false,
        };

        let mut scope = Scope::new();
        let value = to_dynamic(object_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "TagExists(a, \"pii::email\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_tag_exists_static_user() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let user_object = UserModel {
            id: "some_id".to_string(),
            principal_name: "".to_string(),
            role: "".to_string(),
            tags: vec!["pii::email".to_string()],
            account_type: AccountType::User,
        };

        let mut scope = Scope::new();
        let value = to_dynamic(user_object).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "TagExists(a, \"pii::email\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_tag_exists_star_user() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let user_object = UserModel {
            id: "some_id".to_string(),
            principal_name: "".to_string(),
            role: "".to_string(),
            tags: vec!["pii::email".to_string()],
            account_type: AccountType::User,
        };

        let mut scope = Scope::new();
        let value = to_dynamic(user_object).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "TagExists(a, \"pii::*\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_tag_exists_static_group() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let group_model = GroupModel {
            user_id: "user_id".to_string(),
            entity_version: 0,
            groups: vec![GroupInstance {
                id: "group_id".to_string(),
                tags: vec!["pii::email".to_string()],
            }],
        };

        let mut scope = Scope::new();
        let value = to_dynamic(group_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "TagExists(a, \"group_id\", \"pii::email\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_tag_exists_static_star() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let group_model = GroupModel {
            user_id: "user_id".to_string(),
            entity_version: 0,
            groups: vec![GroupInstance {
                id: "group_id".to_string(),
                tags: vec!["pii::email".to_string()],
            }],
        };

        let mut scope = Scope::new();
        let value = to_dynamic(group_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "TagExists(a, \"group_id\", \"pii::*\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_member_of_group_exists() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let group_model = GroupModel {
            user_id: "some_id".to_string(),
            entity_version: 0,
            groups: vec![GroupInstance {
                id: "some_id".to_string(),
                tags: vec!["pii::email".to_string()],
            }],
        };

        let mut scope = Scope::new();
        let value = to_dynamic(group_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "MemberOfGroup(a, \"some_id\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_member_of_group_not_exists() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let group_model = GroupModel {
            user_id: "some_id".to_string(),
            entity_version: 0,
            groups: vec![GroupInstance {
                id: "some_id".to_string(),
                tags: vec!["pii::email".to_string()],
            }],
        };

        let mut scope = Scope::new();
        let value = to_dynamic(group_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "MemberOfGroup(a, \"another_id\")")
            .unwrap();
        assert_eq!(result, false)
    }

    #[test]
    fn test_between_number_of_days() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let result: INT = engine
            .eval("TimeBetween(\"hours\", 1730299362, 1730302962)")
            .unwrap();
        assert_eq!(result, 1)
    }

    #[test]
    fn test_connected_domain_correct() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let mut session_model = get_session_model();
        session_model.domain_id = "some_correct_guid".to_string();

        let mut scope = Scope::new();
        let value = to_dynamic(session_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "ConnectedDomain(a, \"some_correct_guid\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_connected_domain_incorrect() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let mut session_model = get_session_model();
        session_model.domain_id = "some_correct_guid".to_string();

        let mut scope = Scope::new();
        let value = to_dynamic(session_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "ConnectedDomain(a, \"some_incorrect_guid\")")
            .unwrap();
        assert!(!result)
    }

    #[test]
    fn test_connected_workspace_correct() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let mut session_model = get_session_model();
        session_model.workspace_id = "some_correct_guid".to_string();

        let mut scope = Scope::new();
        let value = to_dynamic(session_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "ConnectedWorkspace(a, \"some_correct_guid\")")
            .unwrap();
        assert!(result)
    }

    #[test]
    fn test_connected_workspace_incorrect() {
        let mut engine = rhai::Engine::new();
        add_functions(&mut engine);

        let mut session_model = get_session_model();
        session_model.workspace_id = "some_incorrect_guid".to_string();

        let mut scope = Scope::new();
        let value = to_dynamic(session_model).unwrap();
        scope.push("a", value);

        let result: bool = engine
            .eval_with_scope(&mut scope, "ConnectedWorkspace(a, \"some_correct_guid\")")
            .unwrap();
        assert!(!result)
    }

    fn get_session_model() -> SessionModel {
        SessionModel {
            id: "".to_string(),
            app_id: 0,
            app_name: "".to_string(),
            app_type: "".to_string(),
            app_driver: "".to_string(),
            source_ipv4: "".to_string(),
            country_iso2: "".to_string(),
            continent: "".to_string(),
            timezone: "".to_string(),
            time: 0,
            day_of_week: 0,
            branch: "".to_string(),
            compute_id: "".to_string(),
            policy_id: "".to_string(),
            workspace_id: "".to_string(),
            domain_id: "some_correct_guid".to_string(),
        }
    }
}
