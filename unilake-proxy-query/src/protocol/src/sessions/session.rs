use crate::sessions::session_ctx::SessionContext;
use crate::sessions::session_info::SessionProcessInfo;
use crate::sessions::session_mgr::SessionManager;
use crate::sessions::session_status::SessionStatus;
use crate::sessions::session_type::SessionType;
use crate::sessions::SessionVariable;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use ulid::Ulid;

pub struct Session {
    pub(in crate::sessions) ctx: Arc<SessionContext>,
    pub(in crate::sessions) typ: SessionType,
    pub(in crate::sessions) info: RwLock<SessionProcessInfo>,
    status: Arc<RwLock<SessionStatus>>,
}

impl Session {
    pub fn new(typ: SessionType, ctx: SessionContext) -> Self {
        Session {
            typ,
            ctx: Arc::new(ctx),
            status: Arc::new(RwLock::new(SessionStatus::default())),
            info: RwLock::new(SessionProcessInfo::default()),
        }
    }

    pub fn get_session_id(&self) -> Ulid {
        self.ctx.id
    }

    pub fn get_type(&self) -> SessionType {
        self.typ.clone()
    }

    pub fn get_status(&self) -> Arc<RwLock<SessionStatus>> {
        self.status.clone()
    }

    pub fn get_session_context(&self) -> Arc<SessionContext> {
        self.ctx.clone()
    }

    pub fn get_current_catalog(&self) -> Arc<str> {
        self.ctx.get_current_catalog().unwrap_or_default()
    }

    pub fn set_current_catalog(&self, catalog: Arc<str>) {
        self.ctx.set_current_catalog(catalog);
    }

    pub fn get_current_database(&self) -> Arc<str> {
        self.ctx.get_current_database().unwrap_or_default()
    }

    pub fn set_current_database(&self, database: String) {
        self.ctx.set_current_database(Arc::from(database));
    }

    pub fn get_current_query_id(&self) -> Option<String> {
        self.info.read().current_query_id.clone()
    }

    pub fn get_current_user_id(&self) -> Arc<str> {
        // self.ctx.get_current_user_id()
        todo!()
    }

    pub fn get_current_branch_name(&self) -> Arc<str> {
        todo!()
    }

    pub fn set_current_branch_name(&self, branch_name: Arc<str>) -> Arc<str> {
        todo!()
    }

    pub fn get_current_tenant_id(&self) -> Arc<str> {
        self.ctx.get_current_tenant_id().unwrap_or_default()
    }

    pub fn set_current_tenant_id(&self, tenant_id: Arc<str>) {
        self.ctx.set_current_tenant_id(tenant_id);
    }

    pub fn set_current_user_id(&self, user_id: Arc<str>) -> Arc<str> {
        todo!()
    }

    pub fn get_all_variables(&self) -> HashMap<String, SessionVariable> {
        self.ctx.get_session_variables()
    }

    pub fn get_compute_id(&self) -> Arc<str> {
        todo!()
    }

    pub fn set_compute_id(&self, compute_id: Arc<str>) -> Arc<str> {
        todo!()
    }

    pub fn get_workspace_id(&self) -> Arc<str> {
        todo!()
    }

    pub fn set_workspace_id(&self, workspace_id: Arc<str>) -> Arc<str> {
        todo!()
    }

    pub fn get_domain_id(&self) -> Arc<str> {
        todo!()
    }

    pub fn set_domain_id(&self, domain_id: Arc<str>) -> Arc<str> {
        todo!()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let session_id = self.get_session_id();
        if let Err(e) = SessionManager::instance().destroy_session(session_id) {
            tracing::error!("Error when destroying session({}): {}", e, session_id);
        }
    }
}
