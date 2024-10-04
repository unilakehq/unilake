use crate::frontend::error::TdsWireResult;
use crate::frontend::sqlstring::SqlString;
use crate::frontend::{
    BaseMetaDataColumn, BatchRequest, ColumnData, DataFlags, MetaDataColumn, TokenColMetaData,
    TokenRow, TypeInfo,
};
use std::collections::VecDeque;

pub(crate) fn process(hash: u64, req: &BatchRequest) -> TdsWireResult<Option<ResultSet>> {
    // hash example
    // match hash {
    //     10359985016278064883 => engine_edition(req),
    //     _ => Ok(None),
    // }

    // non-hash, contains example (best to be complimented with ast information?)
    match req {
        n if n.contains("EngineEdition") && n.contains("productversion") => engine_edition(req),
        _ => Ok(None),
    }
}

fn engine_edition(req: &BatchRequest) -> TdsWireResult<Option<ResultSet>> {
    let result_set = ResultSetBuilder::new()
        .add_column(None, TypeInfo::new_int(false), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_nvarchar(100), DataFlags::default())
        .add_column(None, TypeInfo::new_int(false), DataFlags::default())
        .add_row(&[
            ColumnData::I32N(Some(3)),
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
            ColumnData::I32N(Some(1)),
        ]);

    Ok(result_set.into())
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

impl From<ResultSetBuilder> for Option<ResultSet> {
    fn from(value: ResultSetBuilder) -> Self {
        Some(value.result)
    }
}

impl BatchRequest {
    pub fn contains(&self, keyword: &str) -> bool {
        self.query
            .to_uppercase()
            .contains(keyword.to_uppercase().as_str())
    }
}
