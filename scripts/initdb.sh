#!/bin/bash

set -eux;

export MYSQL_BASE_DIR=/usr/local/mysql

$MYSQL_BASE_DIR/bin/mysqld --initialize --user=mysql
$MYSQL_BASE_DIR/bin/mysql_ssl_rsa_setup
