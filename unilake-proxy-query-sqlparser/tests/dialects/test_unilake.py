import unittest
from typing import Type

from sqlglot import parse_one

from sqlparser.dialects import Unilake


class TestDialectUnilake(unittest.TestCase):
    def test_parse_transpile_stmt(self):
        query = "TRANSPILE SELECT COUNT(1) FROM my_table"
        result = parse_one(query, dialect="unilake")
        pass

def _get_dialect(dialect: str) -> str | Type[Unilake]:
    if dialect == "unilake":
        return Unilake
    return dialect
