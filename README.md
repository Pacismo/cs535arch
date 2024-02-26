# SEIS: The Simple, Extensible Instruction Set

This repository contains the code necessary to assemble for and emulate the SEIS ISA.
This follows the specs laid out by UMASS' CS 535: Computer Architecture course.

## Registers

This instruction set has 16 general-purpose registers and a handful of processor-status registers.
Processor-status registers cannot be directly accessed, except when an instruction is a *register transfer* instruction.

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
