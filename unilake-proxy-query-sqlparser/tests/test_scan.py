import unittest

import sqlparser
from sqlparser import scan
from sqlparser.data import ScanOutputType


class TestScan(unittest.TestCase):
    def test_scan_empty_input(self):
        actual_output = scan("", "unilake", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.UNKNOWN)

    def test_output_select(self):
        actual_output = scan("select * from some_table", "unilake", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "SELECT")

    def test_output_insert(self):
        actual_output = scan(
            "insert into some_table select * from another_table", "unilake", "catalog", "database"
        )
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "INSERT")

    def test_output_update(self):
        sql = (
            """
            UPDATE
                Table_A
            SET
                Table_A.col1 = Table_B.col1,
                Table_A.col2 = Table_B.col2
            FROM
                Some_Table AS Table_A
                INNER JOIN Other_Table AS Table_B
                    ON Table_A.id = Table_B.id
            WHERE
                Table_A.col3 = 1
            """
        )
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "UPDATE")

    def test_output_ctas(self):
        actual_output = scan(
            "create table some_table as select * from employees", "unilake", "catalog", "database"
        )
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "CREATE")

    def test_scan_tsql_simple_query(self):
        sql = "SELECT a as [Something] from b"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(
            actual_output.to_json()["objects"],
            [
                {
                    "scope": 0,
                    "entities": [
                        {"catalog": "catalog", "db": "database", "alias": "b", "entity": "b"}
                    ],
                    "attributes": [{"entity": "b", "name": "a", "alias": "Something"}],
                    "is_agg": False,
                }
            ],
        )
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)

    def test_scan_tsql_simple_query_aggregate(self):
        sql = "SELECT a as [Something] from b group by 1"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(
            actual_output.to_json()["objects"],
            [
                {
                    "scope": 0,
                    "entities": [
                        {"catalog": "catalog", "db": "database", "entity": "b", "alias": "b"}
                    ],
                    "attributes": [
                        {"entity": "b", "name": "a", "alias": "Something"},
                        {"entity": "b", "name": "a", "alias": ""},
                    ],
                    "is_agg": True,
                }
            ],
        )
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)

    def test_scan_tsql_multi_scoped_query(self):
        sql = "with src as (SELECT a as [Something] from b), second as (select b as [Something] from b) select distinct * from src cross join second"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(len(actual_output.objects), 3)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)

    def test_scan_tsql_multi_scoped_query_from_join(self):
        sql = "SELECT [a].[a], [b].[b] FROM [a] JOIN [b] ON [a].[id] = [b].[id]"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(
            actual_output.to_json()["objects"],
            [
                {
                    "attributes": [
                        {"alias": "a", "entity": "a", "name": "a"},
                        {"alias": "b", "entity": "b", "name": "b"},
                        {"alias": "", "entity": "a", "name": "id"},
                        {"alias": "", "entity": "b", "name": "id"},
                    ],
                    "entities": [
                        {"alias": "a", "catalog": "catalog", "db": "database", "entity": "a"},
                        {"alias": "b", "catalog": "catalog", "db": "database", "entity": "b"},
                    ],
                    "is_agg": False,
                    "scope": 0,
                }
            ],
        )
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)

    def test_scan_large_query(self):
        with open("data/large_query.sql", "r") as file:
            sql = file.read()

        # start = time.time()
        # for i in range(0, 100):
        #     _ = scan(sql, "tsql", "catalog", "database")
        # end = time.time()
        # print("Time taken per item:", (end - start)/100)
        scan(sql, "snowflake", "catalog", "database")

    # Table Operations
    # [V] - UPDATE FROM SELECT
    # [V] - CREATE TABLE FROM SELECT
    # [?] - CREATE TABLE LIKE
    # [ ] - ATOMIC SWAP TABLE
    # [V] - INSERT INTO SELECT
    # [ ] - DELETE FROM SELECT
    # [ ] - TRUNCATE TABLE

    @unittest.skip("Unimplemented")
    def test_scan_create_table_like(self):
        # some statements should only be run within the same workspace or when same environment entities are used, CREATE TABLE LIKE is one of them
        # same goes for atomic swaps
        # todo(mrhamburg): check if create table like is used and report this as an internal to internal only query (for the PDP)
        pass

    def test_scan_create_table_from_select(self):
        sql = "CREATE TABLE `catalog`.`database`.`new_table` AS SELECT * FROM `catalog`.`database`.`old_table` AS `old_table`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.CREATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

    def test_scan_update_table_from_select(self):
        sql = "UPDATE `catalog`.`database`.`employees` SET `salary` = `salary` * 1.1 WHERE `salary` < (SELECT AVG(`employees`.`salary`) AS `_col_0` FROM `catalog`.`database`.`employees` AS `employees`)"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.UPDATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

    def test_scan_insert_into_select(self):
        sql = "INSERT INTO `catalog`.`database`.`employees` (`name`, `age`, `salary`) SELECT `employees_salary`.`name` AS `name`, `employees_salary`.`age` AS `age`, `employees_salary`.`salary` * 1.1 AS `salary` FROM `catalog`.`database`.`employees_salary` AS `employees_salary`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.INSERT)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)
