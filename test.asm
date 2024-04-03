.array_base = 0x00030000
.loops = 4

#[location = 0]
main:
    load 8, v0
    load 16, v1
    load .array_base, v3

loop:
    add v1, v0, v1
    sub v0, 1, v0

    cmp v0, 0
    jgt loop

    mul v2, 4, v4
    slr v1, v3[v4]

    add v2, 1, v2
    load 8, v0
    cmp v2, .loops
    jle loop

    nop
    halt
