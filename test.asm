.array_base = 0x00030000
.loops = 8

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

    mul v2, 2, v4
    ssr v1, v3[v4]

    add v2, 1, v2
    load 8, v0
    cmp v2, .loops
    jle loop

    nop
    halt
