use crate::frontend::tds::codec::{
    BaseMetaDataColumn, BatchRequest, ColumnData, DataFlags, MetaDataColumn, RpcRequest,
    TokenColMetaData, TokenInfo, TokenRow, TokenSessionState, TypeInfo,
};
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;
use unilake_common::error::Result;

mod app;
mod app_dbeaver;
mod app_pbi;
mod app_ssms;
mod app_unilake;
mod app_vscode;

pub enum FederatedRequestType<'a> {
    Query(&'a BatchRequest),
    Rpc(&'a RpcRequest),
}

pub struct FederatedFrontendHandler {}

impl FederatedFrontendHandler {
    pub fn exec_request(
        hash: u64,
        request: FederatedRequestType,
    ) -> Result<Option<FedResultStream>> {
        let found = app::process_static(hash, &request)
            .or_else(|| app_dbeaver::process_static(hash, &request))
            .or_else(|| app_pbi::process_static(hash, &request))
            .or_else(|| app_ssms::process_static(hash, &request))
            .or_else(|| app_unilake::process_static(hash, &request));

        if found.is_some() {
            tracing::info!("Static query result found for hash: {}", hash);
        }

        Ok(found)
    }
}

pub struct FedResultStream {
    it: Pin<Box<dyn Stream<Item = Result<FedResult>> + Send>>,
}

impl FedResultStream {
    pub fn new(it: Pin<Box<dyn Stream<Item = Result<FedResult>> + Send>>) -> Self {
        Self { it }
    }
}

impl Stream for FedResultStream {
    type Item = Result<FedResult>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.it).poll_next(cx)
    }
}

impl From<&mut ResultSet> for TokenColMetaData {
    fn from(value: &mut ResultSet) -> Self {
        let mut col = TokenColMetaData::new(value.columns.len());
        while let Some(column) = value.columns.pop_front() {
            col.add_column(column);
        }
        col
    }
}

impl Iterator for ResultSet {
    type Item = TokenRow;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.rows.pop_front();
        if let Some(row) = current {
            let mut token_row = TokenRow::new(self.columns.len(), false);
            for item in row {
                token_row.push_row(item);
            }
            Some(token_row)
        } else {
            None
        }
    }
}

// todo(mrhamburg): we need to check where we do the set commands for sessions and which one we support
pub enum FedResult {
    Tabular(ResultSet),
    Info(TokenInfo),
    State(TokenSessionState),
    Empty,
}

pub struct ResultSet {
    columns: VecDeque<MetaDataColumn>,
    rows: VecDeque<VecDeque<ColumnData>>,
}

impl ResultSet {
    pub fn new() -> Self {
        ResultSet {
            columns: VecDeque::new(),
            rows: VecDeque::new(),
        }
    }
}

struct ResultSetBuilder {
    result: ResultSet,
}

impl ResultSetBuilder {
    pub fn new() -> Self {
        ResultSetBuilder {
            result: ResultSet::new(),
        }
    }

    pub fn add_column(
        mut self,
        name: Option<impl ToString>,
        ty: TypeInfo,
        flags: DataFlags,
    ) -> Self {
        self.result.columns.push_back(MetaDataColumn {
            col_name: name.map(|s| s.to_string()).unwrap_or_default(),
            base: BaseMetaDataColumn { flags, ty },
        });

        self
    }

    pub fn add_row(mut self, cells: &[ColumnData]) -> Self {
        self.result.rows.push_back(cells.to_vec().into());
        self
    }
}

impl BatchRequest {
    pub fn contains(&self, keyword: &str, case_insensitive: bool) -> bool {
        if case_insensitive {
            self.query_lowercased
                .contains(keyword.to_lowercase().as_str())
        } else {
            self.query.contains(keyword)
        }
    }

    pub fn starts_with(&self, keyword: &str, case_insensitive: bool) -> bool {
        if case_insensitive {
            self.query_lowercased
                .starts_with(keyword.to_lowercase().as_str())
        } else {
            self.query.starts_with(keyword)
        }
    }
}
