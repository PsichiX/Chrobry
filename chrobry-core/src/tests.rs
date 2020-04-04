#![cfg(test)]

use crate::generate;

#[test]
fn test_parse() {
    let content = include_str!("../../resources/cpp.chrobry");
    let output = match generate(content, "\n", |_| Ok("".to_owned())) {
        Ok(output) => output,
        Err(error) => panic!("Could not generate: {}", error),
    };
    println!("=== OUTPUT:\n{}", output);
}
