#include <stdio.h>
#include <stdlib.h>

#define MAX_SIZE 16

/* 블록 주석 */
typedef struct Point {
    int x;
    int y;
} Point;

static int add(int a, int b) {
    return a + b;
}

int main(int argc, char **argv) {
    // 한 줄 주석
    Point p = {1, 2};
    const char *msg = "안녕, C";
    for (int i = 0; i < MAX_SIZE; i++) {
        if (i % 2 == 0) {
            printf("%s %d\n", msg, add(p.x, i));
        }
    }
    double ratio = 3.14;
    unsigned long big = 0xFFUL;
    return ratio > 1.0 ? 0 : 1;
}
