mod ast;
mod parse;

use color_eyre::{Result, eyre::Context};
use std::fs::read_to_string;

use crate::parse::parse;

fn main() -> Result<()> {
    let text = read_to_string("test.villi").context("File not found")?;
    let program = parse(&text).context("Parse error")?;

    dbg!(program);

    Ok(())
}
