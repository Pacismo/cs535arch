#[location = @0]
.long = 250:long
// comment
.float = 1.0
.string = "a string"
.char = 'c'
.unsp = 250

#[location = 0]
main:
    add v1, .char => v1
    cmp v1, .unsp // comment!
    jle 1
    jmp main
    fadd v1, v2
    fcmp v1, v2
    ldr 12, v1
    halt
    //LOAD .char => v1 cannot run.