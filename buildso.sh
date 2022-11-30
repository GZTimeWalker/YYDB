#!/bin/bash

set -xe

cargo build --release

MYSQL_ARGS="-DMYSQL_DYNAMIC_PLUGIN"
INCLUDES="-I include -I include/mysql -I include/mysql/include"
GCC_ARGS="-Wall -Wextra -std=c++17 -O3 -ffunction-sections -fdata-sections -fPIC -gdwarf-4 -fno-omit-frame-pointer"

g++ $MYSQL_ARGS -shared -o target/release/libyengine.so \
$GCC_ARGS \
$INCLUDES \
src/handler/ha_wapper.cc \
/Y-Engine/target/release/libyengine.a
