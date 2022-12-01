#pragma once

#include "rust/cxx.h"
#include <memory>

namespace yengine {
    /* Logging */
    extern void mysql_log_write_raw(int level, const char *msg, int len);
    void mysql_log_write(int32_t level, rust::Str msg);
    /* End of logging */

    /* Lifecycle */
    int ha_yengine_core_init();
    /* End of lifecycle */

    /* test */
    void do_test();
    void cpp_test();
    extern void put_ha_info();
    /* End of test */
}
