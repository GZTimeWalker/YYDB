#pragma once

#include "rust/cxx.h"
#include <memory>

namespace yengine {
    /* Logging */

    /** @brief
      Write a message to the MySQL error log.
     */
    extern void mysql_log_write_raw(int level, const char *msg, int len);

    /** @brief
      Bridge function to warp the Rust string to C++ string.
      Then call mysql_log_write_raw.
     */
    void mysql_log_write(int32_t level, rust::Str msg);

    /* End of logging */

    /* Lifecycle */

    /** @brief
      Initialize the YEngine core.
     */
    int ha_yengine_core_init();

    /** @brief
    Initialize the YEngine core.
    */
    int ha_yengine_core_deinit();

    /* End of lifecycle */
}
