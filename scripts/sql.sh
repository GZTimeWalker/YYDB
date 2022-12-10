#!/bin/bash

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "SET GLOBAL binlog_format = 'STATEMENT';" || true

mysql -uroot -e "UNINSTALL PLUGIN yydb" || true

mysql -uroot -e "INSTALL PLUGIN yydb SONAME 'ha_yydb.so'"

mysql -uroot -e "CREATE DATABASE IF NOT EXISTS yydb"

mysql -uroot -e "DROP TABLE IF EXISTS yydb.test"

mysql -uroot -e "CREATE TABLE IF NOT EXISTS yydb.test (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
) ENGINE=yydb"

mysql -uroot -e "INSERT INTO yydb.test (id, name) VALUES
    (1, 'test__1'),
    (2, 'test__2'),
    (3, 'test__3'),
    (4, 'test__4')"

mysql -uroot -e "INSERT INTO yydb.test (name) VALUES ('test__8')"

mysql -uroot -e "UPDATE yydb.test SET name = 'test__5' WHERE id = 1"

mysql -uroot -e "SELECT * FROM yydb.test"
