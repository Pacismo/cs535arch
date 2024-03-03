use super::asm_parser::Rule as AsmRule;
use pest::error::Error as PestError;
use std::{io::Error as IOError, path::Path};
use std::{fmt::Display, path::PathBuf};

#[derive(Debug)]
pub struct Error {
    pub path: PathBuf,
    pub source: ErrorSource,
}

impl Error {
    pub fn new(path: &Path, source: ErrorSource) -> Self {
        Self {
            path: path.to_owned(),
            source,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error when parsing {}:\n{}", self.path.display(), self.source)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

#[derive(Debug)]
pub enum ErrorSource {
    Pest(PestError<AsmRule>),
    IO(IOError),
}

impl std::error::Error for ErrorSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorSource::Pest(p) => Some(p),
            ErrorSource::IO(i) => Some(i),
        }
    }
}

impl Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSource::Pest(pest) => {
                let renamed = pest.clone().renamed_rules(|rule| match rule {
                    AsmRule::program => "program".into(),
                    AsmRule::EOI => "end-of-input".into(),
                    AsmRule::WHITESPACE => "whitespace".into(),
                    AsmRule::comment => "comment".into(),
                    AsmRule::line => "line of assembly".into(),
                    AsmRule::constant => "constant".into(),
                    AsmRule::instruction => "instruction".into(),
                    AsmRule::directive => "directive".into(),
                    AsmRule::controlop => "control operator".into(),
                    AsmRule::integerop => "integer operator".into(),
                    AsmRule::floatop => "floating-point operator".into(),
                    AsmRule::registerop => "register operator".into(),
                    AsmRule::ident => "identity".into(),
                    AsmRule::label => "label".into(),
                    AsmRule::r#const => "constant".into(),
                    AsmRule::constref => "reference to a constant".into(),
                    AsmRule::byte => "byte".into(),
                    AsmRule::short => "short".into(),
                    AsmRule::word => "word".into(),
                    AsmRule::r#type => "type".into(),
                    AsmRule::vareg => "variable register".into(),
                    AsmRule::stareg => "stack register".into(),
                    AsmRule::lpreg => "link pointer register".into(),
                    AsmRule::pcreg => "program counter register".into(),
                    AsmRule::psreg => "processor status register".into(),
                    AsmRule::fsreg => "floating-point status register".into(),
                    AsmRule::spreg => "special-purpose register".into(),
                    AsmRule::reg => "register".into(),
                    AsmRule::sign => "sign".into(),
                    AsmRule::dec => "decimal".into(),
                    AsmRule::hex => "hexadecimal".into(),
                    AsmRule::oct => "octal".into(),
                    AsmRule::integer => "integer".into(),
                    AsmRule::uinteger => "unsigned integer".into(),
                    AsmRule::character => "character".into(),
                    AsmRule::int => "integer".into(),
                    AsmRule::float => "floating-point".into(),
                    AsmRule::string => "string".into(),
                    AsmRule::char => "character".into(),
                    AsmRule::zpgaddr => "zero-page address".into(),
                    AsmRule::zpgref => "zero-page address (reference)".into(),
                    AsmRule::halt => "HALT".into(),
                    AsmRule::nop => "NOP".into(),
                    AsmRule::jmp => "JMP".into(),
                    AsmRule::jsr => "JSR".into(),
                    AsmRule::ret => "RET".into(),
                    AsmRule::jeq => "JEQ".into(),
                    AsmRule::jne => "JNE".into(),
                    AsmRule::jgt => "JGT".into(),
                    AsmRule::jlt => "JLT".into(),
                    AsmRule::jge => "JGE".into(),
                    AsmRule::jle => "JLE".into(),
                    AsmRule::jump => "jump".into(),
                    AsmRule::absolute => "absolute jump".into(),
                    AsmRule::relative => "relative jump".into(),
                    AsmRule::add => "ADD".into(),
                    AsmRule::sub => "SUB".into(),
                    AsmRule::mul => "MUL".into(),
                    AsmRule::dvu => "DVU".into(),
                    AsmRule::dvs => "DVS".into(),
                    AsmRule::r#mod => "MOD".into(),
                    AsmRule::and => "AND".into(),
                    AsmRule::ior => "IOR".into(),
                    AsmRule::xor => "XOR".into(),
                    AsmRule::not => "NOT".into(),
                    AsmRule::sxt => "SXT".into(),
                    AsmRule::bsl => "BSL".into(),
                    AsmRule::bsr => "BSR".into(),
                    AsmRule::asr => "ASR".into(),
                    AsmRule::rol => "ROL".into(),
                    AsmRule::ror => "ROR".into(),
                    AsmRule::cmp => "CMP".into(),
                    AsmRule::tst => "TST".into(),
                    AsmRule::int_binop => "integer binary operation".into(),
                    AsmRule::int_unop => "integer unary operation".into(),
                    AsmRule::int_cmpop => "integer comparison operation".into(),
                    AsmRule::fadd => "FADD".into(),
                    AsmRule::fsub => "FSUB".into(),
                    AsmRule::fmul => "FMUL".into(),
                    AsmRule::fdiv => "FDIV".into(),
                    AsmRule::fmod => "FMOD".into(),
                    AsmRule::fcmp => "FCMP".into(),
                    AsmRule::fneg => "FNEG".into(),
                    AsmRule::frec => "FREC".into(),
                    AsmRule::itof => "ITOF".into(),
                    AsmRule::ftoi => "FTOI".into(),
                    AsmRule::fchk => "FCHK".into(),
                    AsmRule::float_cmpop => "floating-point comparison operation".into(),
                    AsmRule::float_binop => "floating-point binary operation".into(),
                    AsmRule::float_unop => "floating-point unary operation".into(),
                    AsmRule::push => "PUSH".into(),
                    AsmRule::pop => "POP".into(),
                    AsmRule::regstack => "register list".into(),
                    AsmRule::lbr => "LBR".into(),
                    AsmRule::sbr => "SBR".into(),
                    AsmRule::lsr => "LSR".into(),
                    AsmRule::ssr => "SSR".into(),
                    AsmRule::llr => "LLR".into(),
                    AsmRule::slr => "SLR".into(),
                    AsmRule::loadsrc => "load source".into(),
                    AsmRule::offsetind => "offset indirect".into(),
                    AsmRule::indexind => "indexed indirect".into(),
                    AsmRule::stackoff => "stack offset".into(),
                    AsmRule::tfr => "TFR".into(),
                    AsmRule::ldr => "LDR".into(),
                    AsmRule::load => "LOAD".into(),
                    AsmRule::zpaload => "zero-page address".into(),
                    AsmRule::immload => "immediate value".into(),
                    AsmRule::part => "short index".into(),
                    AsmRule::assign => "=> or ->".into(),
                    AsmRule::insert => "=|".into(),
                });

                write!(f, "{renamed}")
            }
            ErrorSource::IO(io) => write!(f, "{io}"),
        }
    }
}

impl From<PestError<AsmRule>> for ErrorSource {
    fn from(value: PestError<AsmRule>) -> Self {
        Self::Pest(value)
    }
}

impl From<std::io::Error> for ErrorSource {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
