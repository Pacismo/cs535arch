; v0 -> string0
; v1 -> string1
; v2 -> result
#[location = 0x100]
strcmp:
    push { v3, v4, v5 }
    load 0, v3

strcmp_loop:
    lbr v0[v3], v4 ; load left string value
    lbr v1[v3], v5 ; load right string value

    add v3, 1, v3  ; add one to index
    sub v4, v5, v2 ; subtract character from left and right strings

    cmp v4, 0      ; exit if left is null
    jeq strcmp_end
    cmp v5, 0      ; exit if right is null
    jeq strcmp_end
    cmp v2, 0      ; continue if chars are equal
    jeq strcmp_loop

strcmp_end:
    pop { v3, v4, v5 }
    ret
