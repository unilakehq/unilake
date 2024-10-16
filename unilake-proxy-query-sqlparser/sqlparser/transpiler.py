from typing import Type

from sqlglot import parse_one, exp, Expression, maybe_parse
from sqlglot.expressions import replace_placeholders
from sqlglot.optimizer import traverse_scope
from sqlglot.optimizer.qualify import qualify

from sqlparser.dialects import Unilake
from sqlparser.data import (
    ScanOutput,
    ScanEntity,
    ScanAttribute,
    ScanOutputType,
    ScanOutputObject,
    TranspilerOutput,
    ErrorMessage,
    TranspilerInput,
)

OUTPUT_DIALECT = "starrocks"


def _get_dialect(dialect: str) -> str | Type[Unilake]:
    if dialect == "unilake":
        return Unilake
    return dialect


def _scan_transform(node, scope_id: int, entities, attributes, aggregates):
    node_type = type(node)

    # Get tables
    if node_type is exp.Table:
        entities[scope_id].append(ScanEntity(node.catalog, node.db, node.name, node.alias))

    # Get columns
    elif node_type is exp.Column:
        alias = node.alias
        if node.parent is not None and node.parent.key == "alias":
            alias = node.parent.alias
        attributes[scope_id].append(ScanAttribute(node.table, node.name, alias))

    # Get Stars
    elif node_type is exp.Select and node.is_star:
        found = node.find(exp.From)
        if found is not None:
            attributes[scope_id].append(ScanAttribute(found.this.name, "*", found.this.alias))

    # Get groupBy check indicator
    elif node_type is exp.Group:
        aggregates.append(scope_id)
    return node


def inner_scan(sql: str, dialect: str, catalog: str, database: str) -> ScanOutput:
    if not sql:
        return ScanOutput(
            objects=[], dialect=dialect, query={"query": sql}, type=ScanOutputType.UNKNOWN, error=None
        )
    dialect = _get_dialect(dialect)

    parsed = parse_one(sql, dialect=dialect)
    parsed = qualify(parsed, catalog=catalog, db=database)
    scoped = traverse_scope(parsed)

    entities = [[]]
    attributes = [[]]
    aggregates = []

    query_type = ScanOutputType.from_key(parsed.key)
    objects = []

    # scoped query
    if scoped:
        for i, scope in enumerate(scoped):
            entities.append([])
            attributes.append([])
            scope.expression.transform(
                _scan_transform, i, entities, attributes, aggregates
            )
            objects.append(
                ScanOutputObject(
                    scope=i, entities=entities[i], attributes=attributes[i], is_agg=i in aggregates
                )
            )

        return ScanOutput(
            objects=objects,
            dialect=dialect,
            query=parsed.dump(),
            type=query_type,
            error=None,
        )
    # non-scoped query
    else:
        parsed.transform(_scan_transform, 0, entities, attributes, aggregates)
        objects.append(
            ScanOutputObject(scope=0, entities=entities[0], attributes=attributes[0], is_agg=False)
        )
        return ScanOutput(
            objects=objects,
            dialect=dialect,
            query=parsed.dump(),
            type=query_type,
            error=None,
        )


def _transform_filters(node: exp.Select, scope_id: int, filter_lookup: dict):
    filters = []
    for select in node.selects:
        node_str = str(select.this)
        found = filter_lookup.get(hash((scope_id, node_str)))
        if found:
            cond = maybe_parse(found["expression"], into=exp.Condition, dialect=OUTPUT_DIALECT)
            filters.append(replace_placeholders(cond, select.this))

    if filters:
        node.where(*filters, append=True, dialect=OUTPUT_DIALECT, copy=False)

    return node


def _transform_predicate(node: exp.Column, scope_id: int, rule_lookup: dict):
    node_str = str(node)
    found = rule_lookup.get(hash((scope_id, node_str)))
    if not found:
        return node

    match found["name"]:
        case "xxhash3":
            # XX_HASH3_128(col)
            return exp.func("XX_HASH3_128", node, dialect=OUTPUT_DIALECT)
        case "replace_null":
            # NULL
            return exp.Null()
        case "replace_char":
            # repeat('x', LENGTH(col))
            node_str = exp.to_column(node_str)
            literal = found["properties"]["replacement"]
            return exp.Repeat(
                this=exp.Literal(this=literal, is_string=True),
                times=exp.Length(this=node_str),
                dialect=OUTPUT_DIALECT,
                copy=False,
            )
        case "replace_string":
            # 'x'
            literal = found["properties"]["replacement"]
            return exp.Literal(this=literal, is_string=True)
        case "mask_except_last":
            # concat(REPEAT('x', LENGTH(col)-2), RIGHT(col, 2))
            node_str = exp.to_column(node_str)
            value_lit = exp.Literal(this=found["properties"]["value"], is_string=True)
            len_lit = exp.Literal(this=found["properties"]["len"], is_string=False)
            return exp.Concat(
                expressions=[
                    exp.Repeat(
                        this=value_lit,
                        times=exp.Sub(this=exp.Length(this=node_str), expression=len_lit),
                    ),
                    exp.Right(this=node_str, expression=len_lit),
                ],
                safe=True,
                coalesce=False,
                dialect=OUTPUT_DIALECT,
            )
        case "mask_except_first":
            # concat(LEFT(col, 2), REPEAT('x', LENGTH(col)-2))
            node_str = exp.to_column(node_str)
            value_lit = exp.Literal(this=found["properties"]["value"], is_string=True)
            len_lit = exp.Literal(this=found["properties"]["len"], is_string=False)
            return exp.Concat(
                expressions=[
                    exp.Left(this=node_str, expression=len_lit),
                    exp.Repeat(
                        this=value_lit,
                        times=exp.Sub(this=exp.Length(this=node_str), expression=len_lit),
                    ),
                ],
                safe=True,
                coalesce=False,
                dialect=OUTPUT_DIALECT,
            )
        case "rounding":
            # round(col, x)
            node_str = exp.to_column(node_str)
            value_lit = exp.Literal(this=found["properties"]["value"], is_string=False)
            return exp.func("round", node_str, value_lit, dialect=OUTPUT_DIALECT)
        case "random_number":
            # floor((max-min+1) * rand()+min)
            min_lit = exp.Literal(this=found["properties"]["min"], is_string=False)
            max_lit = exp.Literal(this=found["properties"]["max"], is_string=False)
            one_lit = exp.Literal(this="1", is_string=False)
            return exp.Floor(
                this=exp.Add(
                    this=exp.Mul(
                        this=exp.Paren(
                            this=exp.Add(
                                this=exp.Sub(this=max_lit, expression=min_lit), expression=one_lit
                            )
                        ),
                        expression=exp.Rand(),
                    ),
                    expression=min_lit,
                )
            )
        case "random_multiplication":
            # rand()*x
            max_lit = exp.Literal(this=found["properties"]["max"], is_string=False)
            return exp.Mul(this=exp.Rand(), expression=max_lit, dialect=OUTPUT_DIALECT)
        case "left":
            # left('hello', 3)
            return exp.func("left", node, found["properties"]["len"], dialect=OUTPUT_DIALECT)
        case "right":
            # right('hello', 3)
            return exp.func("right", node, found["properties"]["len"], dialect=OUTPUT_DIALECT)
        case "mail_hash_pres":
            return node
        case "mail_mask_pres":
            return node
        case "mail_mask_username":
            # CONCAT_WS('@', REPEAT('x', LOCATE('@', ?) - 1), SPLIT_PART(?, '@', 2))
            node_str = exp.to_column(node_str)
            at_lit = exp.Literal(this="@", is_string=True)
            instr_exp = exp.StrPosition(substr=at_lit, this=node_str)
            repeat_exp = exp.Repeat(
                this=exp.Literal(this="x", is_string=True),
                times=exp.Sub(this=instr_exp, expression=exp.Literal(this="1", is_string=False)),
            )
            split_part_exp = exp.func(
                "split_part", node_str, at_lit, exp.Literal(this="2", is_string=False)
            )
            return exp.ConcatWs(
                expressions=[at_lit, repeat_exp, split_part_exp],
                safe=True,
                coalesce=False,
                dialect=OUTPUT_DIALECT,
                copy=False,
            )
        case "mail_mask_domain":
            # CONCAT_WS('@',SPLIT_PART(?, '@', 1),CONCAT(REPEAT('x', CHAR_LENGTH(SPLIT_PART(EmailAddress, '@', 2)) - CHAR_LENGTH(SPLIT_PART(SPLIT_PART(EmailAddress, '@', 2), '.', -1)) - 1),'.',SPLIT_PART(SPLIT_PART(EmailAddress, '@', 2), '.', -1)))
            node_str = exp.to_column(node_str)
            at_lit = exp.Literal(this="@", is_string=True)
            x_lit = exp.Literal(this="x", is_string=True)
            one_lit = exp.Literal(this="1", is_string=False)
            dot_lit = exp.Literal(this=".", is_string=True)
            split_part_exp_1 = exp.func(
                "split_part", node_str, at_lit, exp.Literal(this="1", is_string=False)
            )
            split_part_exp_2 = exp.func(
                "split_part", split_part_exp_1, at_lit, exp.Literal(this="2", is_string=False)
            )
            times_exp = exp.Sub(
                this=exp.Sub(
                    this=exp.func("CHAR_LENGTH", split_part_exp_2),
                    expression=exp.func(
                        "CHAR_LENGTH",
                        expressions=[
                            exp.func(
                                "split_part",
                                expressions=[split_part_exp_2, dot_lit, exp.Neg(this=one_lit)],
                            )
                        ],
                    ),
                )
            )
            concat_exp = exp.Concat(expressions=[exp.Repeat(this=x_lit, times=times_exp)])
            return exp.ConcatWs(expressions=[at_lit, split_part_exp_1, concat_exp])
        case "cc_hash_pres":
            # todo, cc = creditcard
            return node
        case "cc_mask_pres":
            return node
        case "cc_last_four":
            # todo: last_four digits of creditcard, rest is asterixed ***
            return node
        case "date_year_only":
            # date_trunc('year', col)
            node_str = exp.to_column(node_str)
            value_lit = exp.Var(this="YEAR")
            return exp.TimestampTrunc(this=node_str, unit=value_lit, dialect=OUTPUT_DIALECT)
        case "date_month_only":
            # date_trunc('month', col)
            node_str = exp.to_column(node_str)
            value_lit = exp.Var(this="MONTH")
            return exp.TimestampTrunc(this=node_str, unit=value_lit, dialect=OUTPUT_DIALECT)
        case "ip_anonymize":
            # CONCAT_WS('.', SPLIT_PART(IpAddress, '.', 1), SPLIT_PART(IpAddress, '.', 2), '0', '0')
            node_str = exp.to_column(node_str)
            dot_lit = exp.Literal(this=".", is_string=True)
            return exp.ConcatWs(
                expressions=[
                    exp.Literal(this=".", is_string=True),
                    exp.func(
                        "split_part", node_str, dot_lit, exp.Literal(this="1", is_string=False)
                    ),
                    exp.func(
                        "split_part", node_str, dot_lit, exp.Literal(this="2", is_string=False)
                    ),
                    exp.Literal(this="0", is_string=True),
                    exp.Literal(this="0", is_string=True),
                ],
                dialect=OUTPUT_DIALECT,
            )
        case "ip_hash_pres":
            # TODO, 192.168.1.15 => hua.1ha.i.a8
            return node
        case "ip_mask_pres":
            # CONCAT_WS('.',
            #            REPEAT('*', CHAR_LENGTH(SPLIT_PART(?, '.', 1))),
            #            REPEAT('*', CHAR_LENGTH(SPLIT_PART(?, '.', 2))),
            #            REPEAT('*', CHAR_LENGTH(SPLIT_PART(?, '.', 3))),
            #            REPEAT('*', CHAR_LENGTH(SPLIT_PART(?, '.', 4))))
            node_str = exp.to_column(node_str)
            dot_lit = exp.Literal(this=".", is_string=True)
            star_lit = exp.Literal(this="*", is_string=True)
            return exp.ConcatWs(
                expressions=[
                    dot_lit,
                    exp.Repeat(
                        this=star_lit,
                        times=exp.func(
                            "CHAR_LENGTH",
                            exp.func(
                                "split_part",
                                node_str,
                                dot_lit,
                                exp.Literal(this="1", is_string=False),
                            ),
                        ),
                    ),
                    exp.Repeat(
                        this=star_lit,
                        times=exp.func(
                            "CHAR_LENGTH",
                            exp.func(
                                "split_part",
                                node_str,
                                dot_lit,
                                exp.Literal(this="2", is_string=False),
                            ),
                        ),
                    ),
                    exp.Repeat(
                        this=star_lit,
                        times=exp.func(
                            "CHAR_LENGTH",
                            exp.func(
                                "split_part",
                                node_str,
                                dot_lit,
                                exp.Literal(this="3", is_string=False),
                            ),
                        ),
                    ),
                    exp.Repeat(
                        this=star_lit,
                        times=exp.func(
                            "CHAR_LENGTH",
                            exp.func(
                                "split_part",
                                node_str,
                                dot_lit,
                                exp.Literal(this="4", is_string=False),
                            ),
                        ),
                    ),
                ],
                dialect=OUTPUT_DIALECT,
            )
        case "semi_structured":
            # TODO, would be nice to have a way to mask or hash semi-structured data
            return node
        case _:
            return node


def _transform_star(node: exp.Star, scope_id: int):
    # todo(mrhamburg): handle star expressions and expansion, based on input json
    # unset star and add security predicates to all columns
    return node
    node.pop()
    for column in rule_lookup.get("some_star_info"):
        col = exp.column(
            this=exp.Identifier(this=column["column_name"], quoted=True),
            table=exp.Identifier(this=column["table_alias"], quoted=True),
        )
        col = transform_predicate(col)
        node.parent_select.expressions.append(col)
    # todo: get the actual table and transform the alias of that, if needed
    return node


def _hide_literals(node: exp.Literal):
    node.set("this", "?", overwrite=True)
    return node


def _transformer(node, scope_id: int, rule_lookup: dict, filter_lookup: dict):
    if node.is_star:
        return _transform_star(node, scope_id)
    elif isinstance(node, exp.Select):
        return _transform_filters(node, scope_id, filter_lookup)
    # we only need to do this for the secured output
    elif isinstance(node, exp.Column):
        return _transform_predicate(node, scope_id, rule_lookup)
    return node


def _transformer_hide_literals(node):
    if isinstance(node, exp.Literal):
        return _hide_literals(node)
    return _transformer(node)


def inner_transpile(source: dict) -> TranspilerOutput:
    # check input
    if source is None:
        return TranspilerOutput(
            sql="",
            sql_transformed="",
            sql_transformed_secure="",
            error=ErrorMessage(msg="Missing input", line=1, column=1),
        )

    # parse input
    transpiler_input = TranspilerInput.from_json(source)
    if (
        transpiler_input is None
        or transpiler_input.query is None
        or len(transpiler_input.query.keys()) == 0
    ):
        return TranspilerOutput(
            sql_transformed="",
            sql_transformed_secure="",
            error=ErrorMessage(msg="Invalid input", line=1, column=1),
        )

    # set environment lookups
    rule_lookup = {
        hash((rule.scope, rule.attribute)): rule.rule_definition for rule in transpiler_input.rules
    }
    filter_lookup = {
        hash((filter.scope, filter.attribute)): filter.filter_definition
        for filter in transpiler_input.filters
    }

    # transform input
    input_sql = Expression.load(transpiler_input.query)
    scoped = traverse_scope(input_sql)

    if scoped:
        for i, scope in enumerate(scoped):
            scope.expression.transform(
                _transformer, i, rule_lookup, filter_lookup, copy=False
            )

    return TranspilerOutput(
        sql_transformed=str(input_sql.sql(OUTPUT_DIALECT)),
        sql_transformed_secure="",
        error=None,
    )
