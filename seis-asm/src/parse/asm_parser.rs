use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parse/asm.pest"]
pub struct AsmParser;
