#include "yengine.h"

namespace yengine {
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

    __LogWapper(prio, msg);

    if(long_buf != nullptr) {
      delete[] long_buf;
    }
  }
}

inline void __LogWapper(int prio, const char* msg) {
  LogErr(prio, ER_LOG_PRINTF_MSG, msg);
}
