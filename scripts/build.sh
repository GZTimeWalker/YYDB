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

# check if /usr/local/mysql/bin/mysql not exists and not NO_INSTALL is set
if [[ ! -f "/usr/local/mysql/bin/mysql" ]] && [[ ! -n "${NO_INSTALL+x}" ]] ; then
    sudo chown -R dev:dev /usr/local/mysql
    make install
    echo "Done, MySQL is installed!"
else
    # if /usr/local/mysql/lib/plugin exists,
    # copy the built ha_yydb.so to /usr/local/mysql/lib/plugin
    if [[ -d "/usr/local/mysql/lib/plugin" ]] ; then
        cp plugin_output_directory/ha_yydb.so /usr/local/mysql/lib/plugin
        echo "Done, ha_yydb.so is installed!"
    fi

    echo "Done, MySQL is built!"
fi
