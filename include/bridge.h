#pragma once

#include "rust/cxx.h"
#include <memory>

namespace yydb {
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
      Initialize the YYDB core.
     */
    int ha_yydb_core_init();

    /** @brief
    Initialize the YYDB core.
    */
    int ha_yydb_core_deinit();

    /* End of lifecycle */

    /* Table */
    uint64_t ha_yydb_open_table(const char* name);

    void ha_yydb_close_table(uint64_t table_id);

    void ha_yydb_insert_row(uint64_t table_id, uint64_t key, const u_char* row, uint length);

    void ha_yydb_update_row(uint64_t table_id, uint64_t key, const u_char* old, const u_char* row, uint length);
    /* End of table */
}
