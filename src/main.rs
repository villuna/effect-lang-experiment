mod interpreter;
mod parse;

use color_eyre::{Result, eyre::Context};
use std::fs::read_to_string;

use crate::{interpreter::interpret, parse::parse};

fn main() -> Result<()> {
    let Some(filename) = std::env::args().nth(1) else {
        println!("Please pass a file to be interpreted");
        return Ok(());
    };

    let text = read_to_string(filename).context("File not found")?;
    let program = parse(&text).context("Parse error")?;

    //dbg!(&program);
    interpret(&program);

    Ok(())
}
