import unittest
from typing import Type

import sqlglot.errors
from sqlglot import parse_one
from sqlparser.dialects import Unilake


class TestDialectUnilake(unittest.TestCase):
    def test_bench(self):
        query = "CREATE RESOURCE GROUP rg1"
        result = parse_one(query, dialect="unilake")
        self.assertEqual(result.this, "TRUNCATE")

    def test_parse_transpile_stmt(self):
        query = "TRANSPILE SELECT COUNT(1) FROM my_table"
        result = parse_one(query, dialect="unilake")
        self.assertEqual(result.this, "TRANSPILE")
        self.assertEqual(result.expression, "SELECT COUNT(1) FROM my_table")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_scan_tags(self):
        query = "SCAN TAGS SELECT * FROM my_table"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.this, "SCAN")
        self.assertEqual(result.args["kind"], "TAGS")
        self.assertEqual(result.args["expression"], "SELECT * FROM my_table")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 4)

    def test_parse_execute_audit(self):
        query = "EXECUTE AUDIT('id', 'environment') AS SELECT * FROM my_table WHERE record = 'faulty'"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.this, "EXECUTE")
        self.assertEqual(result.args["kind"], "AUDIT")
        self.assertEqual(result.args["expression"], "SELECT * FROM my_table WHERE record = 'faulty'")
        self.assertEqual(result.args["id"], "id")
        self.assertEqual(result.args["environment"], "environment")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 6)

    def test_parse_set_something(self):
        query = "SET x = y"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "set")
        self.assertEqual(result.args["variable"], "x")
        self.assertEqual(result.args["value"], "y")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_describe_access(self):
        query = "DESCRIBE ACCESS SELECT * FROM my_table"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "describe")
        self.assertEqual(result.this, "ACCESS")
        self.assertEqual(result.args['expression'], "SELECT * FROM my_table")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_set_impersonate_as(self):
        query = "IMPERSONATE AS my_user"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "command")
        self.assertEqual(result.this, "IMPERSONATE")
        self.assertEqual(result.args['user'], "my_user")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_impersonate_cancel(self):
        query = "IMPERSONATE CANCEL"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "command")
        self.assertEqual(result.this, "IMPERSONATE")
        self.assertEqual(result.args['user'], "")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_set_impersonate_as_incorrect(self):
        query = "IMPERSONATE my_user"
        self.assertRaises(sqlglot.errors.ParseError, parse_one, query, dialect="unilake")

    def test_parse_reset_something(self):
        query = "RESET x"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "command")
        self.assertEqual(result.this, "RESET")
        self.assertEqual(result.args['variable'], "x")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_use_catalog(self):
        query = "USE CATALOG my_catalog"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "use")
        self.assertEqual(result.args["kind"], "CATALOG")
        self.assertEqual(result.args["target"], "my_catalog")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_use_schema(self):
        query = "USE SCHEMA my_schema"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "use")
        self.assertEqual(result.args["kind"], "SCHEMA")
        self.assertEqual(result.args["target"], "my_schema")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

    def test_parse_use_database(self):
        query = "USE DATABASE my_database"
        result = parse_one(query, dialect="unilake")

        self.assertEqual(result.key, "use")
        self.assertEqual(result.args["kind"], "DATABASE")
        self.assertEqual(result.args["target"], "my_database")
        self.assertEqual(result.args["internal"], "true")
        self.assertEqual(len(result.args), 3)

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
