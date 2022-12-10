#!/bin/bash

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "UNINSTALL PLUGIN yydb" || true

mysql -uroot -e "INSTALL PLUGIN yydb SONAME 'ha_yydb.so'"

mysql -uroot -e "CREATE DATABASE IF NOT EXISTS yydb"

mysql -uroot -e "DROP TABLE IF EXISTS yydb.test"

mysql -uroot -e "CREATE TABLE IF NOT EXISTS yydb.test (id INT, name VARCHAR(255)) ENGINE=yydb"

mysql -uroot -e "INSERT INTO yydb.test (id, name) VALUES (233, 'test')"

# mysql -uroot -e "SELECT * FROM yydb.test"
