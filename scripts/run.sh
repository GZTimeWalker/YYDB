#!/bin/bash

set -eux;

export MYSQL_BASE_DIR=/usr/local/mysql

sudo nohup $MYSQL_BASE_DIR/bin/mysqld_safe --user=mysql 2>&1 &
