from dataclasses import dataclass
from sqlglot import exp, tokens, Parser, Generator
from sqlglot.dialects.dialect import Dialect
from sqlglot.tokens import Tokenizer, TokenType


@dataclass
class CreateTag:
    name: str


@dataclass
class UpdateTag:
    name: str


@dataclass
class DeleteTag:
    name: str


@dataclass
class DescribeTag:
    name: str
    pass


@dataclass
class ShowTag:
    pass


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
            "DATA": TokenType.DEFAULT,
            "BUNDLE": TokenType.DEFAULT,
            "ACCESS": TokenType.DEFAULT,
            "USAGE": TokenType.DEFAULT,
        }

        COMMANDS = {*tokens.Tokenizer.COMMANDS, TokenType.END}

    class Parser(Parser):
        STATEMENT_PARSERS = {
            **Parser.STATEMENT_PARSERS,
            TokenType.CREATE: lambda self: self._parse_create(),
            TokenType.UPDATE: lambda self: self._parse_update(),
            TokenType.DELETE: lambda self: self._parse_delete(),
            TokenType.DESCRIBE: lambda self: self._parse_describe(),
        }

        def _parse_describe(self) -> exp.Describe | exp.Command:
            print("Parsing DESCRIBE statement")
            return super()._parse_describe()

        def _parse_delete(self) -> exp.Delete | exp.Command:
            print("Parsing DELETE statement")
            return super()._parse_delete()

        def _parse_update(self) -> exp.Update | exp.Command:
            print("Parsing UPDATE statement")
            return super()._parse_update()

        def _parse_create(self) -> exp.Create | exp.Command:
            replace = False
            if super()._match_pair(TokenType.OR, TokenType.REPLACE):
                replace = True
            if super()._match_text_seq("MASKING", "RULESET"):
                print("Masking Ruleset")
                return self.expression(exp.Create, replace=replace)

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

# CREATE DATA BUNDLE example_bundle AS
# UPDATE DATA BUNDLE example_bundle SET description = 'Updated Example Bundle'
# DELETE DATA BUNDLE example_bundle
# DESCRIBE DATA BUNDLE example_bundle (DESCRIPTION | USAGE) -- returns a table with all access policies that use this bundle
# SHOW DATA BUNDLE (workspace) -- returns a table with all data bundles in the specified workspace or if not specified in any workspace

# CREATE ACCESS POLICY example_policy_with_bundle AS
# UPDATE ACCESS POLICY example_policy_with_bundle SET description = 'Updated Example Policy with Bundle'
# DELETE ACCESS POLICY example_policy_with_bundle
# DESCRIBE ACCESS POLICY example_policy_with_bundle (DESCRIPTION | USAGE) -- returns a table with all access policies that are in use and their status
# SHOW ACCESS POLICY (workspace) -- returns a table with all access policies in the specified workspace or if not specified in any workspace

# TODO(mrhamburg): this also needs functions for handling files
# TODO(mrhamburg): this also needs to check for statements we will not support?