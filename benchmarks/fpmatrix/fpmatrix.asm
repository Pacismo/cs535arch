.M = 10
.N = 10
.O = 10

#[location = 0x00000000]
main:
    load left_matrix, va
    load right_matrix, vb
    load result_matrix, ve

mat_mul:
    load 0, vc

mat_mul_i_loop:
    load 0, vd

mat_mul_j_loop:
    jsr dot_product

    mul vc, .O, v2
    add v2, vd, v2
    mul v2, 4, v2

    slr vf, ve[v2]

    add vd, 1, vd
    cmp vd, .O
    jlt mat_mul_j_loop

    add vc, 1, vc

    cmp vc, .M
    jlt mat_mul_i_loop

    halt

dot_product: ; dot_product(mat1: @va, mat2: @vb, i: @vc, j: @vd) -> @vf
             ; REQUIREMENTS:
             ;  - 0 <= i < .M
             ;  - 0 <= j < .O
    push { v0, v1, v2, v3 }

    load 0, v3 ; index
    load 0, vf ; result

dot_product_loop:
    mul vc, .M, v0 ; Compute index in mat1
    add v3, v0, v0
    mul v0, 4, v0
    llr va[v0], v0 ; Load value from mat1

    mul v3, .O, v1 ; Compute index in mat2
    add vd, v1, v1
    mul v1, 4, v1
    llr vb[v1], v1 ; Load value from mat2

    fmul v0, v1, v2 ; Multiply values
    fadd v2, vf, vf ; Add to sum

    add v3, 1, v3
    cmp v3, .N
    jlt dot_product_loop

    pop { v0, v1, v2, v3 }
    ret

#[location = 0x00030000]
left_matrix: ; M * N
#float?[0.0, 500.0; 100 %15]

#[location = 0x00040000]
right_matrix: ; N * O
#float?[0, 500.0; 100 %20]

#[location = 0x00050000]
result_matrix: ; M * O
#word! { 0 }
