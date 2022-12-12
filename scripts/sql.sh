#!/bin/bash

set -eux;

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "SET GLOBAL binlog_format = 'STATEMENT';" || true

if [[ $(mysql -uroot -e "SHOW PLUGINS" | grep yydb | wc -l) -eq 0 ]] ; then
    mysql -uroot -e "INSTALL PLUGIN yydb SONAME 'ha_yydb.so'"
fi

mysql -uroot -e "CREATE DATABASE IF NOT EXISTS testdb"

mysql -uroot -e "DROP TABLE IF EXISTS testdb.test"

mysql -uroot -e "CREATE TABLE IF NOT EXISTS testdb.test (
    id INT NOT NULL,
    num DOUBLE NOT NULL DEFAULT 2.3,
    date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name VARCHAR(10) NOT NULL
) ENGINE=yydb"

mysql -uroot -e "INSERT INTO testdb.test (id, name) VALUES
    (1, 'test_1_1'),
    (4, 'test_4')"

mysql -uroot -e "INSERT INTO testdb.test (id, name) VALUES (5, 'test__8')"

mysql -uroot -e "UPDATE testdb.test SET name = 'test_??_5'"

mysql -uroot -e "SELECT * FROM testdb.test"
