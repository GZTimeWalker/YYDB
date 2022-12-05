#!/bin/bash

set -eux;

if [[ ! -n "${MYSQL_SOURCE_DIR+x}" ]] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

cd $MYSQL_SOURCE_DIR/bld

cmake ..                             \
-DCMAKE_BUILD_TYPE="Release"         \
-DWITH_EMBEDDED_SERVER=0             \
-DWITH_INNOBASE_STORAGE_ENGINE=1     \
-DDOWNLOAD_BOOST=1 -DWITH_BOOST=../boost

if [[ $(nproc) -gt 32 ]] ; then
    make -j32
else
    make -j$(nproc)
fi

if [[ ! -n "${NO_INSTALL+x}" ]] ; then
    make install
    echo "Done, MySQL with YYDB is installed!"
else
    echo "Done, MySQL with YYDB is built!"
fi
