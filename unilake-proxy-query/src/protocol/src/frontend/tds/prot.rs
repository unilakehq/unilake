use crate::frontend::tds::codec::{
    BatchRequest, LoginMessage, PreloginMessage, RpcRequest, TdsBackendResponse, TdsMessage,
    TdsToken,
};
use crate::server::ServerInstance;
use crate::session::SessionInfo;
use async_trait::async_trait;
use futures::{Sink, SinkExt};
use std::{net::SocketAddr, sync::Arc};
use unilake_common::error::{Result, WireError};

#[derive(Debug, Default)]
pub enum TdsSessionState {
    #[default]
    /// Initial State
    Initial,
    /// Received Initial PRELOGIN Packet State
    PreLoginProcessed,
    /// Received TLS/SSL Negotiation Packet State
    SSLNegotiationProcessed,
    /// Received LOGIN7 Record with Complete Authentication Token state
    CompleteLogin7Processed,
    /// Received LOGIN7 Record with SPNEGO Packet State
    Login7SPNEGOProcessed,
    /// Received LOGIN7 Record with Authentication information request.
    Login7FederatedAuthenticationInformationRequestProcessed,
    /// Logged In State
    LoggedIn,
    /// Received Client Request State
    RequestReceived,
    /// Received Attention State
    AttentionReceived,
    /// Indicates that a connection was re-routed to a different SQL Server and transport needs to be re-established
    ReConnect,
    /// Received a final notification to the TDS Server
    LogoutProcessed,
    /// Final State
    Final,
}

#[async_trait]
pub trait TdsWireHandlerFactory<S>: Send + Sync
where
    S: SessionInfo + Send + Sync,
{
    /// Create a new TDS server session
    async fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<S>;

    /// Close TDS server session
    async fn close_session(&self, session: &mut S);

    /// Called when pre-login request arrives
    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        msg: &PreloginMessage,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when login request arrives
    async fn on_login7_request<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        msg: &LoginMessage,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when federated authentication token message arrives. Called only when
    /// such a message arrives in response to federated authentication info, not when the
    /// token is part of a login request.
    fn on_federated_authentication_token_message(&self, session: &S);

    /// Called when RPC request arrives
    async fn on_remote_procedure_call<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        rpc: &RpcRequest,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when SQL batch request arrives
    async fn on_sql_batch_request<C>(
        &self,
        client: &mut C,
        session_info: &mut S,
        msg: &BatchRequest,
    ) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send;

    /// Called when attention arrives
    fn on_attention(&self, session: &S);

    /// Send message to the client
    async fn send_message<C, M>(&self, client: &mut C, msg: M) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
        M: Into<TdsMessage> + Send,
    {
        client
            .send(TdsBackendResponse::Message(msg.into()))
            .await
            .map_err(|_| WireError::Protocol("Failed to feed message".to_string()))
    }

    /// Send token to the client
    async fn send_token<C, T>(&self, client: &mut C, token: T) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
        T: Into<TdsToken> + Send,
    {
        client
            .send(TdsBackendResponse::Token(token.into()))
            .await
            .map_err(|_| WireError::Protocol("Failed to feed token".to_string()))
    }

    /// Flush all results
    async fn flush<C>(&self, client: &mut C) -> Result<()>
    where
        C: Sink<TdsBackendResponse> + Unpin + Send,
    {
        client
            .send(TdsBackendResponse::Done)
            .await
            .map_err(|_| WireError::Protocol("Failed to feed completion".to_string()))
    }
}
