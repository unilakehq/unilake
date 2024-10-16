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

    @unittest.skip("Tests not implemented yet")
    def test_transpile_single_filter(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_transpile_multiple_rules(self):
        pass
