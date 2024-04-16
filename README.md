# SEIS: The Simple, Extensible Instruction Set

This repository contains the code necessary to assemble for and emulate the SEIS ISA.
This follows the specs laid out by UMASS' CS 535: Computer Architecture course.

## Registers

This instruction set has 16 general-purpose registers and a handful of processor-status registers.
Processor-status registers cannot be directly accessed, except when an instruction is a *register transfer* instruction or a stack operation.

## Instruction Layout

This instruction set uses 32-bit words.

The instruction set has four defined instruction classes, determined by the OPTY field of the instruction.

The OPTY field consumes the most significant 3 bits of the instruction.

| OPTY | Description                                                              |
|-----:|--------------------------------------------------------------------------|
|  000 | Control instructions                                                     |
|  001 | Integer arithmetic (including sign extensions)                           |
|  010 | Floating-point arithmetic (including conversion between ints and floats) |
|  011 | Register transfers, including between registers and memory               |

## Addressing

Pages will span 65,536 bytes. The processor has support for a stack and a "short" page, named that due to the existence of instructions using a 16-bit immediate field to access data in a designated page. Throughout the codebase, it is referred to as a "zero page" due to its parallels with the MOS Technology 6502 processor's zero page.

For this simulation, the stack will be in the second page and the short page will be in the third page. The load and store instructions both have a zero-page variant supporting a 16-bit immediate field, but the short page will not support more advanced addressing modes. The intention of the short page is to act as a scratch-space for frequently-used values and a location to store table addresses.
However, the short page may be treated like any other location in memory using any other addressing mode once the address of a short-page address is stored in a register.

The simulator will allow up to 16 pages of data, with two reserved for the stack and the short page, to be consumed by the programs run in the simulation.

## Assembly Syntax

The assembler, `seis-asm`, suppports all the instructions laid out by the [whitepaper](instruction_set.pdf), as well as the syntaxes it lists.

### Directives

A directive can use one of the following syntaxes:

```asm
#[directive]
#[directive = value]
```

Currently, the only directive available is `location`, which enables the specification of *where* data or instructions should be written to in memory.

### Constants

A constant uses the following syntax:

```asm
.name = value
```

Constants may store integers and floating-point values. They may be consumed by putting `.name` in the location of an immediate value. If it can fit in the immediate field, it will be inserted there; otherwise, an error is emitted.

### Data Blocks

Data blocks use the following syntax:

```asm
#type! {
  ; data goes here...
}
```

It is suggested to put a label preceeding the data block to enable access through the use of the `load` macro. Data blocks allow any number of elements to be written to a location in memory. `type` may be "string", "float", "byte", "short", or "word", and the data contained **must** match the type specified.

### Randomized Data Blocks

Randomized data blocks may use one of the following syntaxes:

```asm
#type?[low, high; count]
#type?[low, high; count %seed]
```

`type` must be "byte", "short", "word", or "float". `low` and `high` must match the type specified. `seed` is optional and enables deterministic output. Just like a data block, it is suggested to have a label preceeding the randomized data block to access the address using a `load` macro.

### Labels

A label uses the following syntax:

```asm
label_name:
  ; data or instructions...
```

Labels mark a location in memory. They may mark the beginning of a subroutine or loop or the location of a data block. They may be consumed using `label_name` in the place of a relative jump or immediate value in a `load` macro.

### `load` Macros

A `load` macro expands to one or two `ldr` instructions. They may use one of the following syntaxes:

```asm
load .const, Vx
load label, Vx
load <imm>, Vx
```

Constants and immediates will be expanded to a single `ldr` instruction if the higher bits are 0, or two if not. Labels will expand to two instructions due to how the assembler processes the input files.

### Instructions

Consult the docs to learn how different instructions must be written. The instructions and their syntaxes are all written in the libseis crate, in the `instruction_set` module.

## Decoding, Encoding, and Disassembling

Decoding is handled by requiring that the various enumerator and struct types in the `libseis` crate implement the `Decode` trait. Such a trait requires implementing the `Decode::decode` function. The standalone `decode` function in the `instruction_set` module requires a type implement `Decode` and simply calls `Decode::decode`.

Decoding is handled in a tree-like fashion:

 1) The type is determined.
 2) The specific instruction is determined
 3) The layout of the instruction is determined
 4) The operands are extracted

This method is reversed for the `Encode` trait (through the `Encode::encode` function) to generate the bytecode. The standalone `encode` function is provided as a convenience in the same way as `decode`.

The `libseis` library is a dependency for the entire codebase because it defines how instructions are to be laid out, decoded, and encoded.

All types implementing `Encode` and `Decode` also implement `Display`, meaning that you can also disassemble an instruction once you decode it.

## Simulation

The simulator, provided under the `seis-sim` crate, allows you to run the assembled code and look at what the processor is doing. The `gui` project is a graphical frontend for the simulator written in C#.

The simulator and the frontend communicate through piped I/O. The frontend sends a command in plaintext and the simulator returns a value in JSON (in most cases). The JSON is then parsed to determine the state of the processor in the simulator and display it to the user.

This infrastructure enabled me to write better code for the simulator by using Rust and easily design a UI for the frontend using the [Windows Presentation Foundation](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/?view=netdesktop-8.0) framework provided by Microsoft for C#.net. It uses XAML (which is not far off from HTML) to allow rapid prototyping for UI designs, which was made easier because Visual Studio has a graphical designer for XAML applications.
