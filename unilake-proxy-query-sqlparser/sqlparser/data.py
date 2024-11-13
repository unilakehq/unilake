import json
from dataclasses import dataclass
from enum import Enum

import sqlglot


@dataclass
class ErrorMessage:
    description: str
    line: int
    col: int
    start_context: str
    highlight: str
    end_context: str
    into_expression: str | None

    def to_json(self):
        return {
            "description": self.description,
            "line": self.line,
            "col": self.col,
            "start_context": self.start_context,
            "highlight": self.highlight,
            "end_context": self.end_context,
            "into_expression": self.into_expression
        }

@dataclass
class ParserError:
    error_type: str
    message: str
    errors: [ErrorMessage]

    def to_json(self):
        return {
            "error_type": self.error_type,
            "message": self.message,
            "errors": [error.to_json() for error in self.errors]
        }

    @staticmethod
    def from_sqlglot_parse_error(error: sqlglot.errors.ParseError) -> "ParserError":
        return ParserError(
            error_type="PARSE_ERROR",
            message="",
            errors=[ErrorMessage(**error_info) for error_info in error.errors]
        )

    @staticmethod
    def from_sqlglot_optimize_error(error: sqlglot.errors.OptimizeError) -> "ParserError":
        x = error.args
        return ParserError(
            error_type="PARSE_ERROR",
            message=str(error.args),
            errors=[]
        )


@dataclass
class ScanEntity:
    catalog: str
    db: str
    name: str
    alias: str

    def to_json(self):
        return {"catalog": self.catalog, "db": self.db, "entity": self.name, "alias": self.alias}


@dataclass
class ScanAttribute:
    entity_alias: str
    name: str
    alias: str

    def to_json(self) -> dict:
        return {"entity_alias": self.entity_alias, "name": self.name, "alias": self.alias}


@dataclass
class ScanOutputType(str, Enum):
    SELECT = "SELECT"
    INSERT = "INSERT"
    UPDATE = "UPDATE"
    DELETE = "DELETE"
    CREATE = "CREATE"
    DESCRIBE = "DESCRIBE"
    UNKNOWN = "UNKNOWN"
    TRUNCATE = "TRUNCATE"
    ALTER = "ALTER"
    DROP = "DROP"
    REFRESH = "REFRESH"
    COMMAND = "COMMAND"
    EXPORT = "EXPORT"
    SET = "SET"

    @classmethod
    def from_key(cls, key: str) -> "ScanOutputType":
        return cls(key.upper()) if key.upper() in cls._value2member_map_ else cls.UNKNOWN


@dataclass
class ScanOutputObject:
    scope: int
    entities: list[ScanEntity]
    attributes: list[ScanAttribute]
    is_agg: bool

    def to_json(self):
        return {
            "scope": self.scope,
            "entities": [entity.to_json() for entity in self.entities],
            "attributes": [attr.to_json() for attr in self.attributes],
            "is_agg": self.is_agg,
        }


# todo: scan output can also include internal commands, we need to pass them from the scan operation (expand ScanOutput)
@dataclass
class ScanOutput:
    objects: list[ScanOutputObject]
    dialect: str
    query: str | None
    type: ScanOutputType
    error: ParserError | None
    target_entity: str | None

    def to_json(self) -> dict:
        return {
            "objects": [obj.to_json() for obj in self.objects],
            "dialects": self.dialect,
            "query": self.query,
            "type": self.type.value,
            "error": self.error.to_json() if self.error else None,
            "target_entity": self.target_entity,
        }

    @staticmethod
    def from_parser_error(parser_error: ParserError) -> "ScanOutput":
        return ScanOutput(
            objects=[],
            dialect="",
            query=None,
            type=ScanOutputType.UNKNOWN,
            error=parser_error,
            target_entity=None,
        )

@dataclass
class TranspilerInputRules:
    scope: int
    attribute: str
    rule_id: str
    rule_definition: dict


@dataclass
class TranspilerInputFilters:
    scope: int
    attribute: str
    filter_id: str
    filter_definition: dict

@dataclass
class TranspilerInput:
    rules: list[TranspilerInputRules]
    filters: list[TranspilerInputFilters]
    visible_schema: dict
    cause: dict | None
    query: str
    request_url: str | None

    @staticmethod
    def from_json(json_data: str | dict) -> "TranspilerInput":
        if isinstance(json_data, str):
            json_data = json.loads(json_data)

        rules = []
        for rule in json_data.get("rules", []):
            rules.append(TranspilerInputRules(**rule))

        filters = []
        for filter_ in json_data.get("filters", []):
            filters.append(TranspilerInputFilters(**filter_))

        return TranspilerInput(
            rules,
            filters,
            json_data.get("visible_schema"),
            json_data.get("cause"),
            json.loads(json_data.get("query")),
            json_data.get("request_url") if "request_url" else None,
        )


@dataclass
class TranspilerOutput:
    sql_transformed: str
    error: ParserError | None

    def to_json(self) -> dict:
        return {
            "sql_transformed": self.sql_transformed,
            "error": self.error.to_json() if self.error else None,
        }

    @staticmethod
    def from_parser_error(parser_error: ParserError) -> "TranspilerOutput":
        return TranspilerOutput(
            sql_transformed="",
            error=parser_error
        )
