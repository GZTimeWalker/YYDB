#include <cstdio>

#include "yydb/include/bridge.h"
#include "yydb/src/bridge.rs.h"

namespace yydb {
    /* Logging */

    void mysql_log_write(std::int32_t level, rust::Str msg) {
        mysql_log_write_raw(level, msg.data(), msg.size());
    }

    /* End of logging */

    /* Lifecycle */

    int ha_yydb_core_init() {
        rust_init();

        // do other stuff
        return 0;
    }

    int ha_yydb_core_deinit() {
        rust_deinit();

        // do other stuff
        return 0;
    }

    /* End of lifecycle */
}
