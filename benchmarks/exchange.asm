.LEN = 16

#[location = 0x00000000]
main:
    load 0, v0    ; x
    load 0, v1    ; y
    load 0, v2    ; min
    load data, v3 ; DATA

x_loop:
    cmp v0, .LEN
    jge end

    tfr v0, min
    tfr v0, v1

y_loop:
    add v1, 1
    cmp v1, .LEN
    jge swap

    jsr compare

    cmp s v4, 0
    jge y_loop

    tfr v1, v2
    jmp y_loop

swap:
    jsr swap
    add 1, v0
    jmp x_loop

end:
    halt

compare:
    push { v1, v2 }

    llr v3[v1] => v1
    llr v3[v2] => v2

    sub v1, v2, v4

    pop { v1, v2 }
    ret

swap:
    push { v4, v5 }

    llr v3[v0] => v4
    llr v3[v2] => v5
    slr v5 => v3[v0]
    slr v4 => v3[v2]

    pop { v4, v5 }
    ret

#[location = 0x00010000]
data:
#word?[0, 65535; 16 %10]
