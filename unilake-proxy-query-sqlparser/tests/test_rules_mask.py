import unittest
from tests.test_query import TestQuery


class TestRulesMask(TestQuery):
    def test_rules_mask_xxhash3(self):
        self.run_test_with_masking_rule(
            {"name": "xxhash3", "properties": None},
            "SELECT a from b",
            "SELECT XX_HASH3_128(`b`.`a`) AS `a` FROM `catalog`.`database`.`b` AS `b`",
        )

    def test_rules_mask_replace_null(self):
        self.run_test_with_masking_rule(
            {"name": "replace_null", "properties": None},
            "SELECT a from b",
            "SELECT NULL AS `a` FROM `catalog`.`database`.`b` AS `b`",
        )

    def test_rules_mask_replace_char(self):
        self.run_test_with_masking_rule(
            {"name": "replace_char", "properties": {"replacement": "X"}},
            "SELECT a from b",
            "SELECT REPEAT('X', LENGTH(`b`.`a`)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_replace_string(self):
        self.run_test_with_masking_rule(
            {
                "name": "replace_string",
                "properties": {"replacement": "[REDACTED]"},
            },
            "SELECT a from b",
            "SELECT '[REDACTED]' AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_except_last(self):
        self.run_test_with_masking_rule(
            {
                "name": "mask_except_last",
                "properties": {"value": "X", "len": "3"},
            },
            "SELECT a from b",
            "SELECT CONCAT(REPEAT('X', LENGTH(`b`.`a`) - 3), RIGHT(`b`.`a`, 3)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_except_first(self):
        self.run_test_with_masking_rule(
            {
                "name": "mask_except_first",
                "properties": {"value": "X", "len": "3"},
            },
            "SELECT a from b",
            "SELECT CONCAT(LEFT(`b`.`a`, 3), REPEAT('X', LENGTH(`b`.`a`) - 3)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_rounding(self):
        self.run_test_with_masking_rule(
            {"name": "rounding", "properties": {"value": "2"}},
            "SELECT a from b",
            "SELECT ROUND(`b`.`a`, 2) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_random_number(self):
        self.run_test_with_masking_rule(
            {"name": "random_number", "properties": {"min": "2", "max": "5"}},
            "SELECT a from b",
            "SELECT FLOOR((5 - 2 + 1) * RAND() + 2) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_random_multiplication(self):
        self.run_test_with_masking_rule(
            {"name": "random_multiplication", "properties": {"max": "5"}},
            "SELECT a from b",
            "SELECT RAND() * 5 AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_left(self):
        self.run_test_with_masking_rule(
            {"name": "left", "properties": {"len": "3"}},
            "SELECT a from b",
            "SELECT LEFT(`b`.`a`, 3) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_right(self):
        self.run_test_with_masking_rule(
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
        self.run_test_with_masking_rule(
            {"name": "mail_mask_username"},
            "SELECT a from b",
            "SELECT CONCAT_WS('@', REPEAT('x', LOCATE('@', `b`.`a`) - 1), SPLIT_PART(`b`.`a`, '@', 2)) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_mail_domain(self):
        self.run_test_with_masking_rule(
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

    def test_rules_mask_date_year_only(self):
        self.run_test_with_masking_rule(
            {"name": "date_year_only"},
            "SELECT a from b",
            "SELECT DATE_TRUNC('YEAR', `b`.`a`) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_date_month_only(self):
        self.run_test_with_masking_rule(
            {"name": "date_month_only"},
            "SELECT a from b",
            "SELECT DATE_TRUNC('MONTH', `b`.`a`) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    def test_rules_mask_ip_anonymize(self):
        self.run_test_with_masking_rule(
            {"name": "ip_anonymize"},
            "SELECT a from b",
            "SELECT CONCAT_WS('.', SPLIT_PART(`b`.`a`, '.', 1), SPLIT_PART(`b`.`a`, '.', 2), '0', '0') AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_ip_hash_pres(self):
        pass

    def test_rules_mask_ip_mask_pres(self):
        self.run_test_with_masking_rule(
            {"name": "ip_mask_pres"},
            "SELECT a from b",
            "SELECT CONCAT_WS('.', REPEAT('*', CHAR_LENGTH(SPLIT_PART(`b`.`a`, '.', 1))), REPEAT('*', CHAR_LENGTH(SPLIT_PART(`b`.`a`, '.', 2))), REPEAT('*', CHAR_LENGTH(SPLIT_PART(`b`.`a`, '.', 3))), REPEAT('*', CHAR_LENGTH(SPLIT_PART(`b`.`a`, '.', 4)))) AS `a` FROM `catalog`.`database`.`b` AS `b` ",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_mask_semi_structured(self):
        pass

    def test_rules_mask_replace_null_nested(self):
        self.run_test_with_custom_masking_rule(
            [
                {
                    "scope": 1,
                    "attribute": '"b"."a"',
                    "rule_id": "some_guid",
                    "rule_definition": {"name": "replace_null"},
                }
            ],
            "SELECT * from (select a from b)",
            "SELECT `_q_0`.`a` AS `a` FROM (SELECT NULL AS `a` FROM `catalog`.`database`.`b` AS `b`) AS `_q_0`",
        )

    # in command usage:
    # [] - CTAS (CREATE TABLE AS SELECT)
    # [] - CTAS (CREATE TABLE AS SELECT star)
    # [] - DELETE FROM (SELECT)
    # [] - DELETE FROM (SELECT star)
    # [] - DELETE FROM (multi-table join)
    # [] - DELETE FROM (CTE)
    # [] - DELETE FROM (subquery)
    # [] - INSERT INTO (select)
    # [] - INSERT INTO (select star)
    # [] - INSERT INTO (CTE)
    # [] - INSERT INTO (multi-table join)
    # [?] - INSERT INTO (subquery)
    # [] - INSERT INTO OVERWRITE (select)
    # [] - INSERT INTO OVERWRITE (select star)
    # [] - UPDATE (FROM select)
    # [] - UPDATE (FROM select star)
    # [] - UPDATE (FROM CTE)
    # [] - UPDATE (FROM multi-table join)
    # [] - UPDATE (FROM subquery)

    def test_rules_mask_insert_into_subquery_result(self):
        self.run_test_with_custom_masking_rule(
            [
                {
                    "scope": 0,
                    "attribute": '"test2"."a"',
                    "rule_id": "some_guid",
                    "rule_definition": {"name": "replace_null"},
                }
            ],
            "INSERT INTO test (a, b) SELECT a, b from test2",
            "INSERT INTO `catalog`.`database`.`test` (`a`, `b`) SELECT NULL AS `a`, `test2`.`b` AS `b` FROM `catalog`.`database`.`test2` AS `test2`",
        )
