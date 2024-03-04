#[location = @0]
.long = 250:long
// comment
.float = 1.0
.string = "a string"
.char = 'c'

#[location = 0]
main:
    add v1, .char => v1
    jle #1
    jmp main
    halt
    //LOAD .char => v1
