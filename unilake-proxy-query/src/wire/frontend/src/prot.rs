use futures::future;
use tokio::sync::RwLock;

use crate::codec::TdsWireError;
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Default)]
pub enum TdsSessionState {
    #[default]
    /// Sent Initial PRELOGIN Packet State
    PreLoginSent,
    /// Sent TLS/SSL Negotiation Packet State
    SSLNegotiationSent,
    /// Sent LOGIN7 Record with Complete Authentication Token state
    CompleteLogin7Sent,
    /// Sent LOGIN7 Record with SPNEGO Packet State
    Login7SPNEGOSent,
    /// Sent LOGIN7 Record with Authentication information request.
    Login7FederatedAuthenticationInformationRequestSent,
    /// Logged In State
    LoggedIn,
    /// Sent Client Request State
    RequestSent,
    /// Sent Attention State
    AttentionSent,
    /// Indicates that a connection was re-routed to a different SQL Server and transport needs to be re-established
    ReConnect,
    /// Sent a final notification to the TDS Server
    LogoutSent,
    /// Final State
    Final,
}

pub trait SessionInfo {
    /// Currently in use socket
    fn socket_addr(&self) -> SocketAddr;

    /// Current session state
    fn state(&self) -> &TdsSessionState;

    /// Mutate current session state
    fn set_state(&mut self, new_state: TdsSessionState);

    /// Session identifier
    fn session_id() -> usize;

    /// Size of the TDS packet
    fn packet_size() -> usize;

    /// User name if SQL authentication is used
    fn sql_user_id() -> String;

    /// Database to which connection is established
    fn database() -> String;

    /// TDS version of the communication
    fn tds_version() -> String;

    /// Counter of connection reset requests for this session
    fn connection_reset_request_count() -> usize;
}

pub struct DefaultSession {
    socket_addr: SocketAddr,
    state: TdsSessionState,
    session_id: usize,
    packet_size: usize,
    sql_user_id: usize,
    database: Option<String>,
    tds_version: String,
}

impl SessionInfo for DefaultSession {
    fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    fn state(&self) -> &TdsSessionState {
        &self.state
    }

    fn set_state(&mut self, new_state: TdsSessionState) {
        self.state = new_state
    }

    fn session_id() -> usize {
        todo!()
    }

    fn packet_size() -> usize {
        todo!()
    }

    fn sql_user_id() -> String {
        todo!()
    }

    fn database() -> String {
        todo!()
    }

    fn tds_version() -> String {
        todo!()
    }

    fn connection_reset_request_count() -> usize {
        todo!()
    }
}

pub struct TdsWireMessageServerCodec {
    pub client_info: DefaultSession,
}

impl DefaultSession {
    pub fn new(socket_addr: SocketAddr) -> Self {
        DefaultSession {
            socket_addr,
            packet_size: 1200,
            session_id: 1,
            sql_user_id: 0,
            tds_version: "".to_string(),
            state: TdsSessionState::default(),
            database: None,
        }
    }
}

pub trait TdsWireHandlerFactory<S>
where
    S: SessionInfo,
{
    /// Create a new TDS server session
    fn open_session(
        &self,
        socker_addr: &SocketAddr,
        instance_info: &ServerInstance,
    ) -> Result<S, TdsWireError>;

    /// Close TDS server session
    fn close_session(&self, session: &S);

    /// Called when pre-login request arrives
    fn on_prelogin_request(&self, session: &S);

    /// Called when login request arrives
    fn on_login7_request(&self, session: &S);

    /// Called when federated authentication token message arrives. Called only when
    /// such a message arrives in response to federated authentication info, not when the
    /// token is part of a login request.
    fn on_federated_authentication_token_message(&self, session: &S);

    /// Called when SQL batch request arrives
    fn on_sql_batch_request(&self, session: &S);

    /// Called when attention arrives
    fn on_attention(&self, session: &S);
}

pub enum ServerInstanceMessage {
    IncrementSessionCounter,
    DecrementSessionCounter,
}

pub struct ServerInstance {
    active_sessions: usize,
    session_limit: usize,
    #[allow(dead_code)]
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>>,
    sender: Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>>,
}

impl ServerInstance {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ServerInstanceMessage>();
        ServerInstance {
            active_sessions: 0,
            session_limit: 32767,
            receiver: Some(receiver),
            sender: Arc::new(sender),
        }
    }

    pub fn start_instance(&mut self, instance: Arc<RwLock<Self>>) -> tokio::task::JoinHandle<()> {
        async fn run(
            instance: Arc<RwLock<ServerInstance>>,
            mut receiver: tokio::sync::mpsc::UnboundedReceiver<ServerInstanceMessage>,
        ) {
            while let Some(r) = receiver.recv().await {
                let mut rwself = instance.write().await;
                match r {
                    ServerInstanceMessage::IncrementSessionCounter => rwself.active_sessions += 1,
                    ServerInstanceMessage::DecrementSessionCounter => rwself.active_sessions -= 1,
                }
            }
        }
        if self.receiver.is_none() {
            panic!("server instance is already started")
        }
        let r = self.receiver.take().unwrap();
        tokio::spawn(async move { run(instance, r).await })
    }

    pub fn active_session_count(&self) -> usize {
        self.active_sessions
    }

    pub fn session_limit(&self) -> usize {
        self.session_limit
    }

    pub fn get_sender(&self) -> Arc<tokio::sync::mpsc::UnboundedSender<ServerInstanceMessage>> {
        self.sender.clone()
    }
}
