#!/bin/bash

set -eux;

if [[ -z "${MYSQL_SOURCE_DIR}" ]] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

cd $MYSQL_SOURCE_DIR/bld

export MTR_UNIT_TESTS=0

rm -rf $MYSQL_SOURCE_DIR/bld/plugin_output_directory/ha_yengine.so

make -j32 && make install
