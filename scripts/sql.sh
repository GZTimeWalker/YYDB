#!/bin/bash

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "UNINSTALL PLUGIN yydb" || true

mysql -uroot -e "INSTALL PLUGIN yydb SONAME 'ha_yydb.so'"

mysql -uroot -e "CREATE DATABASE IF NOT EXISTS yydb"

mysql -uroot -e "CREATE TABLE IF NOT EXISTS yydb.test (id INT NOT NULL AUTO_INCREMENT, name VARCHAR(255), PRIMARY KEY (id)) ENGINE=yydb"

mysql -uroot -e "INSERT INTO yydb.test (name) VALUES ('test')"

mysql -uroot -e "SELECT * FROM yydb.test"
