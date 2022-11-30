#!/bin/bash

set -xe

if [ ! -z $MYSQL_SOURCE_DIR] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

DEST=$MYSQL_SOURCE_DIR/storage/yengine

mkdir -p $DEST

cp include/bridge.h $DEST
cp include/ha_wapper.h $DEST
cp src/handler/ha_wapper.cc $DEST

cargo build --release

cp target/release/libyengine.a $DEST

cp scripts/CMakeLists.txt $DEST

echo "Done, please run 'scripts/cmake.sh' to configure MySQL with YEngine"
