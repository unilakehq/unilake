import sqlglot

from build.lib.sqlparser.data import TranspilerInput
from sqlparser.data import ScanOutput, TranspilerOutput, ErrorMessage, ParserError
from sqlparser.transpiler import inner_scan, inner_transpile


def scan(sql: str, dialect: str, catalog: str, database: str) -> ScanOutput:
    try:
        return transpiler.inner_scan(sql, dialect, catalog, database)
    except sqlglot.errors.ParseError as e:
        parser_error = ParserError.from_sqlglot_parse_error(e)
        return ScanOutput.from_parser_error(parser_error)
    except sqlglot.errors.OptimizeError as e:
        parser_error = ParserError.from_sqlglot_optimize_error(e)
        return ScanOutput.from_parser_error(parser_error)
    except Exception as e:
        parser_error = ParserError(type="INTERNAL_ERROR", message=str(e), errors=[])
        return ScanOutput.from_parser_error(parser_error)

def transpile(source: str | dict | TranspilerInput, secure_output: bool = False) -> TranspilerOutput:
    try:
        return transpiler.inner_transpile(source, secure_output)
    except sqlglot.errors.ParseError as e:
        parser_error = ParserError.from_sqlglot_parse_error(e)
        return TranspilerOutput.from_parser_error(parser_error)
    except Exception as e:
        parser_error = ParserError(type="INTERNAL_ERROR", message=str(e), errors=[])
        return TranspilerOutput.from_parser_error(parser_error)

def secure_query(sql: str, dialect: str, catalog: str, database: str) -> str:
    pass