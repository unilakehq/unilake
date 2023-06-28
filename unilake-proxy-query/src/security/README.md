The security crate is there to evaluate what can be done and what can't be done
when querying data. next to all this, we also have the expression evaluator
in this crate (regex to classifications)

Functions that need to be included (examples):
- Data Masking
    - TRUNCATE
    - REPLACE_CHARS
    - EMAIL_PREFIX
    - HASH
    - REPLACE_STRING
    - PREFIX
    - POSTFIX
    - NULL
    - REPLACE
- Data Filtering
    - COLUMN
    - ROW
    - ROW_RELATIONSHIP_ALLOWED (when a lineheader is the only way to filter a line item, what is allowed variant)
    - ROW_RELATIONSHIP_DENIED (whena  lineheader is the only way to filter a row item, what is denied variant)
- ACCESS
    - READONLY
    - NO_AGGREGATE (in case aggregate queries are not allowed)
- Connection context
    - CONNECTION_FROM_IP (if needed, a source IP address when to activate this rule)
    - CONNECTION_FROM_IP_RANGE (same as IP, start and end range)
    - CONNECTION_FROM_COUNTRY (same as IP, but using geo tracing)
    - CONNECTION_FROM_CONTINENT (same as IP, but using geo tracing)
    - CONNECTION_FROM_TIMEZONE (same as IP, but using geo tracing)
    - CLIENT_TOOL_ID (which application is used for this connection)
    - CLIENT_TOOL_NAME
    - CLIENT_TOOL_TYPE
    - CLIENT_TOOL_DRIVER
- Rule context
    - EXPIRES_ON
    - EXPIRES_INACTIVE (when not used for x number of days)
    - ACTIVE_FROM (what is the start date this rule will be enabled)