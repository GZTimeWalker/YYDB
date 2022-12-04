#include <cstdio>

#include "yengine/include/bridge.h"
#include "yengine/src/bridge.rs.h"

namespace yengine {
    /* Logging */

    void mysql_log_write(std::int32_t level, rust::Str msg) {
        mysql_log_write_raw(level, msg.data(), msg.size());
    }

    /* End of logging */

    /* Lifecycle */

    int ha_yengine_core_init() {
        rust_init();

        // do other stuff
        return 0;
    }

    int ha_yengine_core_deinit() {
        rust_deinit();

        // do other stuff
        return 0;
    }

    /* End of lifecycle */
}
