use crate::frontend::sqlstring::SqlString;
use crate::frontend::{
    BaseMetaDataColumn, BatchRequest, ColumnData, DataFlags, MetaDataColumn, TokenColMetaData,
    TokenInfo, TokenRow, TokenSessionState, TypeInfo,
};
use std::collections::VecDeque;

pub(crate) fn process_static(hash: u64, req: &BatchRequest) -> Option<FedResult> {
    // hash example
    let mut toreturn = match hash {
        10359985016278064883 => Some(FedResult::Tabular(engine_edition(req))),
        17700992380341451191 => Some(FedResult::Tabular(session_properties(req))),
        5755979048921116848 => Some(FedResult::Tabular(databases(req))),
        6768217174072757231 => Some(FedResult::Tabular(context_info(req))),
        9848272818868536402 => Some(FedResult::Tabular(database_size_info(req))),
        7919239051011949721 => Some(FedResult::Tabular(backup_info(req))),
        _ => None,
    };

    // non-hash
    toreturn = match req {
        n if n.starts_with("set", true) => set_statement(req),
        _ => toreturn,
    };

    toreturn
}

fn set_statement(req: &BatchRequest) -> Option<FedResult> {
    tracing::info!("Received SET statement: {}", req.query);
    Some(FedResult::Empty)
}

fn backup_info(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("Within 24hrs"),
            TypeInfo::new_int(false),
            DataFlags::default(),
        )
        .add_column(
            Some("Older than 24hrs"),
            TypeInfo::new_int(false),
            DataFlags::default(),
        )
        .add_column(
            Some("No backup found"),
            TypeInfo::new_int(false),
            DataFlags::default(),
        )
        .add_row(&[ColumnData::I32(0), ColumnData::I32(0), ColumnData::I32(0)]);

    result_set.result
}

fn database_size_info(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("name"),
            TypeInfo::new_nvarchar(255),
            DataFlags::default(),
        )
        .add_column(
            Some("DataFileSizeMB"),
            TypeInfo::new_int(false),
            DataFlags::default(),
        )
        .add_column(
            Some("LogFileSizeMB"),
            TypeInfo::new_int(false),
            DataFlags::default(),
        )
        .add_row(&[
            ColumnData::String(SqlString::from_string(
                Some("default_catalog".to_string()),
                255,
            )),
            ColumnData::I32(0),
            ColumnData::I32(0),
        ]);

    result_set.result
}

fn context_info(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_row(&[ColumnData::String(SqlString::from_string(None, 100))]);
    result_set.result
}

fn databases(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("name"),
            TypeInfo::new_nvarchar(100),
            DataFlags::default(),
        )
        .add_row(&[ColumnData::String(SqlString::from_string(
            Some("dwh".to_string()),
            100,
        ))]);
    result_set.result
}

fn session_properties(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(None, TypeInfo::new_int(false), DataFlags::default())
        .add_column(None, TypeInfo::new_int(false), DataFlags::default())
        .add_row(&[ColumnData::I32(1), ColumnData::I32(1)]);

    result_set.result
}

fn hello_world(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(Some("Hello"), TypeInfo::new_bit(), DataFlags::default())
        .add_column(Some("World"), TypeInfo::new_bit(), DataFlags::default())
        .add_row(&[ColumnData::Bit(true), ColumnData::Bit(false)]);
    result_set.result
}

fn engine_edition(req: &BatchRequest) -> ResultSet {
    let result_set = ResultSetBuilder::new()
        .add_column(None, TypeInfo::new_int(false), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_int(false), DataFlags::default())
        .add_row(&[
            ColumnData::I32(3),
            ColumnData::String(SqlString::from_string(
                Some("Microsoft SQL Server".to_string()),
                256,
            )),
            ColumnData::String(SqlString::from_string(Some("RTM".to_string()), 256)),
            ColumnData::String(SqlString::from_string(
                Some("Developer Edition (64-bit)".to_string()),
                256,
            )),
            ColumnData::String(SqlString::from_string(
                Some("8e833a79ef92".to_string()),
                256,
            )),
            ColumnData::String(SqlString::from_string(
                Some("8e833a79ef92".to_string()),
                256,
            )),
            ColumnData::I32(1),
        ]);

    result_set.result
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

    pub fn add_column(mut self, name: Option<&str>, ty: TypeInfo, flags: DataFlags) -> Self {
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
