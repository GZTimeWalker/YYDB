#pragma once

#define Y_ENGINE_MAX_LOG_BUFFER_SIZE 256

#include <mysql/components/services/log_builtins.h> /* LogErr */
#include "mysqld_error.h"                           /* Errors */

extern REQUIRES_SERVICE_PLACEHOLDER(log_builtins);
extern REQUIRES_SERVICE_PLACEHOLDER(log_builtins_string);

extern SERVICE_TYPE(log_builtins) *log_bi;
extern SERVICE_TYPE(log_builtins_string) *log_bs;

inline void __LogWapper(int prio, const char* msg);

namespace yengine {
    /* Logging */
    void mysql_log_write_raw(int level, const char* msg, int len);
    /* End of logging */

    /* Lifecycle */
    extern int ha_yengine_core_init();
    /* End of lifecycle */

    /* test */
    extern void rust_test();
    extern void do_test();
    /* End of test */
}
