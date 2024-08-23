use async_trait::async_trait;
use futures::Sink;
use std::{net::SocketAddr, sync::Arc};

use crate::frontend::{
    error::{TdsWireError, TdsWireResult},
    prot::{DefaultSession, ServerInstance, SessionInfo, TdsWireHandlerFactory},
    BatchRequest, LoginMessage, PreloginMessage, TdsBackendResponse,
};

type StarRocksSession = DefaultSession;

pub struct StarRocksTdsHandlerFactory {
    // pools: Arc<Vec<(Ulid, Pool)>>,
}

impl StarRocksTdsHandlerFactory {
    // fn get_pool(&self, ulid: &Ulid) -> Option<&Pool> {
    //     self.pools.iter().find(|(id, _)| id == ulid).map(|_, p| p)
    // }
}

#[async_trait]
impl TdsWireHandlerFactory<StarRocksSession> for StarRocksTdsHandlerFactory {
    fn open_session(
        &self,
        socket_addr: &SocketAddr,
        instance_info: Arc<ServerInstance>,
    ) -> Result<StarRocksSession, TdsWireError> {
        todo!()
    }

    fn close_session(&self, session: &StarRocksSession) {
        todo!()
    }

    async fn on_prelogin_request<C>(
        &self,
        client: &mut C,
        msg: &PreloginMessage,
    ) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        todo!()
    }

    async fn on_login7_request<C>(&self, client: &mut C, msg: &LoginMessage) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        todo!()
    }

    fn on_federated_authentication_token_message(&self, session: &StarRocksSession) {
        todo!()
    }

    async fn on_sql_batch_request<C>(&self, client: &mut C, msg: &BatchRequest) -> TdsWireResult<()>
    where
        C: SessionInfo + Sink<TdsBackendResponse> + Unpin + Send,
    {
        todo!()
    }

    fn on_attention(&self, session: &StarRocksSession) {
        todo!()
    }
}
