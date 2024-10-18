import unittest
from sqlparser import transpile, scan


class TestTranspile(unittest.TestCase):
    def test_transpile_empty_input(self):
        input = {}
        output = transpile(input)
        self.assertIsNotNone(output.error)

    def test_transpile_no_input(self):
        input = {
            "rules": [],
            "filters": [],
            "star_expand": [],
            "cause": None,
            "query": {},
            "request_url": None,
        }
        output = transpile(input)
        self.assertIsNotNone(output.error)

    def test_transpile_single_rule(self):
        query = scan("SELECT a from b", "unilake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "rule_id": "some_guid",
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

    def test_transpile_star_expand_with_mask(self):
        query = scan("SELECT * from b", "unilake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "rule_id": "some_guid",
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

    def test_transpile_star_expand_with_filter(self):
        query = scan("SELECT * from b", "unilake", "catalog", "database")
        input = {
            "rules": [],
            "filters": [{
                "scope": 0,
                "attribute": '"b"."a"',
                "filter_id": "some_guid",
                "filter_definition": {
                    "expression": "? > 0"
                }
            }],
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
                    "attribute": '"b"."a"',
                    "rule_id": "some_guid",
                    "rule_definition": {"name": "xxhash3", "properties": None},
                }
            ],
            "filters": [{
                "scope": 0,
                "attribute": '"b"."a"',
                "filter_id": "some_guid",
                "filter_definition": {
                    "expression": "? > 0"
                }
            }],
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

    def test_transpile_large_query(self):
        with open("data/large_query.sql", "r") as file:
            sql = file.read()
        query = scan(sql, "snowflake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "rule_id": "some_guid",
                    "rule_definition": {"name": "xxhash3", "properties": None},
                }
            ],
            "filters": [{
                "scope": 0,
                "attribute": '"b"."a"',
                "filter_id": "some_guid",
                "filter_definition": {
                    "expression": "? > 0"
                }
            }],
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
