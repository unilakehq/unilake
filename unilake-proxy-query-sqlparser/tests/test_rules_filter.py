import unittest
from tests.test_query import TestQuery


class TestRulesFilter(TestQuery):
    def test_rules_filter_single(self):
        self.run_test_with_filter_rule(
            {"expression": "? > 0"},
            "SELECT a from b",
            "SELECT `b`.`a` AS `a` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0",
        )

    def test_rules_filter_single_next_to_existing_filter(self):
        self.run_test_with_filter_rule(
            {"expression": "? > 0"},
            "SELECT a from b where a < 10000",
            "SELECT `b`.`a` AS `a` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` < 10000 AND `b`.`a` > 0",
        )

    def test_rules_filter_single_next_to_existing_filter_2(self):
        self.run_test_with_filter_rule(
            {"expression": "? > 0"},
            "SELECT a from b where a < 10000 or a < 0",
            "SELECT `b`.`a` AS `a` FROM `catalog`.`database`.`b` AS `b` WHERE (`b`.`a` < 10000 OR `b`.`a` < 0) AND `b`.`a` > 0",
        )

    def test_rules_filter_in(self):
        self.run_test_with_filter_rule(
            {"expression": "? in (0,1,2,3)"},
            "SELECT a from b",
            "SELECT `b`.`a` AS `a` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` IN (0, 1, 2, 3)",
        )

    def test_rules_filter_multi(self):
        self.run_test_with_custom_filter_rule(
            [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "filter_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                },
                {
                    "scope": 0,
                    "attribute": '"b"."b"',
                    "filter_id": "some_guid",
                    "filter_definition": {"expression": "? < 1000"},
                },
            ],
            "SELECT a,b from b",
            "SELECT `b`.`a` AS `a`, `b`.`b` AS `b` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0 AND `b`.`b` < 1000",
        )

    def test_rules_filter_multi_next_to_existing_filters(self):
        self.run_test_with_custom_filter_rule(
            [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "filter_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                },
                {
                    "scope": 0,
                    "attribute": '"b"."b"',
                    "filter_id": "some_guid",
                    "filter_definition": {"expression": "? < 1000"},
                },
            ],
            "SELECT a,b from b where (a < 0 and b > 1000) or a <> b",
            "SELECT `b`.`a` AS `a`, `b`.`b` AS `b` FROM `catalog`.`database`.`b` AS `b` WHERE ((`b`.`a` < 0 AND `b`.`b` > 1000) OR `b`.`a` <> `b`.`b`) AND `b`.`a` > 0 AND `b`.`b` < 1000",
        )

    def test_rules_filter_nested_cte(self):
        self.run_test_with_custom_filter_rule(
            [
                {
                    "scope": 1,
                    "attribute": '"b"."a"',
                    "filter_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                }
            ],
            "with base as (SELECT * FROM (SELECT a, b FROM b) AS subquery) SELECT * FROM base",
            "WITH `base` AS (SELECT `subquery`.`a` AS `a`, `subquery`.`b` AS `b` FROM (SELECT `b`.`a` AS `a`, `b`.`b` AS `b` FROM `catalog`.`database`.`b` AS `b` WHERE `b`.`a` > 0) AS `subquery`) SELECT `base`.`a` AS `a`, `base`.`b` AS `b` FROM `base` AS `base`",
        )

    def test_rules_filter_with_join(self):
        self.run_test_with_filter_rule(
            {"expression": "? > 0"},
            "select b.a from b as b inner join c as c on b.id = c.b_id",
            "SELECT `b`.`a` AS `a` FROM `catalog`.`database`.`b` AS `b` INNER JOIN `catalog`.`database`.`c` AS `c` ON `b`.`id` = `c`.`b_id` WHERE `b`.`a` > 0",
        )

    # in command usage:
    # [?] - CTAS (CREATE TABLE AS SELECT)
    # [] - CTAS (CREATE TABLE AS SELECT star)
    # [?] - DELETE FROM (SELECT)
    # [] - DELETE FROM (SELECT star)
    # [?] - DELETE FROM (multi-table join)
    # [?] - DELETE FROM (CTE)
    # [] - DELETE FROM (subquery)
    # [?] - INSERT INTO (select)
    # [] - INSERT INTO (select star)
    # [] - INSERT INTO (CTE)
    # [] - INSERT INTO (multi-table join)
    # [?] - INSERT INTO (subquery)
    # [] - INSERT INTO OVERWRITE (select)
    # [] - INSERT INTO OVERWRITE (select star)
    # [?] - UPDATE (FROM select)
    # [] - UPDATE (FROM select star)
    # [] - UPDATE (FROM CTE)
    # [] - UPDATE (FROM multi-table join)
    # [] - UPDATE (FROM subquery)

    @unittest.skip("Tests not implemented yet")
    def test_rules_filter_update_from_select(self):
        self.run_test_with_filter_rule(
            {},
            "UPDATE ",
            "",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_filter_insert_into_from_select(self):
        self.run_test_with_filter_rule(
            {},
            "INSERT INTO [a] SELECT * FROM (SELECT * FROM [b]) AS [subquery]",
            "",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_filter_delete_from_select(self):
        self.run_test_with_filter_rule(
            {},
            "DELETE FROM score_board WHERE name IN (select name from users where country = 'The Netherlands');",
            "",
        )

    def test_rules_filter_insert_into_subquery_result(self):
        self.run_test_with_custom_filter_rule(
            [
                {
                    "scope": 0,
                    "attribute": '"b"."a"',
                    "filter_id": "some_guid",
                    "filter_definition": {"expression": "? > 0"},
                }
            ],
            "INSERT INTO test (a, b) SELECT a,b from test2",
            "INSERT INTO test (a, b) SELECT `a` AS `a`, b AS `b` from test2 WHERE `a` > 0",
        )


    @unittest.skip("Tests not implemented yet")
    def test_rules_filter_delete_from_cte(self):
        self.run_test_with_filter_rule(
            {},
            "WITH foo_producers as (SELECT * from producers where producers.name = 'foo') DELETE FROM films USING foo_producers WHERE producer_id = foo_producers.id",
            "",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_filter_delete_from_multi_table_join(self):
        self.run_test_with_filter_rule(
            {},
            "DELETE FROM films USING producers WHERE producer_id = producers.id AND producers.name = 'foo';",
            "",
        )

    @unittest.skip("Tests not implemented yet")
    def test_rules_filter_create_table_as_select(self):
        self.run_test_with_filter_rule(
            {},
            "",
            "",
        )
