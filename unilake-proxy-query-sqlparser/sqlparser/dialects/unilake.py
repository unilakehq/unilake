from sqlglot import exp, tokens, Parser, Generator
from sqlglot.dialects.dialect import Dialect
from sqlglot.tokens import Tokenizer, TokenType


class Unilake(Dialect):
    class Tokenizer(Tokenizer):
        QUOTES = ["'", '"']
        IDENTIFIERS = ["[", "]"]

        KEYWORDS = {
            **Tokenizer.KEYWORDS,
            "MASKING": TokenType.DEFAULT,
            "RULESET": TokenType.DEFAULT,
            "POLICY": TokenType.DEFAULT,
            "CONDITION": TokenType.DEFAULT,
            "SECURITY": TokenType.DEFAULT,
            "DATASET": TokenType.DEFAULT,
            "ACCESS": TokenType.DEFAULT,
            "USAGE": TokenType.DEFAULT,
            "TRANSPILE": TokenType.DEFAULT,
            "SCAN": TokenType.DEFAULT,
            "RESET": TokenType.DEFAULT,
            "IMPERSONATE": TokenType.DEFAULT,
            "EXECUTE": TokenType.DEFAULT,
        }

        COMMANDS = {*tokens.Tokenizer.COMMANDS, TokenType.END}

    class Parser(Parser):
        STATEMENT_PARSERS = {
            **Parser.STATEMENT_PARSERS,
            TokenType.CREATE: lambda self: self._parse_create(),
            TokenType.UPDATE: lambda self: self._parse_update(),
            TokenType.DELETE: lambda self: self._parse_delete(),
            TokenType.DESCRIBE: lambda self: self._parse_describe(),
            TokenType.DEFAULT: lambda self: self._parse_default(),
            TokenType.USE: lambda self: self._parse_use(),
            TokenType.SET: lambda self: self._parse_set(),
            TokenType.COMMAND: lambda self: self._parse_command(),
        }

        def _parse_default(self) -> exp.Command:
            if self._prev.text == "TRANSPILE":
                return self._parse_transpile()
            if self._prev.text == "SCAN":
                return self._parse_scan()
            if self._prev.text == "RESET":
                return self._parse_reset()
            if self._prev.text == "IMPERSONATE":
                return self._parse_impersonate()
            if self._prev.text == "EXECUTE":
                return self._parse_execute()

            self.raise_error("Unrecognized command '%s'" % self._prev.text)

        def _advance_and_consume(self) -> str:
            start = self._curr
            while self._curr:
                self._advance()
            return self._find_sql(start, self._prev)

        def _match_and_expect_token(self, token_type:TokenType, error_message: str):
            if not self._match(token_type):
                self.raise_error(error_message)

        def _match_and_expect_text(self, text: str, error_message: str):
            if not self._match_text_seq(text):
                self.raise_error(error_message)

        def _match_and_parse_string(self, error_message: str) -> str:
            if not self._match(TokenType.STRING, advance=False):
                self.raise_error(error_message)
            return self._parse_string().this

        def _parse_command(self) -> exp.Command:
            return super()._parse_command()

        def _parse_set(self, unset: bool = False, tag: bool = False) -> exp.Set | exp.Command:
            found = super()._parse_set()
            found_var: exp.EQ = found.find(exp.EQ)
            if found_var is not None:
                return exp.Set(variable=found_var.this.kind, value=found_var.expression.kind, internal="true")
            return found

        def _parse_impersonate(self) -> exp.Command:
            # clear command
            if self._match_text_seq("CANCEL"):
                return exp.Command(this="IMPERSONATE", user="", internal="true")

            # set impression command
            self._match_and_expect_text("AS", "Expected 'AS' or 'CANCEL' after 'IMPERSONATE'")
            target_user = self._parse_var_or_string()
            if target_user is None:
                self.raise_error("Expected username after 'AS'")
            return exp.Command(this="IMPERSONATE", user=target_user.name, internal="true")

        def _parse_execute(self) -> exp.Command:
            self._match_and_expect_text("AUDIT", "Expected 'AUDIT'")
            self._match_l_paren()
            audit_id = self._match_and_parse_string("Expected string input variable")
            self._match_and_expect_token(TokenType.COMMA, "Expected ',' after audit ID")
            audit_environment = self._match_and_parse_string("Expected string input variable")
            self._match_r_paren()
            self._match_and_expect_token(TokenType.ALIAS, "Expected 'AS'")

            return exp.Command(this="EXECUTE", expression=self._advance_and_consume(), kind="AUDIT", id=audit_id, environment=audit_environment, internal="true")

        def _parse_transpile(self) -> exp.Command:
            return exp.Command(this="TRANSPILE", expression=self._advance_and_consume(), internal="true")

        def _parse_reset(self) -> exp.Command:
            # self._match_and_expect_text("RESET", "Expected 'RESET'")
            found = self._parse_var_or_string()
            return exp.Command(this="RESET", variable=found.name if found is not None else "", internal="true")

        def _parse_scan(self) -> exp.Command:
            self._match_and_expect_text("TAGS", "Expected 'TAGS'")
            return exp.Command(this="SCAN", kind="TAGS", expression=self._advance_and_consume(), internal="true")

        def _parse_describe(self) -> exp.Describe | exp.Command:
            if self._match_text_seq("ACCESS"):
                return exp.Describe(this="ACCESS", expression=self._advance_and_consume(), internal="true")
            return super()._parse_describe()

        def _parse_delete(self) -> exp.Delete | exp.Command:
            return super()._parse_delete()

        def _parse_update(self) -> exp.Update | exp.Command:
            return super()._parse_update()

        def _parse_use(self) -> exp.Use | exp.Command:
            kind = self._advance_any()
            if kind is None or kind.text.upper() not in ["CATALOG", "DATABASE", "SCHEMA"]:
                self.raise_error("Expected 'CATALOG', 'DATABASE' or 'SCHEMA' after 'USE'")
            return exp.Use(kind=kind.text.upper(), target=self._advance_and_consume(), internal="true")

        def _parse_create(self) -> exp.Create | exp.Command:
            replace = False
            if super()._match_pair(TokenType.OR, TokenType.REPLACE):
                replace = True
            if super()._match_text_seq("MASKING", "RULESET"):
                return self.expression(exp.Create, replace=replace)
            elif super()._match_text_seq("TAG"):
                return exp.Create(this="TAG", name="", description="")
            elif self._match_text_seq("RESOURCE", "GROUP"):
                return exp.Create(this="RESOURCE_GROUP", kind="RESOURCE GROUP", expression=self._advance_and_consume())

            return super()._parse_create()

    class Generator(Generator):
        TRANSFORMS = {
            **Generator.TRANSFORMS,
        }

        def tag_sql(self, expression: exp.Create):
            pass


# CREATE TAG [category].[name] (WITH DESCRIPTION 'Example Tag');
# UPDATE TAG [category].[name] SET description = 'Updated Example Tag'
# DELETE TAG [category].[name]
# DESCRIBE TAG [category].[name] (DESCRIPTION | USAGE) -- returns a table with all entities that have this tag
# SHOW TAG (workspace) -- returns a table with all tags in the specified workspace or if not specified in any workspace
# APPLY TAG <Tag> TO <Entity Name>

# CREATE MASKING RULESET example_masking_ruleset AS
# UPDATE MASKING RULESET example_ruleset SET description = 'Updated Example Masking Ruleset'
# DELETE MASKING RULESET example_ruleset
# DESCRIBE MASKING RULESET example_ruleset (DESCRIPTION | USAGE) -- returns a table with all security policies that use this ruleset
# SHOW MASKING RULESET (workspace) -- returns a table with all masking rulesets in the specified workspace or if not specified in any workspace

# CREATE FILTER RULSET example_filter_ruleset AS
# UPDATE FILTER RULSET example_filter_ruleset SET description = 'Updated Example Filter Ruleset'
# DELETE FILTER RULSET example_filter_ruleset
# DESCRIBE FILTER RULSET example_filter_ruleset (DESCRIPTION | USAGE) -- returns a table with all access policies that use this ruleset
# SHOW FILTER RULSET (workspace) -- returns a table with all filter rulesets in the specified workspace or if not specified in any workspace

# CREATE POLICY CONDITION example_condition AS
# UPDATE POLICY CONDITION example_condition SET description = 'Updated Example Condition'
# DELETE POLICY CONDITION example_condition
# DESCRIBE POLICY CONDITION example_condition (DESCRIPTION | USAGE) -- returns a table with all policies that use this condition
# SHOW POLICY CONDITION (workspace) -- returns a table with all policies in the specified workspace or if not specified in any workspace

# CREATE SECURITY POLICY example_policy AS
# UPDATE SECURITY POLICY example_policy SET description = 'Updated Example Policy'
# DELETE SECURITY POLICY example_policy
# DESCRIBE SECURITY POLICY example_policy (DESCRIPTION | USAGE) -- returns a table with all access policies that use this policy
# SHOW SECURITY POLICY (workspace) -- returns a table with all policies in the specified workspace or if not specified in any workspace

# CREATE DATASET example_bundle AS
# UPDATE DATASET example_bundle SET description = 'Updated Example Bundle'
# DELETE DATASET example_bundle
# DESCRIBE DATASET example_bundle (DESCRIPTION | USAGE) -- returns a table with all access policies that use this bundle
# SHOW DATASET (workspace) -- returns a table with all data bundles in the specified workspace or if not specified in any workspace

# CREATE ACCESS POLICY example_policy_with_bundle AS
# UPDATE ACCESS POLICY example_policy_with_bundle SET description = 'Updated Example Policy with Bundle'
# DELETE ACCESS POLICY example_policy_with_bundle
# DESCRIBE ACCESS POLICY example_policy_with_bundle (DESCRIPTION | USAGE) -- returns a table with all access policies that are in use and their status
# SHOW ACCESS POLICY (workspace) -- returns a table with all access policies in the specified workspace or if not specified in any workspace

# ANALYZE ACCESS SELECT * FROM TABLE -- returns information about any security policies applied to the given query, this can be used for the split between local execution and sql flight. Should not trigger activity update

# TODO(mrhamburg): this also needs functions for handling files / things we need to intercept
#   PIPE
#   
# TODO(mrhamburg): this also need to check for statements we will not support?
