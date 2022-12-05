#include "yydb.h"

namespace yydb {
    void mysql_log_write_raw(int level, const char* msg, int len) {
        int prio;
        char buf[Y_ENGINE_MAX_LOG_BUFFER_SIZE];
        char* long_buf = nullptr;

        switch(level) {
            case 1:
                prio = ERROR_LEVEL;
                break;
            case 2:
                prio = WARNING_LEVEL;
                break;
            case 3:
            case 4:
                prio = SYSTEM_LEVEL;
                break;
            default:
                prio = INFORMATION_LEVEL;
                break;
        }

        if(len > Y_ENGINE_MAX_LOG_BUFFER_SIZE - 1) {
            long_buf = new char[len + 1];
            memcpy(long_buf, msg, len);
            long_buf[len] = '\0';
            msg = long_buf;
        }
        else {
            memcpy(buf, msg, len);
            buf[len] = '\0';
            msg = buf;
        }

        __mysql_log(prio, msg);

        if(long_buf != nullptr) {
            delete[] long_buf;
        }
    }
}
