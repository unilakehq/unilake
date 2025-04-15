use crate::backend::app::{FedResult, FedResultStream, FederatedRequestType, ResultSetBuilder};
use crate::frontend::tds::codec::sqlstring::SqlString;
use crate::frontend::tds::codec::{BatchRequest, ColumnData, DataFlags, TypeInfo};
use async_stream::stream;
use unilake_common::error::Result;

pub(crate) fn process_static(hash: u64, req: &FederatedRequestType) -> Option<FedResultStream> {
    // hash based
    if let FederatedRequestType::Query(req) = req {
        let found = match hash {
            // SELECT SERVERPROPERTY('EngineEdition'), SERVERPROPERTY('productversion'), SERVERPROPERTY ('productlevel'), SERVERPROPERTY ('edition'), SERVERPROPERTY ('MachineName'), SERVERPROPERTY ('ServerName'), (SELECT CASE WHEN EXISTS (SELECT TOP 1 1 from [sys].[all_columns] WITH (NOLOCK) WHERE name = N'xml_index_type' AND OBJECT_ID(N'sys.xml_indexes') = object_id) THEN 1 ELSE 0 END AS SXI_PRESENT)
            10359985016278064883 => Some(engine_edition(req)),
            17700992380341451191 => Some(session_properties(req)),
            5755979048921116848 => Some(databases(req)),
            6768217174072757231 => Some(context_info(req)),
            9848272818868536402 => Some(database_size_info(req)),
            7919239051011949721 => Some(backup_info(req)),
            12637854610589088817 | 804025963826738980 => Some(noop()),
            _ => None,
        };
        if let Some(result_set) = found {
            let stream = stream! {yield result_set;};
            return Some(FedResultStream::new(Box::pin(stream)));
        }

        if hash == 3415367573379425041 {
            return Some(server_edition(req));
        }
    }

    None
    // non-hash
    // toreturn = match req {
    //     n if n.starts_with("set", true) => set_statement(req),
    //     _ => toreturn,
    // };
    //
    // toreturn
}

fn server_edition(_req: &BatchRequest) -> FedResultStream {
    let stream = stream! {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("DatabaseEngineType"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_column(
            Some("DatabaseEngineEdition"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_column(
            Some("ProductVersion"),
            TypeInfo::new_nvarchar(Some(40)),
            DataFlags::default(),
        )
        .add_column(
            Some("MicrosoftVersion"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_row(&[
            ColumnData::I32(1),
            ColumnData::I32(3),
            ColumnData::new_nvarchar(Some("16.0.4140.4".to_string()), Some(255)),
            ColumnData::I32(268439596),
        ]);

        // first resultset
        yield Ok(FedResult::Tabular(result_set.result));

       let result_set = ResultSetBuilder::new()
            .add_column(Some("host_platform"), TypeInfo::new_nvarchar(Some(255)), DataFlags::default())
            .add_row(&[ColumnData::new_nvarchar(Some("Linux".to_string()), Some(255))]);

        // second resultset
        yield Ok(FedResult::Tabular(result_set.result));

        let result_set = ResultSetBuilder::new()
            .add_column(Some("ConnectionProtocol"), TypeInfo::new_nvarchar(Some(255)), DataFlags::default())
            .add_row(&[ColumnData::new_nvarchar(Some("TCP".to_string()), Some(255))]);

        // third resultset
        yield Ok(FedResult::Tabular(result_set.result));
    };

    FedResultStream::new(Box::pin(stream))
}

fn set_statement(req: &BatchRequest) -> Option<FedResult> {
    tracing::info!("Received SET statement: {}", req.query);
    Some(FedResult::Empty)
}

fn noop() -> Result<FedResult> {
    Ok(FedResult::Empty)
}

fn backup_info(_req: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("Within 24hrs"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_column(
            Some("Older than 24hrs"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_column(
            Some("No backup found"),
            TypeInfo::new_intn(false),
            DataFlags::default(),
        )
        .add_row(&[ColumnData::I32(0), ColumnData::I32(0), ColumnData::I32(0)]);

    Ok(FedResult::Tabular(result_set.result))
}

fn database_size_info(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("name"),
            TypeInfo::new_nvarchar(Some(2000)),
            DataFlags::default(),
        )
        .add_column(
            Some("DataFileSizeMB"),
            TypeInfo::new_intn(true),
            DataFlags::default(),
        )
        .add_column(
            Some("LogFileSizeMB"),
            TypeInfo::new_intn(true),
            DataFlags::default(),
        )
        .add_row(&[
            ColumnData::String(SqlString::from_string(Some("default_catalog"), Some(255))),
            ColumnData::I32(0),
            ColumnData::I32(0),
        ]);

    Ok(FedResult::Tabular(result_set.result))
}

fn context_info(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(
            None,
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        .add_row(&[ColumnData::String(SqlString::from_string(None, Some(100)))]);
    Ok(FedResult::Tabular(result_set.result))
}

fn databases(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(
            Some("name"),
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        .add_row(&[ColumnData::String(SqlString::from_string(
            Some("dwh"),
            Some(100),
        ))]);
    Ok(FedResult::Tabular(result_set.result))
}

fn session_properties(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(None, TypeInfo::new_intn(false), DataFlags::default())
        .add_column(None, TypeInfo::new_intn(false), DataFlags::default())
        .add_row(&[ColumnData::I32(1), ColumnData::I32(1)]);

    Ok(FedResult::Tabular(result_set.result))
}

fn engine_edition(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column(None, TypeInfo::new_intn(false), DataFlags::default())
        // todo: 40
        .add_column(
            None,
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        .add_column(
            None,
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        .add_column(
            None,
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        // todo: 4000
        .add_column(
            None,
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        .add_column(
            None,
            TypeInfo::new_nvarchar(Some(100)),
            DataFlags::default(),
        )
        .add_column(None, TypeInfo::new_intn(false), DataFlags::default())
        .add_row(&[
            ColumnData::I32(3),
            ColumnData::String(SqlString::from_string(
                Some("Microsoft SQL Server"),
                Some(100),
            )),
            ColumnData::String(SqlString::from_string(Some("RTM"), Some(100))),
            ColumnData::String(SqlString::from_string(
                Some("Developer Edition (64-bit)"),
                Some(100),
            )),
            ColumnData::String(SqlString::from_string(
                // todo: set server name from context
                Some("8e833a79ef92"),
                Some(100),
            )),
            ColumnData::String(SqlString::from_string(
                // todo: set server name from context
                Some("8e833a79ef92"),
                Some(100),
            )),
            ColumnData::I32(1),
        ]);

    Ok(FedResult::Tabular(result_set.result))
}
