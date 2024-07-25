pub mod linker;
pub mod parse;
#[cfg(test)]
mod test;

use linker::link_symbols;
use parse::tokenize;
use std::{error::Error, io::Cursor, path::Path};

pub fn compile(data: &str, path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let tokens = tokenize(data, path)?;
    let linked = link_symbols(tokens)?;

    let mut result = vec![];

    linked.write(Cursor::new(&mut result))?;

    Ok(result)
}
