import unittest
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
            ""
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

    def test_scan_large_query(self):
        with open("data/large_query.sql", "r") as file:
            sql = file.read()

        # start = time.time()
        # for i in range(0, 100):
        #     _ = scan(sql, "tsql", "catalog", "database")
        # end = time.time()
        # print("Time taken per item:", (end - start)/100)
        scan(sql, "snowflake", "catalog", "database")
