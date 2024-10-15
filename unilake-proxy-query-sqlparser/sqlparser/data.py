from dataclasses import dataclass
from enum import Enum


@dataclass
class ErrorMessage:
    msg: str
    line: int
    column: int

    def to_json(self):
        return {"msg": self.msg, "line": self.line, "column": self.column}


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
    entity: str
    name: str
    alias: str

    def to_json(self) -> dict:
        return {"entity": self.entity, "name": self.name, "alias": self.alias}


@dataclass
class ScanOutputType(str, Enum):
    SELECT = "SELECT"
    INSERT = "INSERT"
    UPDATE = "UPDATE"
    DELETE = "DELETE"
    CREATE = "CREATE"
    DESCRIBE = "DESCRIBE"
    UNKNOWN = "UNKNOWN"

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


@dataclass
class ScanOutput:
    objects: list[ScanOutputObject]
    dialect: str
    query: dict | None
    type: ScanOutputType
    error: ErrorMessage | None

    def to_json(self) -> dict:
        return {
            "objects": [obj.to_json() for obj in self.objects],
            "dialect": self.dialect,
            "query": self.query,
            "type": self.type.value,
            "error": self.error.to_json() if self.error else None,
        }


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
class TranspilerInputStarExpand:
    scope: int
    table_alias: str
    column_name: str
    column_alias: str


@dataclass
class TranspilerInput:
    rules: list[TranspilerInputRules]
    filters: list[TranspilerInputFilters]
    star_expand: list[TranspilerInputStarExpand]
    cause: dict | None
    query: dict
    request_url: str | None

    @staticmethod
    def from_json(json_data: dict) -> "TranspilerInput":
        rules = []
        for rule in json_data.get("rules", []):
            rules.append(TranspilerInputRules(**rule))

        filters = []
        for filter_ in json_data.get("filters", []):
            filters.append(TranspilerInputFilters(**filter_))

        star_expand = []
        for expand in json_data.get("star_expand", []):
            star_expand.append(TranspilerInputStarExpand(**expand))

        return TranspilerInput(
            rules,
            filters,
            star_expand,
            json_data.get("cause"),
            json_data.get("query"),
            json_data.get("request_url") if "request_url" else None,
        )


@dataclass
class TranspilerOutput:
    sql_transformed: str
    sql_transformed_secure: str
    error: ErrorMessage | None

    def to_json(self) -> dict:
        return {
            "sql_transformed": self.sql_transformed,
            "sql_transformed_secure": self.sql_transformed_secure,
            "error": self.error.to_json() if self.error else None,
        }
