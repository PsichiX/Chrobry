#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod parser;
pub mod processor;

#[cfg(test)]
mod tests;

use crate::parser::parse;
use crate::processor::process;

pub fn generate<F>(content: &str, separator: &str, on_import: F) -> Result<String, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let ast = parse(content)?;
    process(&ast, separator, on_import)
}
