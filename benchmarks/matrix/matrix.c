#include <stdio.h>

const unsigned M = 10;
const unsigned N = 10;
const unsigned O = 10;

unsigned MATRIX_1[M * N] = {
    1,  2,  3,  4,  5,  6,  7,  8,  9,  10,  //
    11, 12, 13, 14, 15, 16, 17, 18, 19, 20,  //
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30,  //
    31, 32, 33, 34, 35, 36, 37, 38, 39, 40,  //
    41, 42, 44, 44, 45, 46, 47, 48, 49, 50,  //
    51, 52, 55, 55, 55, 56, 57, 58, 59, 60,  //
    61, 62, 63, 64, 65, 66, 67, 68, 69, 70,  //
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80,  //
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90,  //
    91, 92, 93, 94, 95, 96, 97, 98, 99, 100, //
};

unsigned MATRIX_2[N * O] = {
    1,  2,  3,  4,  5,  6,  7,  8,  9,  10,  //
    11, 12, 13, 14, 15, 16, 17, 18, 19, 20,  //
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30,  //
    31, 32, 33, 34, 35, 36, 37, 38, 39, 40,  //
    41, 42, 44, 44, 45, 46, 47, 48, 49, 50,  //
    51, 52, 55, 55, 55, 56, 57, 58, 59, 60,  //
    61, 62, 63, 64, 65, 66, 67, 68, 69, 70,  //
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80,  //
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90,  //
    91, 92, 93, 94, 95, 96, 97, 98, 99, 100, //
};

unsigned RESULT_MATRIX[M * O];

unsigned dot_product(unsigned[], unsigned[], unsigned, unsigned);

int main()
{
    unsigned i = 0;
    unsigned j;

    while (i < M) {
        j = 0;
        while (j < O) {
            //           [x,      y]
            //     result[i,      j] = dot_product(mat1, mat2, i, j)
            RESULT_MATRIX[j + i * O] = dot_product(MATRIX_1, MATRIX_2, i, j);

            j = j + 1;
        }
        i = i + 1;
    }

    for (j = 0; j < O; ++j) {
        for (i = 0; i < M; ++i)
            printf("%8u ", RESULT_MATRIX[j * M + i]);
        printf("\n");
    }
    printf("\n");
    for (j = 0; j < O; ++j) {
        for (i = 0; i < M; ++i)
            printf("%8x ", RESULT_MATRIX[j * M + i]);
        printf("\n");
    }
}

unsigned dot_product(unsigned mat1[], unsigned mat2[], unsigned i, unsigned j)
{
    unsigned sum = 0;
    unsigned idx = 0;

    while (idx < N) {
        //         [  x,      y]       [x,        y]
        //     mat1[idx,      i] * mat2[j,      idx]
        sum += mat1[idx + i * M] * mat2[j + idx * O];
        idx = idx + 1;
    }

    return sum;
}
