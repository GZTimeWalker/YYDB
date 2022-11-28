#include "yengine_bridge/include/ha_wapper.hpp"
#include "yengine_bridge/src/bridge.rs.h"

void do_test() {
    int a = 1;
    int b = 2;

    printf("%d + %d = %d -- from cpp", a, b, a + b);
}
