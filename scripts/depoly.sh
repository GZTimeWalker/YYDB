#!/bin/bash

set -xe

if [[ -z "${MYSQL_SOURCE_DIR}" ]] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

sudo killall mysqld || true

WORK_DIR=$PWD
# if 'scripts' in $PWD, use $PWD/.. instead
if [[ $PWD =~ scripts$ ]] ; then
    WORK_DIR=$PWD/..
fi

DEST=$MYSQL_SOURCE_DIR/storage/yydb

mkdir -p $DEST
rm -rf $DEST/*

rm $MYSQL_SOURCE_DIR/bld/plugin_output_directory/ha_yydb.so || true
rm $MYSQL_SOURCE_DIR/bld/storage/yydb/libha_yydb.a || true
rm $MYSQL_SOURCE_DIR/bld/storage/yydb/CMakeFiles/yydb.dir/*.o || true
rm /usr/local/mysql/lib/plugin/ha_yydb.so || true

if [[ -z "${NO_SOFT_LINK}" ]] ; then
    COPY_CMD="ln -s"
else
    COPY_CMD="cp -f"
fi

$COPY_CMD $WORK_DIR/include/*.h $DEST
$COPY_CMD $WORK_DIR/src/handler/*.cc $DEST

cargo build --release

$COPY_CMD $WORK_DIR/target/release/libyydb.a $DEST
$COPY_CMD $WORK_DIR/scripts/CMakeLists.txt $DEST

mkdir -p $MYSQL_SOURCE_DIR/bld

echo "Done, please run 'scripts/build.sh' to build MySQL with YYDB"
