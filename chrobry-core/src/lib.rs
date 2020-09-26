#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod parser;
pub mod processor;

use crate::parser::parse;
use crate::processor::process;
use std::collections::HashMap;

pub fn generate<F>(
    content: &str,
    separator: &str,
    variables: HashMap<String, String>,
    on_import: F,
) -> Result<String, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let ast = parse(content)?;
    process(&ast, separator, variables, on_import)
}
