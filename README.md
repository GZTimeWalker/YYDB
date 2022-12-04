# Y-Engine

Yat another MySQL storage engine.

Database course project written in Rust.

## How does this Rust project work with MySQL?

This project is a MySQL storage engine. We only support compiling with `MYSQL_DYNAMIC_PLUGIN` enabled, which means the final binary is a shared library (`.so` on Linux) that is loaded by the MySQL server.

The compilation process is as follows:

```log
                                      MySQL source dir
+------------------+-+               ./storage/yengine
| ha_wapper.h      | |   Copy        +----------------+-+
| yengine.h        | +-----------+   | CMakeLists.txt | |
| ha_wapper.cc     | |           |   +----------------+ |
| yengine.cc       | |           +-> |  C++ sources   | +--+
+------------------+-+               +----------------+ |  |
                                 +-> |  libyengine.a  | |  |
+------------------+-+           |   +----------------+-+  |
| bridge.rs (Rust) | |           |                         |
| bridge.cc (C++)  | | cxxbridge |           Compile CXX   |
+------------------+ +-----------+        & Link Rust libs |
| lib.rs (Rust)    | | Rust Compile                        |
| other files...   | |                     ha_yengine.so <-+
+------------------+-+
```

You can refer to [depoly.sh](./scripts/deploy.sh) and [build.sh](./scripts/build.sh) for more details.

We need to expose some symbols to the MySQL server, such as `_mysql_plugin_declarations_`, so we need to make sure that `ha_wapper.cc` is the main source file in case the linker might delete or reorder these symbols. (which the rustc linker might do)

## How to build or develop this project?

We provide a [Dev Container](https://code.visualstudio.com/docs/remote/containers) for you to develop this project. You can use [VSCode](https://code.visualstudio.com/) to open this project and press `F1` to run `Dev Containers: Reopen in Container` to start the Dev Container.

Since it takes about 10GB+ of disk space to build MySQL, the default container does not come with MySQL source code. If you need to do a full build with MySQL, please edit the `.devcontainer/docker-compose.yaml` file and change the `Dockerfile` to `Dockerfile.full` and create the container, or you can do a manual source install via `scripts/install-source.sh`.

Then you can simply run `scripts/depoly.sh && scripts/build.sh` to build the project.
