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

## Note

This project is only a course project, so it is not well tested and may have bugs.

Many design decisions are not well considered, so it may not be a good example for you to learn from.

## References

- [Chapter 23 Writing a Custom Storage Engine](https://web.archive.org/web/20200617083105/https://dev.mysql.com/doc/internals/en/custom-engine.html)
- [KipDB - Github](https://github.com/KKould/KipDB)
