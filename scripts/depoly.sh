#!/bin/bash

set -xe

if [ ! -z $MYSQL_SOURCE_DIR] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

DEST=$MYSQL_SOURCE_DIR/storage/yengine

mkdir -p $DEST
rm -rf $DEST/*

ln -s $PWD/include/bridge.h $DEST
ln -s $PWD/include/ha_wapper.h $DEST
ln -s $PWD/src/handler/ha_wapper.cc $DEST

cargo build --release

ln -s $PWD/target/release/libyengine.a $DEST

ln -s $PWD/scripts/CMakeLists.txt $DEST

echo "Done, please run 'scripts/cmake.sh' to configure MySQL with YEngine"
