from setuptools import setup, find_packages

setup(
    name="sqlparser",
    description="",
    author="Menno Hamburg",
    license="AGPL-3.0",
    packages=find_packages(include=["sqlparser"]),
    setup_requires=["setuptools_scm"],
    python_requires=">=3.7",
    install_requires=["sqlglot[rs]==25.25.0"],
    extras_require={
        "dev": ["pre-commit", "ruff==0.4.3"],
    },
)
