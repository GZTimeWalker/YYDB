if [ ! -z $MYSQL_SOURCE_DIR] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

cd $MYSQL_SOURCE_DIR/bld

make -j32 && make install
