.long = 250
// comment
.float = 1.0
.char = 'c'
.unsp = 250

#[location = 0]
main:
    add v1, .char => v1
    cmp v1, .unsp // comment!
    jle 1
    jmp main
    fadd v1, v2, v1
    fcmp v1, v2
    ldr 12, v1
    halt
    load main, v1
    //LOAD .char => v1 cannot run.
