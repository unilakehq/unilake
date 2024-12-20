import unittest

import sqlparser
from sqlparser import scan
from sqlparser.data import ScanOutputType, ScanAttribute, ScanEntity


class TestScan(unittest.TestCase):
    def test_scan_empty_input(self):
        actual_output = scan("", "unilake", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.UNKNOWN)

    def test_scan_invalid_input(self):
        sql = "SELECT SUM(Amount( FROM Finance"
        actual_output = scan(sql, "snowflake", "catalog", "database")
        self.assertIsNotNone(actual_output.error)

    def test_scan_malformed_input(self):
        sql = "select a"

        actual_output = scan(sql, "snowflake", "catalog", "database")
        self.assertIsNotNone(actual_output.error)
        self.assertEqual(len(actual_output.objects), 0)

    def test_scan_literal_only_input(self):
        sql = "select 1"

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(len(actual_output.objects), 1)
        self.assertEqual(actual_output.objects[0].entities, set())
        self.assertEqual(actual_output.objects[0].attributes, set())

    def test_output_select(self):
        actual_output = scan("select * from some_table", "unilake", "catalog", "database")
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='some_table', alias='some_table'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='some_table', name='*'),
        }

        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "SELECT")
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_output_select_aliased(self):
        actual_output = scan("select b.* from some_table as b", "unilake", "catalog", "database")
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='some_table', alias='b'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='b', name='*'),
        }

        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "SELECT")
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_output_select_with_where_single_attribute(self):
        actual_output = scan("select b.a from some_table as b where b.a = 0", "unilake", "catalog", "database")
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='some_table', alias='b'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='b', name='a'),
        }

        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "SELECT")
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_output_select_with_where_double_attribute(self):
        actual_output = scan("select b.a, func(b.a) from some_table as b", "unilake", "catalog", "database")
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='some_table', alias='b'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='b', name='a'),
        }

        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "SELECT")
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_output_entity(self):
        actual_output = scan(
            "create table some_catalog.some_schema.some_table as select * from some_other_catalog.some_schema.some_table",
            "unilake",
            "catalog",
            "database",
        )
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "CREATE")
        self.assertEqual(actual_output.target_entity, '"some_catalog"."some_schema"."some_table"')

    def test_output_set_command(self):
        actual_output = scan("set some_var=10", "unilake", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "SET")
        self.assertEqual(actual_output.target_entity, None)

    def test_output_insert(self):
        actual_output = scan(
            "insert into some_table select * from another_table", "unilake", "catalog", "database"
        )
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "INSERT")

    def test_output_update(self):
        sql = """
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
        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "UPDATE")

    def test_output_ctas(self):
        sql = "create table some_table as select * from employees"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='employees', name='*'),
        }

        actual_output = scan(
            sql, "unilake", "catalog", "database"
        )
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type.value, "CREATE")
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_tsql_simple_query(self):
        sql = "SELECT a as [Something] from b"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='b', alias='b'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='b', name='a'),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_tsql_simple_query_aggregate(self):
        sql = "SELECT a as [Something] from b group by 1"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='b', alias='b'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='b', name='a'),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_tsql_multi_scoped_query(self):
        sql = "with src as (SELECT a as [Some_A] from b), second as (select b as [Some_B] from b) select distinct * from src cross join second"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='b', alias='b'),
        }
        expected_entities_2 = {
            ScanEntity(catalog='catalog', db='database', name='b', alias='b'),
            ScanEntity(catalog=None, db=None, name='second', alias='second'),
            ScanEntity(catalog=None, db=None, name='src', alias='src')
        }
        expected_attributes_0 = {
            ScanAttribute(entity_alias='b', name='a'),
        }
        expected_attributes_1 = {
            ScanAttribute(entity_alias='b', name='b'),
        }
        expected_attributes_2 = {
            ScanAttribute(entity_alias='src', name='Some_A'),
            ScanAttribute(entity_alias='second', name='Some_B'),
            ScanAttribute(entity_alias='b', name='a'),
            ScanAttribute(entity_alias='b', name='b')
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(len(actual_output.objects), 3)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[1].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[2].entities, expected_entities_2)

        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes_0)
        self.assertSetEqual(actual_output.objects[1].attributes, expected_attributes_1)
        self.assertSetEqual(actual_output.objects[2].attributes, expected_attributes_2)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)

    def test_scan_tsql_multi_scoped_query_from_join(self):
        sql = "SELECT [x].[a], [b].[b] FROM [a] as [x] JOIN [b] ON [x].[id] = [b].[id]"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='a', alias='x'),
            ScanEntity(catalog='catalog', db='database', name='b', alias='b')
        }
        expected_attributes = {
            ScanAttribute(entity_alias='x', name='a'),
            ScanAttribute(entity_alias='b', name='b'),
            ScanAttribute(entity_alias='x', name='id'),
            ScanAttribute(entity_alias='b', name='id'),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)

    def test_scan_large_query(self):
        with open("data/large_query.sql", "r") as file:
            sql = file.read()

        # start = time.time()
        # for i in range(0, 100):
        #     _ = scan(sql, "tsql", "catalog", "database")
        # end = time.time()
        # print("Time taken per item:", (end - start)/100)
        result = scan(sql, "snowflake", "catalog", "database")
        self.assertIsNone(result.error)
        

    def test_scan_valid_order_by(self):
        sql = "SELECT hire_date FROM employees ORDER BY salary DESC, fire_date ASC"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees')
        }
        expected_attributes = {
            ScanAttribute(entity_alias='employees', name='hire_date'),
            ScanAttribute(entity_alias='employees', name='fire_date'),
            ScanAttribute(entity_alias='employees', name='salary'),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_valid_group_by(self):
        sql = "SELECT department, count(employee_id) FROM employees GROUP BY department, employee_id"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees')
        }
        expected_attributes = {
            ScanAttribute(entity_alias='employees', name='department'),
            ScanAttribute(entity_alias='employees', name='employee_id'),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)

    def test_scan_get_star(self):
        sql = "SELECT departments.name, employees.* FROM employees JOIN departments ON employees.department_id = departments.department_id"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees'),
            ScanEntity(catalog='catalog', db='database', name='departments', alias='departments')
        }
        expected_attributes = {
            ScanAttribute(entity_alias='departments', name='department_id'),
            ScanAttribute(entity_alias='employees', name='department_id'),
            ScanAttribute(entity_alias='departments', name='name'),
            ScanAttribute(entity_alias='employees', name='*'),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_get_duplicate_star(self):
        # todo: also add this to transpile to see what happens
        sql = "SELECT * FROM catalog.schema_1.employees, catalog.schema_2.employees WHERE catalog.schema_1.employees.id = catalog.schema_2.employees.id"
        expected_entities = {
            ScanEntity(catalog='catalog', db='schema_1', name='employees', alias='employees'),
            ScanEntity(catalog='catalog', db='schema_2', name='employees', alias='employees_2')
        }
        expected_items = {
            ScanAttribute(entity_alias="employees", name="id"),
            ScanAttribute(entity_alias="employees_2", name="id"),
            ScanAttribute(entity_alias="employees", name="*"),
        }

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertEqual(len(actual_output.objects[0].entities), 2)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_items)

    def test_scan_get_star_in_function(self):
        # we should ignore star expansions for these kind of queries (these are exceptional)
        sql = "SELECT COUNT(*) FROM employees"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees'),
        }
        expected_attributes = set()

        actual_output = scan(sql, "tsql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_get_attribute_in_functions(self):
        sql = "SELECT UPPER(CONCAT('Value: ', TRIM(LEADING '0' FROM column_name))) AS formatted_value FROM your_table"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='your_table', alias='your_table'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='your_table', name='column_name'),
        }

        actual_output = scan(sql, "mysql", "catalog", "database")
        self.assertIsNone(actual_output.error)
        self.assertEqual(actual_output.type, ScanOutputType.SELECT)
        self.assertSetEqual(actual_output.objects[0].entities, expected_entities)
        self.assertSetEqual(actual_output.objects[0].attributes, expected_attributes)

    def test_scan_error_sql_incorrect_format(self):
        # I believe this is only applicable for count(*), but who knows
        sql = "SELECT foo FROM (SELECT baz FROM t"
        actual_output = scan(sql, "mysql", "catalog", "database")
        self.assertIsNotNone(actual_output.error)

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

        entity = result.objects[0].entities.pop()
        self.assertEqual(entity.catalog, "catalog")
        self.assertEqual(entity.db, "database")
        self.assertEqual(entity.name, "old_table")
        self.assertEqual(entity.alias, "old_table")
        self.assertEqual(result.target_entity, '"catalog"."database"."new_table"')

    def test_scan_update_table_from_select(self):
        sql = "UPDATE `catalog`.`database`.`employees` SET `salary` = `salary` * 1.1 WHERE `salary` < (SELECT AVG(`employees`.`salary`) AS `_col_0` FROM `catalog`.`database`.`employees` AS `employees`)"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='employees', name='salary'),
        }

        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.UPDATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)
        self.assertSetEqual(result.objects[0].entities, expected_entities)
        self.assertSetEqual(result.objects[0].attributes, expected_attributes)
        self.assertEqual(result.target_entity, '"catalog"."database"."employees"')

    def test_scan_delete_from_cte(self):
        sql = "WITH `foo_producers` AS (SELECT * FROM `catalog`.`database`.`producers` AS `producers` WHERE `producers`.`name` = 'foo') DELETE FROM `catalog`.`database`.`films` USING `catalog`.`database`.`foo_producers` WHERE `producer_id` = `foo_producers`.`id`"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='producers', alias='producers'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='producers', name='*'),
            ScanAttribute(entity_alias='producers', name='name'),
        }

        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.DELETE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertSetEqual(result.objects[0].entities, expected_entities)
        self.assertSetEqual(result.objects[0].attributes, expected_attributes)
        self.assertEqual(result.target_entity, '"catalog"."database"."films"')

    def test_scan_insert_into_select(self):
        sql = "INSERT INTO `catalog`.`database`.`employees` (`name`, `age`, `salary`) SELECT `employees_salary`.`name` AS `name`, `employees_salary`.`age` AS `age`, `employees_salary`.`salary` * 1.1 AS `salary` FROM `catalog`.`database`.`employees_salary` AS `employees_salary`"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees_salary', alias='employees_salary'),
        }
        result = scan(sql, "starrocks", "catalog", "database")

        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.INSERT)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertSetEqual(result.objects[0].entities, expected_entities)
        self.assertEqual(result.target_entity, '"catalog"."database"."employees"')

    def test_scan_create_view(self):
        sql = "CREATE VIEW `catalog`.`database`.`v_employees` AS SELECT `employees`.`name` AS `name`, `employees`.`age` AS `age`, `employees`.`salary` AS `salary` FROM `catalog`.`database`.`employees` AS `employees`"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='employees', alias='employees'),
        }
        expected_attributes = {
            ScanAttribute(entity_alias='employees', name='name'),
            ScanAttribute(entity_alias='employees', name='age'),
            ScanAttribute(entity_alias='employees', name='salary'),
        }

        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.CREATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(len(result.objects[0].attributes), 3)
        self.assertSetEqual(result.objects[0].entities, expected_entities)
        self.assertSetEqual(result.objects[0].attributes, expected_attributes)
        self.assertEqual(result.target_entity, '"catalog"."database"."v_employees"')

    def test_scan_create_materialized_view(self):
        sql = "CREATE MATERIALIZED VIEW `catalog`.`database`.`k2_order` AS SELECT `duplicate_table`.`k2` AS `k2`, `duplicate_table`.`k1` AS `k1` FROM `catalog`.`database`.`duplicate_table` AS `duplicate_table` ORDER BY `k2`"
        expected_entities = {
            ScanEntity(catalog='catalog', db='database', name='duplicate_table', alias='duplicate_table'),
        }

        result = scan(sql, "starrocks", "catalog", "database")
        self.assertIsNone(result.error)
        self.assertIs(result.type, ScanOutputType.CREATE)
        testing = sqlparser.transpile(result.to_json())
        self.assertEqual(testing.sql_transformed, sql)

        self.assertEqual(len(result.objects[0].entities), 1)
        self.assertEqual(len(result.objects[0].attributes), 3)
        self.assertSetEqual(result.objects[0].entities, expected_entities)
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
