.M = 10
.N = 10
.O = 10

#[location = 0x00000000]
main:
    load left_matrix, va
    load right_matrix, vb
    load result_matrix, ve
    load 0, v0

main_i_loop:
    load 0, v1
main_j_loop:
    jsr mat_mul

    mul v1, .O, v2
    add v2, v0, v2
    mul v2, 4, v2

    slr vf, ve[v2]

    add v1, 1, v1
    cmp v1, .O
    jlt main_j_loop

    add v0, 1, v0

    cmp v0, .M
    jlt main_i_loop

    halt

mat_mul: ; mat_mul(mat1: @va, mat2: @vb, i: @vc, j: @vd) -> @vf
         ; REQUIREMENTS:
         ;  - 0 <= i < .M
         ;  - 0 <= j < .O
    push { v0, v1, v2, v3 }

    load 0, v3 ; index
    load 0, vf ; result

mat_mul_loop:
    mul v3, .M, v0 ; Compute index in mat1
    add vc, v0, v0
    mul v0, 4, v0

    mul v3, .O, v1 ; Compute index in mat2
    add vd, v1, v1
    mul v1, 4, v1

    llr va[v0], v0 ; Read values
    llr vb[v1], v1

    mul v0, v1, v2 ; Multiply values
    add v2, vf, vf ; Add to sum

    add v3, 1, v3
    cmp v4, .N
    jlt mat_mul_loop

    pop { v0, v1, v2, v3 }
    ret

#[location = 0x00030000]
left_matrix: ; M * N
#word! {
     1,  2,  3,  4,  5,  6,  7,  8,  9,  10,
    11, 12, 13, 14, 15, 16, 17, 18, 19,  20,
    21, 22, 23, 24, 25, 26, 27, 28, 29,  30,
    31, 32, 33, 34, 35, 36, 37, 38, 39,  40,
    41, 42, 44, 44, 45, 46, 47, 48, 49,  50,
    51, 52, 55, 55, 55, 56, 57, 58, 59,  60,
    61, 62, 63, 64, 65, 66, 67, 68, 69,  70,
    71, 72, 73, 74, 75, 76, 77, 78, 79,  80,
    81, 82, 83, 84, 85, 86, 87, 88, 89,  90,
    91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
}

right_matrix: ; N * O
#word! {
     1,  2,  3,  4,  5,  6,  7,  8,  9,  10,
    11, 12, 13, 14, 15, 16, 17, 18, 19,  20,
    21, 22, 23, 24, 25, 26, 27, 28, 29,  30,
    31, 32, 33, 34, 35, 36, 37, 38, 39,  40,
    41, 42, 44, 44, 45, 46, 47, 48, 49,  50,
    51, 52, 55, 55, 55, 56, 57, 58, 59,  60,
    61, 62, 63, 64, 65, 66, 67, 68, 69,  70,
    71, 72, 73, 74, 75, 76, 77, 78, 79,  80,
    81, 82, 83, 84, 85, 86, 87, 88, 89,  90,
    91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
}

#[location = 0x00040000]
result_matrix: ; M * O
#word! { 0 }
