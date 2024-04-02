#[location = 0]
main:
    load 32, v0
    load 16, v1

loop:
    add v1, v0, v1
    sub v0, 1, v0

    cmp s v0, 0
    jgt loop

    add v2, 1, v2
    cmp v2, 4
    jle loop

    load 0, v0
    load 0, v1
    load 0, v2

    nop
    halt
