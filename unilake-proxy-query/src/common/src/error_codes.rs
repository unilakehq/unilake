#![allow(non_snake_case)]

use crate::error_code::ErrorCode;

macro_rules! build_exceptions {
    ($($(#[$meta:meta])* $body:ident($code:expr)),*$(,)*) => {
        impl ErrorCode {
            $(

                paste::item! {
                    $(
                        #[$meta]
                    )*
                    pub const [< $body:snake:upper >]: u16 = $code;
                }
                $(
                    #[$meta]
                )*
                pub fn $body(display_text: impl Into<String>) -> ErrorCode {
                    ErrorCode::create(
                        $code,
                        stringify!($body),
                        display_text.into(),
                        String::new(),
                    )
                }
            )*
        }
    }
}

// Internal errors [0, 2000].
build_exceptions! {
    Ok(0),
    UnknownDatabase(1003),
    FailedScanOperation(1001),
    FailedTranspileOperation(1001),
    FailedSecureOperation(1001),
    PolicyNotFound(1001),
    QueryError(1002),
    TooManySessions(1003),
    ServerShutdown(1004)
}

// Security errors [0, 2000].
build_exceptions! {
    /// In case we cannot find an entity from the attribute
    EntityNotFoundFromAttribute(0),
    /// Happens when a requested entity <catalog>.<schema>.<entity> does not exist
    EntityNotFound(0),
    /// Happens when a requested entity <catalog>.<schema>.<entity> exists but is not allowed to be accessed
    EntityNotAllowed(0),
    /// Happens when the user groups cannot be found
    UserGroupsNotFound(0),
    /// Happens when the user cannot be found
    UserNotFound(0),
    /// Happens when there are issues with the policy being used
    PolicyError(0),
    /// Happens when the cache is in an invalid state, retry the process. Returns the current iteration count
    InvalidCacheError(0),
    /// Happens when the iteration limit is reached for processing the security checks
    IterationLimitReached(0),

    // todo: model errors
    SessionIpInfoModel(0),
    SessionAppInfoModel(0)
}

// TDS protocol errors [0, 2000].
build_exceptions! {
    TdsProtocolFailedToDecodeVarchar(0),
    TdsTokenDoneInvalidStatus(0),
    TdsInvalidEnvChangeType(0),
    TdsUtfConversionFailed(0),
    TdsInvalidPacket(0),
    TdsProtocolError(0),
    TdsInvalidTimeScale(0),
    TdsProtocol(10),
    TdsEncodingUnsupported(0),
    TdsIncorrectState(0),
    TdsInvalidRpcProcedureType(0),
    TdsUnknownTokenType(0),
    TdsStringLengthTooLong(0),
}

// StarRocks Backend errors [0, 2000].
build_exceptions! {
    StarRocksConnectionPoolError(0)
}
