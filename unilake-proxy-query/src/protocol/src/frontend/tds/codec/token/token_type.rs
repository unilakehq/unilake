uint_enum! {
    /// The token type [2.2.4]
    /// Types of tokens in a token stream. Read from the first byte of the stream.
    #[repr(u8)]
    pub enum TdsTokenType {
        /// Used to send the status value of an RPC to the client. The server
        /// also uses this token to send the result status value of a stored
        /// procedure executed through SQL Batch.
        ///
        /// This token MUST be returned to the client when an RPC is executed by
        /// the server.
        ReturnStatus = 0x79,

        /// Describes the result set for interpretation of following ROW data
        /// streams
        ColMetaData = 0x81,

        /// Used to send an error message to the client.
        Error = 0xAA,

        /// Used to send an information message to the client.
        Info = 0xAB,

        /// Used to inform the client by which columns the data is ordered.
        Order = 0xA9,

        /// Describes the column information in browse mode.
        ColInfo = 0xA5,

        /// Used to send the return value of an RPCto the client. When an RPC is
        /// executed, the associated parameters may be defined as input or
        /// output (or "return") parameters.
        ///
        /// This token is used to send a description of the return parameter to
        /// the client. This token is also used to describe the value returned
        /// by a user-defined function (UDF) when executed as an RPC.
        ReturnValue = 0xAC,

        /// Used to send a response to a login request to the client.
        LoginAck = 0xAD,

        /// Used to send a complete row, as defined by the COLNAME and COLFMT
        /// tokens, to the client.
        Row = 0xD1,

        /// Used to send a row with null bitmap compression, as defined by the
        /// COLMETADATA token.
        NbcRow = 0xD2,

        /// The SSPI token returned during the login process.
        Sspi = 0xED,

        /// A notification of an environment change (such as database and
        /// language).
        EnvChange = 0xE3,

        /// Indicates the completion status of a SQL statement.
        ///
        /// This token is used to indicate the completion of a SQL statement.
        /// Because multiple SQL statements may be sent to the server in a
        /// single SQL batch, multiple DONE tokens may be generated. In this
        /// case, all but the final DONE token will have a Status value with the
        /// DONE_MORE bit set.
        ///
        /// A DONE token is returned for each SQL statement in the SQL batch,
        /// except for variable declarations.
        ///
        /// For execution of SQL statements within stored procedures, DONEPROC
        /// and DONEINPROC tokens are used in place of DONE tokens.
        Done = 0xFD,

        /// Indicates the completion status of a stored procedure. This is also
        /// generated for stored procedures executed through SQL statements.
        DoneProc = 0xFE,

        /// Indicates the completion status of a SQL statement within a stored procedure.
        DoneInProc = 0xFF,

        /// used to send an optional acknowledge message to the client for features that
        /// are defined in FeatureExt. The token stream is sent only along with the LOGINACK
        /// in a Login Response message.
        FeatureExtAck = 0xAE,

        /// Introduced in TDS 7.4, federated authentication information is returned to the client
        /// to be used for generating a Federated Authentication Token during the login process.
        /// This token MUST be the only token in a Federated Authentication Information
        /// message and MUST NOT be included in any other message type
        FedAuthInfo = 0xEE,

        /// Used to send session state data to the client.
        /// The data format defined here can also be used to send session state data
        /// for session recovery during login and login response.
        SessionState = 0xE4,

        // The following types have been left out
        /*
            AlternativeMetadata = 0x88,
            AlternativeRow = 0xD3,
            Offset = 0x78,
            TableName = 0xA4,
        */
    }
}

// TODO: check if the following tokens should be implemented:
// ColumnInfo = 0xA5,
// DoneProcedure = 0xFE,
// DoneInProc = 0xFF,
// FedAuthInfo = 0xEE,
// ReturnStatus = 0x79,
