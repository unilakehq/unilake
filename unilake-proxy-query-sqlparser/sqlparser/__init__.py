from sqlparser.data import ScanOutput, TranspilerOutput
from sqlparser.transpiler import inner_scan, inner_transpile


def scan(sql: str, dialect: str, catalog: str, database: str) -> ScanOutput:
    return transpiler.inner_scan(sql, dialect, catalog, database)

def transpile(source: dict) -> TranspilerOutput:
    return transpiler.inner_transpile(source)
