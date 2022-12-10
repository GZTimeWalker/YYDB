#pragma once

#define Y_ENGINE_MAX_LOG_BUFFER_SIZE 256

#include <mysql/components/services/log_builtins.h> /* LogErr */
#include "mysqld_error.h"                           /* Errors */

extern REQUIRES_SERVICE_PLACEHOLDER(log_builtins);
extern REQUIRES_SERVICE_PLACEHOLDER(log_builtins_string);

extern SERVICE_TYPE(log_builtins)* log_bi;
extern SERVICE_TYPE(log_builtins_string)* log_bs;

/** @brief
 Helper function to log a message to the error log.

    @details
 The macro LogErr will be expanded to this function.
 In order to use the log_bi and log_bs services.
*/
inline void __mysql_log(int prio, const char* msg) {
    LogErr(prio, ER_LOG_PRINTF_MSG, msg);
}

namespace yydb {
    /* Logging */

    /** @brief
     Write a message to the MySQL error log.
    */
    void mysql_log_write_raw(int level, const char* msg, int len);

    /* End of logging */

    /* Lifecycle */

    /** @brief
      Initialize the YYDB core.
     */
    extern int ha_yydb_core_init();

    /** @brief
      Initialize the YYDB core.
    */
    extern int ha_yydb_core_deinit();

    /* End of lifecycle */

    /* Table */
    extern uint64_t ha_yydb_open_table(const char* name);

    extern void ha_yydb_close_table(uint64_t table_id);
    /* End of table */
}
