#!/bin/bash

# ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';

mysql -uroot -e "UNINSTALL PLUGIN yydb" || true

mysql -uroot -e "INSTALL PLUGIN yydb SONAME 'ha_yydb.so'"
