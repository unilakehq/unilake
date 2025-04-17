use crate::frontend::tds::codec::LoginMessage;
use crate::frontend::tds::prot::TdsSessionState;
use crate::sessions::{FeSessionContext, SessionVariable};
use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;

pub struct TdsFeSessionInfo {
    // todo: socketaddr can move to session instead
    socket_addr: SocketAddr,
    state: TdsSessionState,
    packet_size: Arc<AtomicU16>,
    sql_user_id: Option<Arc<str>>,
    connection_reset_request_count: usize,
    login_message: Option<LoginMessage>,
    client_nonce: Option<[u8; 32]>,
    server_nonce: Option<[u8; 32]>,
}

impl TdsFeSessionInfo {
    fn state(&self) -> &TdsSessionState {
        &self.state
    }

    fn set_state(&mut self, new_state: TdsSessionState) {
        self.state = new_state;
    }

    fn packet_size(&self) -> Arc<AtomicU16> {
        self.packet_size.clone()
    }

    fn set_sql_user_id(&mut self, sql_user_id: String) {
        self.sql_user_id = Some(Arc::from(sql_user_id));
    }
}

impl FeSessionContext for TdsFeSessionInfo {
    fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    fn get_username(&self) -> Option<Arc<str>> {
        self.sql_user_id.clone()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_default_variables(&self, variables: &mut HashMap<String, SessionVariable>) {
        todo!()
    }
}
