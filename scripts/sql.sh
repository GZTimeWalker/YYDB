#!/bin/bash

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "SET GLOBAL binlog_format = 'STATEMENT';" || true

mysql -uroot -e "UNINSTALL PLUGIN yydb" || true

mysql -uroot -e "INSTALL PLUGIN yydb SONAME 'ha_yydb.so'"

mysql -uroot -e "CREATE DATABASE IF NOT EXISTS testdb"

mysql -uroot -e "DROP TABLE IF EXISTS testdb.test"

mysql -uroot -e "CREATE TABLE IF NOT EXISTS testdb.test (
    id INT NOT NULL,
    name VARCHAR(255) NOT NULL
) ENGINE=yydb"

mysql -uroot -e "INSERT INTO testdb.test (id, name) VALUES
    (1, 'test__1'),
    (4, 'test__4')"

mysql -uroot -e "INSERT INTO testdb.test (id, name) VALUES (5, 'test__8')"

mysql -uroot -e "UPDATE testdb.test SET name = 'test__5'"

mysql -uroot -e "SELECT * FROM testdb.test"
