.array_base = 0x00030000

#[location = 0]
main:
    load 32, v0
    load 16, v1
    load 0x00030000, v5

loop:
    add v1, v0, v1
    sub v0, 1, v0

    cmp v0, 0
    jgt loop

    mul v2, 4, v3
    slr v1, v3[v2]

    add v2, 1, v2
    load 32, v0
    cmp v2, 4
    jle loop

    load 0, v0
    load 0, v1
    load 0, v2

    nop
    halt
