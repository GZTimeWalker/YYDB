#!/bin/bash

set -eux;

if [[ -z "${MYSQL_SOURCE_DIR}" ]] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

cd $MYSQL_SOURCE_DIR/bld

export MTR_UNIT_TESTS=0

cmake ..                             \
-DCMAKE_BUILD_TYPE="Release"         \
-DWITH_EMBEDDED_SERVER=0             \
-DWITH_EXTRA_CHARSETS=all            \
-DWITH_MYISAM_STORAGE_ENGINE=1       \
-DWITH_INNOBASE_STORAGE_ENGINE=1     \
-DDOWNLOAD_BOOST=1 -DWITH_BOOST=../boost

make -j32 && make install

echo "Done, MySQL with YYDB is installed!"
