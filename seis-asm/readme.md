# ASSEMBLER

This assembler has a number of features:

- First, it uses [Pest](https://pest.rs) to parse inputs
- Second, inputs may exist accross multiple files
- Third, it has directives (`#[<DIRECTIVE>[=<VALUE>]]`), whose identities are case-insensitive.
  - These directives enable having "public" constants and labels

## Directives

```rs
#[location = LOCATION]
```

Sets the starting address of the sequence. `LOCATION` must be a valid integer.
To specify a location in the "zero-page", use `#[location = @OFFSET]`.

```rs
#[export]
```

Exports the following label or constant, making it publicly visible.

## Constants

A constant is defined as such:

```asm
.key = value
```

If the type is an *integer*, you may specify the width using the syntax `:byte`, `:short`, or `:word`/`:long`, where the lengths are 1, 2, and 4 bytes, respectively.

To load a constant, the suggested method is as such:

```asm
LOAD .key => Vx // Load the value directly
LOAD &key => Vx // Load the address of the value
```

They will expand to the appropriate sequence of instructions to perform the operation.
The former version loads the value directly, using the appropriate `LDR` instructions.
The latter will load the address of the value, using the appropriate `LDR` instructions.

A constant whose reference is never loaded may be optimized away.

```asm
#[location=@0]
  .X = 32 // A constant
  .Y = 16:byte // Another constant
#[location=0]
main:
  LLR .X => V1
  LBR .Y => V2
  SXT byte V2
  ADD V1, V2 => V3
  // V3 should contain 48
```
