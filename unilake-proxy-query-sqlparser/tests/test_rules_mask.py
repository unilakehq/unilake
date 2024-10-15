import unittest
from sqlparser import transpile, scan


class TestRulesMask(unittest.TestCase):
    def run_test(self, rule_definition: dict, input_query: str, expected_output: str):
        query = scan(input_query, "unilake", "catalog", "database")
        input = {
            "rules": [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "rule_id": "some_guid",
                    "rule_definition": rule_definition,
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
            expected_output.strip(),
            output.sql_transformed,
        )

    def test_rules_mask_xxhash3(self):
        self.run_test(
            {"name": "xxhash3", "properties": None},
            "SELECT a from b",
            "SELECT XX_HASH3_128(`b`.`a`) AS `a` FROM `catalog`.`database`.`b` AS `b`",
        )

    def test_rules_mask_replace_null(self):
        self.run_test(
            {"name": "replace_null", "properties": None},
            "SELECT a from b",
            "SELECT NULL AS `a` FROM `catalog`.`database`.`b` AS `b`",
        )

    def test_rules_mask_replace_char(self):
        self.run_test(
            {"name": "replace_char", "properties": {"replacement": "X"}},
            "SELECT a from b",
            "SELECT REPEAT('X', LENGTH(`b`.`a`)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_replace_string(self):
        self.run_test(
            {
                "name": "replace_string",
                "properties": {"replacement": "[REDACTED]"},
            },
            "SELECT a from b",
            "SELECT '[REDACTED]' AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_except_last(self):
        self.run_test(
            {
                "name": "mask_except_last",
                "properties": {"value": "X", "len": "3"},
            },
            "SELECT a from b",
            "SELECT CONCAT(REPEAT('X', LENGTH(`b`.`a`) - 3), RIGHT(`b`.`a`, 3)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_except_first(self):
        self.run_test(
            {
                "name": "mask_except_first",
                "properties": {"value": "X", "len": "3"},
            },
            "SELECT a from b",
            "SELECT CONCAT(LEFT(`b`.`a`, 3), REPEAT('X', LENGTH(`b`.`a`) - 3)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_rounding(self):
        self.run_test(
            {"name": "rounding", "properties": {"value": "2"}},
            "SELECT a from b",
            "SELECT ROUND(`b`.`a`, 2) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_random_number(self):
        self.run_test(
            {"name": "random_number", "properties": {"min": "2", "max": "5"}},
            "SELECT a from b",
            "SELECT FLOOR((5 - 2 + 1) * RAND() + 2) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_random_multiplication(self):
        self.run_test(
            {"name": "random_multiplication", "properties": {"max": "5"}},
            "SELECT a from b",
            "SELECT RAND() * 5 AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_left(self):
        self.run_test(
            {"name": "left", "properties": {"len": "3"}},
            "SELECT a from b",
            "SELECT LEFT(`b`.`a`, 3) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_right(self):
        self.run_test(
            {"name": "right", "properties": {"len": "3"}},
            "SELECT a from b",
            "SELECT RIGHT(`b`.`a`, 3) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_mail_hash_pres(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_mail_mask_pres(self):
        pass

    def test_rules_mask_mail_username(self):
        self.run_test(
            {"name": "mail_mask_username"},
            "SELECT a from b",
            "SELECT CONCAT_WS('@', REPEAT('x', LOCATE('@', `b`.`a`) - 1), SPLIT_PART(`b`.`a`, '@', 2)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_mail_domain(self):
        self.run_test(
            {"name": "mail_mask_domain"},
            "SELECT a from b",
            "SELECT CONCAT_WS('@',SPLIT_PART(`b`.`a`, '@', 1),CONCAT(REPEAT('x', CHAR_LENGTH(SPLIT_PART(`b`.`a`, '@', 2)) - CHAR_LENGTH(SPLIT_PART(SPLIT_PART(`b`.`a`, '@', 2), '.', -1)) - 1),'.',SPLIT_PART(SPLIT_PART(`b`.`a`, '@', 2), '.', -1))) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_cc_hash_pres(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_cc_mask_pres(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_cc_last_four(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_date_year_only(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_date_month_only(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_ip_anonymize(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_ip_hash_pres(self):
        pass

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_semi_structured(self):
        pass
