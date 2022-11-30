if [ ! -z $MYSQL_SOURCE_DIR] ; then
    echo "MYSQL_SOURCE_DIR is not set"
    exit 1
fi

cd $MYSQL_SOURCE_DIR/bld

cmake ..                             \
-DCMAKE_BUILD_TYPE="Release"         \
-DWITH_EMBEDDED_SERVER=0             \
-DWITH_EXTRA_CHARSETS=all            \
-DWITH_MYISAM_STORAGE_ENGINE=1       \
-DWITH_INNOBASE_STORAGE_ENGINE=1     \
-DWITH_PARTITION_STORAGE_ENGINE=1    \
-DWITH_CSV_STORAGE_ENGINE=1          \
-DWITH_ARCHIVE_STORAGE_ENGINE=1      \
-DWITH_BLACKHOLE_STORAGE_ENGINE=1    \
-DWITH_Y_ENGINE_STORAGE_ENGINE=1     \
-DDOWNLOAD_BOOST=1 -DWITH_BOOST=../boost

echo "Done, please run 'scripts/build.sh' to build MySQL with YEngine"
