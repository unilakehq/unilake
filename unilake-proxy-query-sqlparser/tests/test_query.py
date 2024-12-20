import unittest
from sqlparser import scan, transpile, TranspilerOutput, ScanOutput


class TestQuery(unittest.TestCase):
    def run_test_with_filter_rule(
        self, filters: dict, input_query: str, expected_output: str
    ) -> (ScanOutput, TranspilerOutput):
        return self.run_test(
            input_query,
            expected_output,
            [],
            [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "attribute_id": "some_guid",
                    "policy_id": "some_guid",
                    "filter_definition": filters,
                }
            ]
            if filters
            else [],
        )

    def run_test_with_custom_filter_rule(
        self, filters: [], input_query: str, expected_output: str
    ) -> (ScanOutput, TranspilerOutput):
        return self.run_test(input_query, expected_output, [], filters)

    def run_test_with_masking_rule(
        self, rule_definition: dict, input_query: str, expected_output: str
    ) -> (ScanOutput, TranspilerOutput):
        return self.run_test(
            input_query,
            expected_output,
            [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "policy_id": "some_guid",
                    "rule_definition": rule_definition,
                }
            ]
            if rule_definition
            else [],
            [],
        )

    def run_test_with_custom_masking_rule(
        self, rule_definitions: [], input_query: str, expected_output: str
    ) -> (ScanOutput, TranspilerOutput):
        return self.run_test(input_query, expected_output, rule_definitions, [])

    def run_test(
        self,
        input_query: str,
        expected_output: str,
        rule_definition: list,
        filters: list,
    ) -> (ScanOutput, TranspilerOutput):
        query = scan(input_query, "unilake", "catalog", "database")
        json_input = {
            "rules": rule_definition,
            "filters": filters,
            "star_expand": [],
            "cause": None,
            "query": query.query,
            "request_url": None,
        }
        output = transpile(json_input)
        self.assertIsNone(output.error)
        self.assertEqual(
            expected_output.strip(),
            output.sql_transformed,
        )
        return query, output
