#!/bin/bash

export MYSQL_BASE_DIR=/usr/local/mysql

sudo nohup $MYSQL_BASE_DIR/bin/mysqld_safe --user=dev 2>&1 &
