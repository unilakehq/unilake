use crate::sessions::Session;
use async_trait::async_trait;
use opensrv_mysql::{AsyncMysqlShim, ParamParser, QueryResultWriter, StatementMetaWriter};
use std::sync::Arc;
use tokio::io::AsyncWrite;
use unilake_common::error::Result;
use unilake_common::error_code::ErrorCode;

struct MysqlHandlerFactory {
    session: Option<Arc<Session>>,
}

impl MysqlHandlerFactory {
    fn new(session: Option<Arc<Session>>) -> Self {
        MysqlHandlerFactory { session }
    }
}

#[async_trait]
impl<W: AsyncWrite + Send + Unpin> AsyncMysqlShim<W> for MysqlHandlerFactory {
    type Error = ErrorCode;

    async fn on_prepare<'a>(
        &'a mut self,
        query: &'a str,
        info: StatementMetaWriter<'a, W>,
    ) -> Result<()> {
        todo!()
    }

    async fn on_execute<'a>(
        &'a mut self,
        id: u32,
        params: ParamParser<'a>,
        results: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        todo!()
    }

    async fn on_close<'a>(&'a mut self, stmt: u32)
    where
        W: 'async_trait,
    {
        todo!()
    }

    async fn on_query<'a>(
        &'a mut self,
        query: &'a str,
        results: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        todo!()
    }
}
