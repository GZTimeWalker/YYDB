#!/bin/bash

set -eux;

export MYSQL_BASE_DIR=/usr/local/mysql

$MYSQL_BASE_DIR/bin/mysqld --initialize --user=mysql
$MYSQL_BASE_DIR/bin/mysql_ssl_rsa_setup

sudo chown mysql:mysql -R /usr/local/mysql/data

echo "Use mysql -uroot -p to login to the mysql"
echo "Use set password for 'root'@'localhost' = ''; to remove password."
