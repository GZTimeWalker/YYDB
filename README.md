# YYDB

Yat another MySQL storage engine.

Database course project written in Rust.

## How does this Rust project work with MySQL?

This project is a MySQL storage engine. We only support compiling with `MYSQL_DYNAMIC_PLUGIN` enabled, which means the final binary is a shared library (`.so` on Linux) that is loaded by the MySQL server.

The compilation process is as follows:

```log
                                      MySQL source dir
+------------------+-+               ./storage/yydb
| ha_wapper.h      | |   Copy        +----------------+-+
| yydb.h           | +-----------+   | CMakeLists.txt | |
| ha_wapper.cc     | |           |   +----------------+ |
| yydb.cc          | |           +-> |  C++ sources   | +--+
+------------------+-+               +----------------+ |  |
                                 +-> |   libyydb.a    | |  |
+------------------+-+           |   +----------------+-+  |
| bridge.rs (Rust) | |           |                         |
| bridge.cc (C++)  | | cxxbridge |           Compile CXX   |
+------------------+ +-----------+        & Link Rust libs |
| lib.rs (Rust)    | |  Compile                            |
| other files...   | |                        ha_yydb.so <-+
+------------------+-+
```

You can refer to [depoly.sh](./scripts/deploy.sh) and [build.sh](./scripts/build.sh) for more details.

We need to expose some symbols to the MySQL server, such as `_mysql_plugin_declarations_`, so we need to make sure that `ha_wapper.cc` is the main source file in case the linker might delete or reorder these symbols. (which the rustc linker might do)

## How to build or develop this project?

We provide a [Dev Container](https://code.visualstudio.com/docs/remote/containers) for you to develop this project. You can use [VSCode](https://code.visualstudio.com/) to open this project and press `F1` to run `Dev Containers: Reopen in Container` to start the Dev Container.

Then you can simply run `scripts/depoly.sh && scripts/build.sh` to build the project.

## Demo

- Do something

```sql
mysql> SET binlog_format = 'STATEMENT';
Query OK, 0 rows affected (0.00 sec)

mysql> select count(*) from test;
+----------+
| count(*) |
+----------+
|    50000 |
+----------+
1 row in set (0.30 sec)

mysql> call p1; -- add 10000 rows
Query OK, 0 rows affected (13.63 sec)

mysql> select count(*) from test;
+----------+
| count(*) |
+----------+
|    60000 |
+----------+
1 row in set (0.35 sec)

mysql> select * from test order by num limit 5; -- query
+----+-----+---------------------+--------+-------------+
| id | num | date                | name   | other       |
+----+-----+---------------------+--------+-------------+
|  1 | 0.5 | 2022-12-21 11:00:34 | test_1 | hello world |
|  2 |   1 | 2022-12-21 11:00:34 | test_2 | hello world |
|  3 | 1.5 | 2022-12-21 11:00:34 | test_3 | hello world |
|  4 |   2 | 2022-12-21 11:00:34 | test_4 | hello world |
|  5 | 2.5 | 2022-12-21 11:00:34 | test_5 | hello world |
+----+-----+---------------------+--------+-------------+
5 rows in set (0.38 sec)

mysql> select * from test order by num desc limit 5;
+-------+---------+---------------------+------------+-------+
| id    | num     | date                | name       | other |
+-------+---------+---------------------+------------+-------+
| 99999 | 49999.5 | 2022-12-21 10:56:18 | test_99999 | nope  |
| 99998 |   49999 | 2022-12-21 10:56:18 | test_99998 | nope  |
| 99997 | 49998.5 | 2022-12-21 10:56:18 | test_99997 | nope  |
| 99996 |   49998 | 2022-12-21 10:56:18 | test_99996 | nope  |
| 99995 | 49997.5 | 2022-12-21 10:56:18 | test_99995 | nope  |
+-------+---------+---------------------+------------+-------+
5 rows in set (0.38 sec)

mysql> delete from test where num >= 45000; -- delete
Query OK, 10000 rows affected (0.37 sec)

mysql> select count(*) from test;
+----------+
| count(*) |
+----------+
|    50000 |
+----------+
1 row in set (0.33 sec)

mysql> update test set other="yydb!" where num < 26000; -- update
Query OK, 12000 rows affected (1.83 sec)
Rows matched: 12000  Changed: 12000  Warnings: 0

mysql> update test set other="nope!" where num > 47000;
Query OK, 5999 rows affected (0.80 sec)
Rows matched: 5999  Changed: 5999  Warnings: 0

mysql> update test set other="nope!?!" where num = 47500.5;
Query OK, 1 row affected (0.40 sec)
Rows matched: 1  Changed: 1  Warnings: 0

mysql> update test set other="meow~" where num = 47501;
Query OK, 1 row affected (0.41 sec)
Rows matched: 1  Changed: 1  Warnings: 0

mysql> select * from test where num >= 47500 order by num limit 5; -- trigger a full scan
+-------+---------+---------------------+------------+---------+
| id    | num     | date                | name       | other   |
+-------+---------+---------------------+------------+---------+
| 95000 |   47500 | 2022-12-21 10:56:11 | test_95000 | nope!   |
| 95001 | 47500.5 | 2022-12-21 10:56:11 | test_95001 | nope!?! |
| 95002 |   47501 | 2022-12-21 10:56:11 | test_95002 | meow~   |
| 95003 | 47501.5 | 2022-12-21 10:56:11 | test_95003 | nope!   |
| 95004 |   47502 | 2022-12-21 10:56:11 | test_95004 | nope!   |
+-------+---------+---------------------+------------+---------+
5 rows in set (0.40 sec)

mysql> select * from test limit 5; -- iter directly, order by the timestamp and id
+-------+---------+---------------------+------------+---------+
| id    | num     | date                | name       | other   |
+-------+---------+---------------------+------------+---------+
| 95001 | 47500.5 | 2022-12-21 10:56:11 | test_95001 | nope!?! |
| 95002 |   47501 | 2022-12-21 10:56:11 | test_95002 | meow~   |
| 99858 |   49929 | 2022-12-21 10:56:18 | test_99858 | nope!   |
| 99859 | 49929.5 | 2022-12-21 10:56:18 | test_99859 | nope!   |
| 99860 |   49930 | 2022-12-21 10:56:18 | test_99860 | nope!   |
+-------+---------+---------------------+------------+---------+
5 rows in set (0.00 sec)
```

- Storage directory

```log
$ ls -alh ./test
total 1.6M
drwxr-x--x 2 dev dev  24K Dec 21 11:13 .
drwxr-x--- 3 dev dev 4.0K Dec 21 10:54 ..
-rw-r----- 1 dev dev 1.4K Dec 21 11:13 0ffa0fab5c5e16c8.l0
-rw-r----- 1 dev dev 5.0K Dec 21 11:13 1ffa0fab5c5e1b7c.l1
-rw-r----- 1 dev dev 5.0K Dec 21 11:13 1ffa0fab5c5ef70b.l1
-rw-r----- 1 dev dev  78K Dec 21 11:13 3ffa0fab5c5e6124.l3
-rw-r----- 1 dev dev  80K Dec 21 11:06 3ffa0fab7480878a.l3
-rw-r----- 1 dev dev 307K Dec 21 11:06 4ffa0fab748e4d4d.l4
-rw-r----- 1 dev dev 463K Dec 21 10:58 5ffa0fab92d15adf.l5
-rw-r----- 1 dev dev  402 Dec 21 11:14 .cache
-rw-r----- 1 dev dev 611K Dec 21 11:14 .meta
```

## Note

This project is only a **course project**, so it is not well tested and **may have bugs**.

Many design decisions are **not well thought out**, so it may not be a good, high performance, typical implementation.

Even though we need to define a primary key, we don't implement an index, so we can't do a quick lookup by primary key. The reason for adding a primary key is simply because our storage implementation is based on key-value pairs.

To make the implementation simple, we assume that the primary keys are data that can be converted to u64.

## References

- [Chapter 23 Writing a Custom Storage Engine](https://web.archive.org/web/20200617083105/https://dev.mysql.com/doc/internals/en/custom-engine.html)
- [KipDB - Github](https://github.com/KKould/KipDB)
