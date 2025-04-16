use crate::backend::app::{FedResult, FedResultStream, FederatedRequestType, ResultSetBuilder};
use crate::frontend::tds::codec::column_data::{ColumnData, SqlString};
use crate::frontend::tds::codec::token::DataFlags;
use crate::frontend::tds::codec::{BatchRequest, RpcRequest, TypeInfo};
use async_stream::stream;
use unilake_common::error::Result;
use unilake_common::EMPTY_STRING;

// todo: instead of generating static responses, we can also generate a round-trip response (select "" as "", so the backend should come up with a response) ???
pub(crate) fn process_static(hash: u64, request: &FederatedRequestType) -> Option<FedResultStream> {
    let found = match request {
        FederatedRequestType::Query(query) => {
            match hash {
                // SELECT db_name(), schema_name(), original_login()
                9987961048535481693 => Some(db_info(query)),
                // SELECT @@TRANCOUNT
                347093103039328761 => Some(tran_count(query)),

                // todo: can be removed, is for testing only
                105025061482393546 => Some(get_tables()),
                _ => None,
            }
        }
        FederatedRequestType::Rpc(rpc) => match hash {
            // SqlString { value: Some("SELECT @@VERSION") }
            16324815309295181742 => Some(get_version(rpc)),
            // SqlString { value: Some("SELECT * FROM sys.types WHERE is_user_defined = 0 order by name") }
            3192063118983394666 => Some(get_types()),
            105025061482393546 => Some(get_tables()),
            _ => None,
        },
    };

    if let Some(result_set) = found {
        let stream = stream! {yield result_set;};
        return Some(FedResultStream::new(Box::pin(stream)));
    }

    None
}

#[rustfmt::skip]
fn get_tables() -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column("name", TypeInfo::new_sysname(), DataFlags::default())
        .add_column("database_id",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("source_database_id",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("owner_sid",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("create_date",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("compatibility_level",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("collation_name",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("user_access",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("user_access_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_read_only",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_auto_close_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_auto_shrink_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("state", TypeInfo::new_sysname(), DataFlags::default())
        .add_column("state_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_in_standby",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_cleanly_shutdown",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_supplemental_logging_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("snapshot_isolation_state",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("snapshot_isolation_state_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_read_committed_snapshot_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("recovery_model",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("recovery_model_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("page_verify_option",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("page_verify_option_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_auto_create_stats_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_auto_create_stats_incremental_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_auto_update_stats_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_auto_update_stats_async_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_ansi_null_default_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_ansi_nulls_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_ansi_padding_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_ansi_warnings_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_arithabort_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_concat_null_yields_null_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_numeric_roundabort_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_quoted_identifier_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_recursive_triggers_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_cursor_close_on_commit_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_local_cursor_default",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_fulltext_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_trustworthy_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_db_chaining_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_parameterization_forced",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_master_key_encrypted_by_server",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_query_store_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_published",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_subscribed",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_merge_published",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_distributor",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_sync_with_backup",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("service_broker_guid",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_broker_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("log_reuse_wait",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("log_reuse_wait_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_date_correlation_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_cdc_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_encrypted",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_honor_broker_priority_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("replica_id",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("group_database_id",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("resource_pool_id",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("default_language_lcid",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("default_language_name",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("default_fulltext_language_lcid",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("default_fulltext_language_name",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_nested_triggers_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_transform_noise_words_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("two_digit_year_cutoff",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("containment",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("containment_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("target_recovery_time_in_seconds",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("delayed_durability",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("delayed_durability_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_memory_optimized_elevate_to_snapshot_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_federation_member",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_remote_data_archive_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_mixed_page_allocation_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_temporal_history_retention_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("catalog_collation_type",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("catalog_collation_type_desc",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("physical_database_name",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_result_set_caching_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_accelerated_database_recovery_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_tempdb_spill_to_remote_store",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_stale_page_detection_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_memory_optimized_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_data_retention_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_ledger_on",TypeInfo::new_sysname(),DataFlags::default())
        .add_column("is_change_feed_enabled",TypeInfo::new_sysname(),DataFlags::default())
        .add_row(&[]);

    Ok(FedResult::Tabular(result_set.result))
}

#[rustfmt::skip]
fn get_types() -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column("name",TypeInfo::new_nvarchar(Some(128)),DataFlags::default())
        .add_column("system_type_id",TypeInfo::new_tiny_intn(false),DataFlags::default())
        .add_column("user_type_id",TypeInfo::new_intn(false),DataFlags::default())
        .add_column("schema_id",TypeInfo::new_intn(false),DataFlags::default())
        .add_column(("principal_id"),TypeInfo::new_intn(true),DataFlags::default())
        .add_column("max_length",TypeInfo::new_small_intn(false),DataFlags::default())
        .add_column("precision",TypeInfo::new_tiny_intn(false),DataFlags::default())
        .add_column("scale",TypeInfo::new_tiny_intn(false),DataFlags::default())
        .add_column("collation_name",TypeInfo::new_nvarchar(Some(128)),DataFlags::default())
        .add_column("is_nullable",TypeInfo::new_bit(),DataFlags::default())
        .add_column("is_user_defined",TypeInfo::new_bit(),DataFlags::default())
        .add_column("is_assembly_type",TypeInfo::new_bit(),DataFlags::default())
        .add_column("default_object_id",TypeInfo::new_intn(false),DataFlags::default())
        .add_column("rule_object_id",TypeInfo::new_intn(true),DataFlags::default())
        .add_column("is_table_type",TypeInfo::new_bit(),DataFlags::default())
        
        .add_row(&[ColumnData::new_nvarchar(Some("bigint"), Some(128)),ColumnData::U8(127),ColumnData::I32(127),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8),ColumnData::U8(19),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("binary"), Some(128)),ColumnData::U8(173),ColumnData::I32(173),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8000),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("bit"), Some(128)),ColumnData::U8(104),ColumnData::I32(104),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(1),ColumnData::U8(1),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("char"), Some(128)),ColumnData::U8(175),ColumnData::I32(175),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8000),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("date"), Some(128)),ColumnData::U8(40),ColumnData::I32(40),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(3),ColumnData::U8(10),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("datetime"), Some(128)),ColumnData::U8(61),ColumnData::I32(61),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8),ColumnData::U8(23),ColumnData::U8(3),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("datetime2"), Some(128)),ColumnData::U8(42),ColumnData::I32(42),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8),ColumnData::U8(27),ColumnData::U8(7),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("datetimeoffset"), Some(128)),ColumnData::U8(43),ColumnData::I32(43),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(10),ColumnData::U8(34),ColumnData::U8(7),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("decimal"), Some(128)),ColumnData::U8(106),ColumnData::I32(106),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(17),ColumnData::U8(38),ColumnData::U8(38),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("float"), Some(128)),ColumnData::U8(62),ColumnData::I32(62),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8),ColumnData::U8(53),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("geography"), Some(128)),ColumnData::U8(240),ColumnData::I32(130),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(-1),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(true),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("geometry"), Some(128)),ColumnData::U8(240),ColumnData::I32(129),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(-1),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(true),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("hierachyid"), Some(128)),ColumnData::U8(240),ColumnData::I32(128),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(892),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(true),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("image"), Some(128)),ColumnData::U8(34),ColumnData::I32(34),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(16),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("int"), Some(128)),ColumnData::U8(56),ColumnData::I32(56),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(4),ColumnData::U8(10),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("money"), Some(128)),ColumnData::U8(60),ColumnData::I32(60),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8),ColumnData::U8(19),ColumnData::U8(4),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("nchar"), Some(128)),ColumnData::U8(239),ColumnData::I32(239),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8000),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("ntext"), Some(128)),ColumnData::U8(99),ColumnData::I32(99),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(16),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("numeric"), Some(128)),ColumnData::U8(108),ColumnData::I32(108),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(17),ColumnData::U8(38),ColumnData::U8(38),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("nvarchar"), Some(128)),ColumnData::U8(231),ColumnData::I32(231),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8000),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("real"), Some(128)),ColumnData::U8(59),ColumnData::I32(59),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(4),ColumnData::U8(24),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("smalldatetime"), Some(128)),ColumnData::U8(58),ColumnData::I32(58),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(4),ColumnData::U8(16),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("smallint"), Some(128)),ColumnData::U8(52),ColumnData::I32(52),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(2),ColumnData::U8(5),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("smallmoney"), Some(128)),ColumnData::U8(122),ColumnData::I32(122),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(4),ColumnData::U8(10),ColumnData::U8(4),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("sql_variant"), Some(128)),ColumnData::U8(98),ColumnData::I32(98),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8016),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("sysname"), Some(128)),ColumnData::U8(231),ColumnData::I32(256),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(256),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("text"), Some(128)),ColumnData::U8(35),ColumnData::I32(35),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(16),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("time"), Some(128)),ColumnData::U8(41),ColumnData::I32(41),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(5),ColumnData::U8(16),ColumnData::U8(7),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("timestamp"), Some(128)),ColumnData::U8(189),ColumnData::I32(189),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("tinyint"), Some(128)),ColumnData::U8(48),ColumnData::I32(48),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(1),ColumnData::U8(3),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("uniqueidentifier"), Some(128)),ColumnData::U8(36),ColumnData::I32(36),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(16),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("varbinary"), Some(128)),ColumnData::U8(165),ColumnData::I32(165),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8000),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("varchar"), Some(128)),ColumnData::U8(167),ColumnData::I32(167),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(8000),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(Some("SQL_Latin1_General_CP1_CI_AS"), Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),])
        .add_row(&[ColumnData::new_nvarchar(Some("xml"), Some(128)),ColumnData::U8(241),ColumnData::I32(241),ColumnData::I32(4),ColumnData::I32N(None),ColumnData::I16(-1),ColumnData::U8(0),ColumnData::U8(0),ColumnData::new_nvarchar(EMPTY_STRING, Some(125)),ColumnData::Bit(true),ColumnData::Bit(false),ColumnData::Bit(false),ColumnData::I32(0),ColumnData::I32N(Some(0)),ColumnData::Bit(false),]);

    Ok(FedResult::Tabular(result_set.result))
}

fn get_version(_: &RpcRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column("", TypeInfo::new_nvarchar(Some(4000)), DataFlags::default())
        .add_row(&[ColumnData::String(SqlString::from_string(Some("Unilake SQL Proxy 2025 - version info needed here - Copyright (C) Menno Hamburg.  All rights reserved.".to_string()), Some(4000)))]);

    Ok(FedResult::Tabular(result_set.result))
}

fn tran_count(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column("", TypeInfo::new_intn(false), DataFlags::default())
        .add_row(&[ColumnData::I32(0)]);

    Ok(FedResult::Tabular(result_set.result))
}

fn db_info(_: &BatchRequest) -> Result<FedResult> {
    let result_set = ResultSetBuilder::new()
        .add_column("", TypeInfo::new_nvarchar(Some(128)), DataFlags::default())
        .add_column("", TypeInfo::new_nvarchar(Some(128)), DataFlags::default())
        .add_column("", TypeInfo::new_nvarchar(Some(4000)), DataFlags::default())
        .add_row(&[
            ColumnData::String(SqlString::from_string(Some("default_catalog"), Some(128))),
            ColumnData::String(SqlString::from_string(Some("dwh"), Some(128))),
            ColumnData::String(SqlString::from_string(Some("sa"), Some(4000))),
        ]);

    Ok(FedResult::Tabular(result_set.result))
}
