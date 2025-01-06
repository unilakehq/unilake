.PHONY: install install-dev install-pre-commit test unit style check docs docs-serve

install:
	pip install -e .

install-dev:
	pip install -e ".[dev]"

style:
	pre-commit run --all-files

check: style

test:
	python -m unittest

build-wheel:
	python setup.py sdist bdist_wheel

deploy-locally:
	pip install ./dist/sqlparser-0.0.0.tar.gz