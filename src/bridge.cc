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

    /* Table */
    std::uint64_t ha_yydb_open_table(const char* name) {
        rust::Str name_str(name);
        return open_table(name_str);
    }

    void ha_yydb_close_table(std::uint64_t table_id) {
        close_table(table_id);
    }

    void ha_yydb_insert_row(uint64_t table_id, uint64_t key, const u_char* row, uint length) {
        insert_row(table_id, key, (const uint8_t*)row, length);
    }

    void ha_yydb_update_row(uint64_t table_id, uint64_t key, const u_char* old, const u_char* row, uint length) {
        update_row(table_id, key, (const uint8_t*)old, (const uint8_t*)row, length);
    }
    /* End of table */
}
