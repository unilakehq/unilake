import json
import unittest

from sqlparser import transpile, scan

# todo: make sure transpiling of windows functions are supported! so order by x is also hashed(x)
# todo: also make sure that the order of unpacking a star is consistent with the input schema
# todo: add tests for window functions, expressions:
#   - order by
#   - group by
#   - partition by
#   - window clause, over
#   - select 1
#   - select some, 1
#   - select count(*)

class TestTranspile(unittest.TestCase):
    def test_transpile_empty_input(self):
        input = {}
        output = transpile(input)
        self.assertIsNotNone(output.error)

    def test_transpile_no_input(self):
        input = {
            "rules": [],
            "filters": [],
            "visible_schema": [],
            "cause": None,
            "query": {},
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNotNone(output.error)

    def test_transpile_correct_star_expand(self):
        query = scan(
            "select top 5 * from default_catalog.dwh.[DimAccount]", "tsql", "default_catalog", "dwh"
        )
        input = {
            "rules": [],
            "filters": [],
            "visible_schema": {
                "default_catalog": {
                    "dwh": {
                        "DimAccount": {"AccountKey": "INT", "ParentAccountKey": "INT"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT `DimAccount`.`AccountKey` AS `AccountKey`, `DimAccount`.`ParentAccountKey` AS `ParentAccountKey` FROM `default_catalog`.`dwh`.`DimAccount` AS `DimAccount` LIMIT 5",
            output.sql_transformed,
        )

    def test_transpile_single_rule(self):
        query = scan("SELECT a from b", "unilake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "rule_definition": {"name": "xxhash3", "properties": None},
                }
            ],
            "filters": [],
            "star_expand": [],
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT XX_HASH3_128(`b`.`a`) AS `a` FROM `catalog`.`database`.`b` AS `b`",
            output.sql_transformed,
        )

    def test_transpile_add_filter_non_selected_column(self):
        query = scan("SELECT c from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [{
                "scope": 0,
                "attribute_id": "some_guid",
                "attribute": '"b"."a"',
                "policy_id": "some_guid",
                "filter_definition": {"expression": "? > 0"},
            }],
            "star_expand": [],
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT `b`.`c` AS `c` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0",
            output.sql_transformed,
        )

    def test_transpile_multiple_filters_remove_duplicates(self):
        query = scan("SELECT a, a as x, a as xx, a as xxx, from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [{
                "scope": 0,
                "attribute_id": "some_guid",
                "attribute": '"b"."a"',
                "policy_id": "some_guid",
                "filter_definition": {"expression": "? > 0"},
                },
                {
                "scope": 0,
                "attribute_id": "some_guid",
                "attribute": '"b"."a"',
                "policy_id": "another_guid",
                "filter_definition": {"expression": "? < 10"},
            }],
            "star_expand": [],
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT `b`.`a` AS `a`, `b`.`a` AS `x`, `b`.`a` AS `xx`, `b`.`a` AS `xxx` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0 AND `b`.`a` < 10",
            output.sql_transformed,
        )

    def test_transpile_star_expand_with_mask(self):
        query = scan("SELECT * from b", "unilake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "rule_definition": {"name": "xxhash3", "properties": None},
                }
            ],
            "filters": [],
            "visible_schema": {
                "catalog": {
                    "database": {
                        "b": {"a": "INT", "b": "VARCHAR"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT XX_HASH3_128(`b`.`a`) AS `a`, `b`.`b` AS `b` FROM `catalog`.`database`.`b` AS `b`",
            output.sql_transformed,
        )
        print(query.to_json())

    def test_transpile_star_expand_with_filter(self):
        query = scan("SELECT * from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                }
            ],
            "visible_schema": {
                "catalog": {
                    "database": {
                        "b": {"a": "INT", "b": "VARCHAR"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT `b`.`a` AS `a`, `b`.`b` AS `b` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0",
            output.sql_transformed,
        )

    def test_transpile_star_expand_with_mask_and_filter(self):
        query = scan("SELECT * from b", "unilake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "rule_definition": {"name": "xxhash3", "properties": None},
                }
            ],
            "filters": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                }
            ],
            "visible_schema": {
                "catalog": {
                    "database": {
                        "b": {"a": "INT", "b": "VARCHAR"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT XX_HASH3_128(`b`.`a`) AS `a`, `b`.`b` AS `b` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0",
            output.sql_transformed,
        )

    def test_transpile_inaccurate_visible_schema_results_in_no_query(self):
        query = scan("SELECT * from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [],
            "visible_schema": {
                "catalog": {
                    "incorrect_database": {
                        "b": {"a": "INT", "b": "VARCHAR"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        print(output.sql_transformed)
        self.assertIsNotNone(output.error)

    def test_transpile_missing_columns_visible_schema_results_in_no_query(self):
        query = scan("SELECT c from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [],
            "visible_schema": {
                "catalog": {
                    "database": {
                        "b": {"a": "INT", "b": "VARCHAR"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        print(output.sql_transformed)
        self.assertIsNotNone(output.error)

    def test_transpile_missing_table_visible_schema_results_in_no_query(self):
        query = scan("SELECT c from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [],
            "visible_schema": {
                "catalog": {
                    "database": {
                        "missing": {"a": "INT", "b": "VARCHAR"},
                    }
                }
            },
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        print(output.sql_transformed)
        self.assertIsNotNone(output.error)

    def test_transpile_large_query(self):
        with open("data/large_query.sql", "r") as file:
            sql = file.read()
        query = scan(sql, "snowflake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "rule_definition": {"name": "xxhash3", "properties": None},
                }
            ],
            "filters": [
                {
                    "scope": 0,
                    "attribute_id": "some_guid",
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                }
            ],
            "visible_schema": None,
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNone(output.error)

    def test_transpile_secure_output(self):
        sql = "SELECT firstname FROM b where username = 'admin' and country in ('USA', 'Canada') and age > 30"
        query = scan(sql, "snowflake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [],
            "star_expand": [],
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(input, secure_output=True)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT `b`.`firstname` AS `firstname` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`username` = '?' AND `b`.`country` IN ('?', '?') AND `b`.`age` > ?",
            output.sql_transformed,
        )

    def test_transpile_string_json_input(self):
        sql = "SELECT firstname FROM b where username = 'admin' and country in ('USA', 'Canada') and age > 30"
        query = scan(sql, "snowflake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [],
            "visible_schema": None,
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        some_input_json = json.dumps(input)
        output = transpile(some_input_json, secure_output=True)
        self.assertIsNone(output.error)
        self.assertEqual(
            "SELECT `b`.`firstname` AS `firstname` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`username` = '?' AND `b`.`country` IN ('?', '?') AND `b`.`age` > ?",
            output.sql_transformed,
        )
