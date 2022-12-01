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

DEST=$MYSQL_SOURCE_DIR/storage/yengine

mkdir -p $DEST
rm -rf $DEST/*

rm $MYSQL_SOURCE_DIR/bld/plugin_output_directory/ha_yengine.so || true
rm $MYSQL_SOURCE_DIR/bld/storage/yengine/libha_yengine.a || true
rm $MYSQL_SOURCE_DIR/bld/storage/yengine/CMakeFiles/yengine.dir/*.o || true
rm /usr/local/mysql/lib/plugin/ha_yengine.so || true

ln -s $WORK_DIR/include/*.h $DEST
ln -s $WORK_DIR/src/handler/*.cc $DEST

cargo build --release --verbose

ln -s $WORK_DIR/target/release/libyengine.a $DEST
ln -s $WORK_DIR/scripts/CMakeLists.txt $DEST

echo "Done, please run 'scripts/cmake.sh' to configure MySQL with YEngine"
