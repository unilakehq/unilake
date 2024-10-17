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

    def test_scan_valid_order_by(self):
        sql = "SELECT hire_data FROM employees ORDER BY salary DESC, hire_date ASC"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertEqual(len(actual_output.objects[0].attributes), 1)

    def test_scan_valid_group_by(self):
        sql = "SELECT department FROM employees GROUP BY department, employee_id"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertEqual(len(actual_output.objects[0].attributes), 1)

    def test_scan_get_star(self):
        sql = "SELECT departments.name, employees.* FROM employees JOIN departments ON employees.department_id = departments.department_id"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertEqual(len(actual_output.objects[0].entities), 2)
        self.assertEqual(actual_output.objects[0].attributes[0].name, "*")

    def test_scan_get_star_in_function(self):
        # I believe this is only applicable for count(*), but who knows
        sql = "SELECT COUNT(*) FROM employees"
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertEqual(len(actual_output.objects[0].entities), 1)
        self.assertEqual(len(actual_output.objects[0].attributes), 0)

    def test_scan_get_attribute_in_functions(self):
        # I believe this is only applicable for count(*), but who knows
        sql = "SELECT UPPER(CONCAT('Value: ', TRIM(LEADING '0' FROM column_name))) AS formatted_value FROM your_table"
        actual_output = scan(sql, "mysql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertEqual(len(actual_output.objects[0].entities), 1)
        self.assertEqual(len(actual_output.objects[0].attributes), 1)
        self.assertEqual(actual_output.objects[0].attributes[0].name, "column_name")

    # Table Operations
    # [V] - UPDATE FROM SELECT
    # [V] - CREATE TABLE FROM SELECT
    # [V] - CREATE TABLE LIKE
    # [/] - ATOMIC SWAP TABLE
    # [V] - INSERT INTO SELECT
    # [V] - DELETE FROM SELECT
    # [/] - TRUNCATE TABLE
    # [V] - CREATE VIEW
    # [V] - CREATE MATERIALIZED VIEW

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
        self.assertEqual(len(result.objects), 1)

        entity = result.objects[0].entities[0]
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "old_table")
        self.assertEqual(entity.alias, "old_table")
        self.assertEqual(result.target_entity, '"catalog"."database"."new_table"')

    def test_scan_update_table_from_select(self):
        sql = "UPDATE `catalog`.`database`.`employees` SET `salary` = `salary` * 1.1 WHERE `salary` < (SELECT AVG(`employees`.`salary`) AS `_col_0` FROM `catalog`.`database`.`employees` AS `employees`)"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.UPDATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "employees")
        self.assertEqual(entity.alias, "employees")
        self.assertEqual(result.target_entity, '"catalog"."database"."employees"')

    def test_scan_delete_from_cte(self):
        sql = "WITH `foo_producers` AS (SELECT * FROM `catalog`.`database`.`producers` AS `producers` WHERE `producers`.`name` = 'foo') DELETE FROM `catalog`.`database`.`films` USING `catalog`.`database`.`foo_producers` WHERE `producer_id` = `foo_producers`.`id`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.DELETE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "producers")
        self.assertEqual(entity.alias, "producers")
        self.assertEqual(result.target_entity, '"catalog"."database"."films"')

    def test_scan_insert_into_select(self):
        sql = "INSERT INTO `catalog`.`database`.`employees` (`name`, `age`, `salary`) SELECT `employees_salary`.`name` AS `name`, `employees_salary`.`age` AS `age`, `employees_salary`.`salary` * 1.1 AS `salary` FROM `catalog`.`database`.`employees_salary` AS `employees_salary`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.INSERT)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "employees_salary")
        self.assertEqual(entity.alias, "employees_salary")
        self.assertEqual(result.target_entity, '"catalog"."database"."employees"')

    def test_scan_create_view(self):
        sql = "CREATE VIEW `catalog`.`database`.`v_employees` AS SELECT `employees`.`name` AS `name`, `employees`.`age` AS `age`, `employees`.`salary` AS `salary` FROM `catalog`.`database`.`employees` AS `employees`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.CREATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(len(result.objects[0].attributes), 3)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "employees")
        self.assertEqual(entity.alias, "employees")
        self.assertEqual(result.target_entity, '"catalog"."database"."v_employees"')

    def test_scan_create_materialized_view(self):
        sql = "CREATE MATERIALIZED VIEW `catalog`.`database`.`k2_order` AS SELECT `duplicate_table`.`k2` AS `k2`, `duplicate_table`.`k1` AS `k1` FROM `catalog`.`database`.`duplicate_table` AS `duplicate_table` ORDER BY `k2`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.CREATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(len(result.objects[0].attributes), 2)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "duplicate_table")
        self.assertEqual(entity.alias, "duplicate_table")
        self.assertEqual(result.target_entity, '"catalog"."database"."k2_order"')

    def test_scan_truncate_table(self):
        sql = "TRUNCATE TABLE `catalog`.`database`.`employees`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.TRUNCATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "employees")
        self.assertEqual(entity.alias, "employees")
        self.assertEqual(result.target_entity, '"catalog"."database"."employees"')

    def test_scan_create_table_like(self):
        sql = "CREATE TABLE `catalog`.`test1`.`table2` LIKE `catalog`.`test1`.`table1`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.CREATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 2)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "test1")
        self.assertEqual(entity.name, "table2")
        self.assertEqual(entity.alias, "")
        self.assertEqual(result.target_entity, '"catalog"."test1"."table2"')

    @unittest.skip("Waiting for sqlglot to support atomic swap table")
    def test_scan_atomic_swap_table(self):
        # TODO: probably requires a PR in sqlglot to support this command
        # SEE: https://github.com/tobymao/sqlglot/pull/4256
        sql = "ALTER TABLE `catalog`.`database`.`table1` SWAP WITH `catalog`.`database`.`table2`"
        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.ALTER)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        entity = result.objects[0].entities[0]
        self.assertEqual(len(result.objects[0].entities), 2)
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "table1")
        self.assertEqual(entity.alias, "")
        self.assertEqual(result.target_entity, '"catalog"."database"."table1"')

    # todo(mrhamburg): scan should also check for the query correctness and return an error if it's not valid', same goes for transpiler
    # see: https://sqlglot.com/sqlglot.html#parser-errors