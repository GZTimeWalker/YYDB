#include "yengine/include/bridge.h"
#include "yengine/src/bridge.rs.h"

void do_test() {
    int a = 1, b = 2;
    printf("%d + %d = %d -- from cpp\n", a, b, a + b);
    put_ha_info();
}
