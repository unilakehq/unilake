import unittest
from typing import Type
from sqlglot import parse_one
from sqlparser.dialects import Unilake

class TestDialectUnilake(unittest.TestCase):
    def test_parse_transpile_stmt(self):
        query = "TRANSPILE SELECT COUNT(1) FROM my_table"
        result = parse_one(query, dialect="unilake")
        self.assertEqual(result.this, "TRANSPILE")
        self.assertEqual(result.expression, "SELECT COUNT(1) FROM my_table")

    def test_parse_scan_tags(self):
        query = "SCAN TAGS SELECT * FROM my_table"
        result = parse_one(query, dialect="unilake")
        self.assertEqual(result.this, "SCAN TAGS")
        self.assertEqual(result.args["expression"], "SELECT * FROM my_table")

    def test_parse_create_tag_with_desc_stmt(self):
        query = "CREATE TAG my_tag (WITH DESCRIPTION 'example tag')"
        result = parse_one(query, dialect="unilake")
        self.assertEqual(result.this, "CREATE TAG")
        self.assertEqual(result.args["name"], "my_tag")
        self.assertEqual(result.args["description"], "example tag")
        pass

def _get_dialect(dialect: str) -> str | Type[Unilake]:
    if dialect == "unilake":
        return Unilake
    return dialect
