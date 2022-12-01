#!/bin/bash

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "UNINSTALL PLUGIN yengine" || true

mysql -uroot -e "INSTALL PLUGIN yengine SONAME 'ha_yengine.so'"

mysql -uroot < scripts/test.sql
