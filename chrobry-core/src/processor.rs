use crate::ast::*;
use regex::{Captures, Regex};
use std::collections::HashMap;

enum Context {
    None,
    Struct(String),
    Enum(String),
}

pub fn process<F>(
    ast: &Ast,
    separator: &str,
    variables: HashMap<String, String>,
    _on_import: F,
) -> Result<String, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let impls = get_impl_targets(ast);
    validate_type_impls(ast, &impls)?;
    let mut output = String::default();
    for code in &ast.injects {
        process_code(&Context::None, code, ast, &variables, &mut output)?;
        output.push_str(separator);
    }
    for external in &ast.externs {
        process_extern(external, ast, separator, &mut output)?;
    }
    for enum_ in &ast.enums {
        process_enum(enum_, ast, separator, &mut output)?;
    }
    for struct_ in &ast.structs {
        process_struct(struct_, ast, separator, &mut output)?;
    }
    for replace in &ast.replacements {
        output = process_replacement(replace, &output, ast, &variables);
    }
    Ok(output)
}

fn get_impl_targets(ast: &Ast) -> Vec<(String, AstImplementationTarget)> {
    ast.implementations
        .iter()
        .map(|i| (i.name.to_owned(), i.target.clone()))
        .collect::<Vec<_>>()
}

fn validate_type_impls(
    ast: &Ast,
    impl_targets: &Vec<(String, AstImplementationTarget)>,
) -> Result<(), String> {
    for external in &ast.externs {
        for type_ in &external.types {
            for (implementation, _) in &external.implementations {
                if !impl_targets.iter().any(|(n, _)| implementation == n) {
                    return Err(format!(
                        "Trying to apply non-existing trait `{}` for external type `{}`",
                        implementation, type_
                    ));
                }
            }
        }
    }
    for struct_ in &ast.structs {
        for (tag, _) in &struct_.tags {
            if !impl_targets
                .iter()
                .any(|(n, t)| tag == n && t.is_valid(AstImplementationTarget::Struct))
            {
                return Err(format!(
                    "Trying to apply non-existing or non-struct trait `{}` for struct `{}`",
                    tag, struct_.name
                ));
            }
        }
    }
    for enum_ in &ast.enums {
        for (tag, _) in &enum_.tags {
            if !impl_targets
                .iter()
                .any(|(n, t)| tag == n && t.is_valid(AstImplementationTarget::Enum))
            {
                return Err(format!(
                    "Trying to apply non-existing or non-enum trait `{}` for enum `{}`",
                    tag, enum_.name
                ));
            }
        }
    }
    Ok(())
}

fn process_code(
    context: &Context,
    code: &AstCode,
    ast: &Ast,
    variables: &HashMap<String, String>,
    output: &mut String,
) -> Result<(), String> {
    for chunk in &code.0 {
        match chunk {
            AstCodeChunk::Content(content) => output.push_str(&content),
            AstCodeChunk::Variable(variable) => {
                if let Some(found) = variables.get(variable) {
                    output.push_str(found);
                } else {
                    return Err(format!(
                        "Trying to place non-existing variable `{}`",
                        variable
                    ));
                }
            }
            AstCodeChunk::For(for_) => process_code_for(context, for_, ast, variables, output)?,
            AstCodeChunk::None => {}
        }
    }
    Ok(())
}

fn process_code_for(
    context: &Context,
    code: &AstCodeFor,
    ast: &Ast,
    variables: &HashMap<String, String>,
    output: &mut String,
) -> Result<(), String> {
    if code.variables.is_empty() {
        unreachable!();
    }
    let iterables = get_container_iterables(context, &code.container, ast, variables)?;
    let count = iterables.len() / code.variables.len();
    for i in 0..count {
        let start = i * code.variables.len();
        let end = start + code.variables.len();
        let values = &iterables[start..end];
        let mut variables = variables.clone();
        for (name, value) in code.variables.iter().zip(values.iter()) {
            variables.insert(name.to_owned(), value.to_owned());
        }
        process_code(context, &code.code, ast, &variables, output)?;
    }
    Ok(())
}

fn get_container_iterables(
    context: &Context,
    container: &AstIn,
    ast: &Ast,
    variables: &HashMap<String, String>,
) -> Result<Vec<String>, String> {
    match container {
        AstIn::Fields => match context {
            Context::Struct(name) => {
                let s = ast.structs.iter().find(|s| &s.name == name).unwrap();
                Ok(s.fields
                    .iter()
                    .flat_map(|(n, t)| vec![n.to_owned(), t.to_string()])
                    .collect::<Vec<_>>())
            }
            Context::Enum(name) => {
                let e = ast.enums.iter().find(|e| &e.name == name).unwrap();
                Ok(e.fields.clone())
            }
            Context::None => Err("Trying to iterate over fields of no context".to_owned()),
        },
        AstIn::Variable(variable) => {
            if let Some(found) = variables.get(variable) {
                Ok(found.split("|").map(str::to_owned).collect::<Vec<_>>())
            } else {
                Err(format!(
                    "Trying to iterate over non-existing variable `{}`",
                    variable
                ))
            }
        }
        AstIn::None => Err("There is no container specified to iterate over".to_owned()),
    }
}

fn process_replacement(
    replace: &AstReplace,
    input: &str,
    ast: &Ast,
    variables: &HashMap<String, String>,
) -> String {
    let pattern = Regex::new(&replace.pattern).expect("Could not parse replacement pattern");
    pattern
        .replace_all(input, |captures: &Captures| {
            let mut variables = variables.clone();
            for i in 0..captures.len() {
                if let Some(capture) = captures.get(i) {
                    variables.insert(format!("_{}", i), capture.as_str().to_owned());
                }
            }
            let mut output = String::new();
            process_code(
                &Context::None,
                &replace.template,
                ast,
                &variables,
                &mut output,
            )
            .expect("Could not process replacement template code");
            output
        })
        .to_owned()
        .into()
}

fn process_extern(
    external: &AstExtern,
    ast: &Ast,
    separator: &str,
    output: &mut String,
) -> Result<(), String> {
    for type_ in &external.types {
        let mut variables = HashMap::new();
        variables.insert("TYPENAME".to_owned(), type_.to_owned());
        for (_, code) in &external.implementations {
            process_code(&Context::None, code, ast, &variables, output)?;
            output.push_str(separator);
        }
    }
    Ok(())
}

fn process_enum(
    enum_: &AstEnum,
    ast: &Ast,
    separator: &str,
    output: &mut String,
) -> Result<(), String> {
    let context = Context::Enum(enum_.name.to_owned());
    for (name, params) in &enum_.tags {
        let mut variables = HashMap::new();
        variables.insert("TYPENAME".to_owned(), enum_.name.to_owned());
        for (key, value) in params {
            variables.insert(key.to_owned(), value.to_owned());
        }
        let trait_ = ast
            .implementations
            .iter()
            .find(|i| &i.name == name && i.target == AstImplementationTarget::Enum)
            .unwrap();
        // TODO: where rules validation.
        process_code(&context, &trait_.code, ast, &variables, output)?;
        output.push_str(separator);
    }
    Ok(())
}

fn process_struct(
    struct_: &AstStruct,
    ast: &Ast,
    separator: &str,
    output: &mut String,
) -> Result<(), String> {
    let context = Context::Struct(struct_.name.to_owned());
    for (name, params) in &struct_.tags {
        let mut variables = HashMap::new();
        variables.insert("TYPENAME".to_owned(), struct_.name.to_owned());
        for (key, value) in params {
            variables.insert(key.to_owned(), value.to_owned());
        }
        let trait_ = ast
            .implementations
            .iter()
            .find(|i| &i.name == name && i.target == AstImplementationTarget::Struct)
            .unwrap();
        // TODO: where rules validation.
        process_code(&context, &trait_.code, ast, &variables, output)?;
        output.push_str(separator);
    }
    Ok(())
}
