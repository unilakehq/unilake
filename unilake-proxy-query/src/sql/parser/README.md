Crate for parsing SQL statements from a connection, next to parsing this crate
should also make sure we can manipulate the sql statements


SELECT first_name, last_name FROM some_table

(some_table: ((name: first_name, tags: []), (name: last_name, tags: [])))