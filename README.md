# SEIS: The Simple, Extensible Instruction Set

This repository contains the code necessary to assemble for and emulate the SEIS ISA.
This follows the specs laid out by UMASS' CS 535: Computer Architecture course.

## Registers

This instruction set has 16 general-purpose registers and a handful of processor-status registers.
Processor-status registers cannot be directly accessed, except when an instruction is a *register transfer* instruction.

The stack registers are both 16 bits wide for this simulator.

## Instruction Layout

This instruction set uses 32-bit words. For the purposes of this document, the bit-IDs for data words and instruction words will be different: while both will assign each bit with a number between 0 and 31,

- *Data words* number bits in ascending significance.
  - LSB = 0, MSB = 31
- *Instruction words* number bits in descending significance.
  - LSB = 31, MSB = 0

The instruction set has four defined instruction classes, determined by the OPTY field of the instruction.
The OPTY field consumes the most significant 3 bits of the instruction.

| OPTY | Description                                                              |
|-----:|--------------------------------------------------------------------------|
|  000 | Control instructions                                                     |
|  001 | Integer arithmetic (including sign extensions)                           |
|  010 | Floating-point arithmetic (including conversion between ints and floats) |
|  011 | Register transfers, including between registers and memory               |

## Addressing

Pages will span 65,536 bytes. The processor will support a stack and a "zero page", known as the "short page".

For this simulation, the stack will be in the second page and the short page will be in the third page. The load and store instructions both have a zero-page variant supporting a 16-bit immediate field, but the short page will not support more advanced addressing modes. The intention of the short page is to act as a scratch-space for frequently-used values and a location to store table addresses.
However, the short page may be treated like any other location in memory using any other addressing mode once the address of a short-page address is stored in a register.
