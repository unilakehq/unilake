.PHONY: install install-dev install-pre-commit test unit style check docs docs-serve

install:
	pip install -e .

install-dev:
	pip install -e ".[dev]"

style:
	pre-commit run --all-files

check: style