#include <stdint.h>
#include <stdio.h>

const uint32_t DATA[] = {9, 4, 1, 2, 6, 4, 9, 1, 4, 1, 10, 15, 7, 14, 12, 9};
const size_t   COUNT  = sizeof(DATA) / sizeof(uint32_t);

int32_t compare(uint32_t[], uint32_t, uint32_t);
void    swap(uint32_t[], uint32_t, uint32_t);

int main()
{
    uint32_t x = 0;
    uint32_t y;
    uint32_t min;

    while (x < COUNT) {
        min = x;
        y   = x + 1;
        while (y < COUNT) {
            int32_t comp = compare(DATA, y, min);

            if (comp < 0)
                min = y;
        }
        swap(DATA, x, y);
        x = x + 1;
    }
}

int32_t compare(uint32_t array[], uint32_t idx1, uint32_t idx2)
{
    uint32_t left  = array[idx1];
    uint32_t right = array[idx2];

    return left - right;
}

void swap(uint32_t array[], uint32_t idx1, uint32_t idx2)
{
    uint32_t temp = array[idx1];
    array[idx1]   = array[idx2];
    array[idx2]   = temp;
}
