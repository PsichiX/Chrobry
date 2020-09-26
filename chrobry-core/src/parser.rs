use crate::ast::*;
use pest::{iterators::Pair, Parser};
use std::collections::HashMap;

pub fn parse(content: &str) -> Result<Ast, String> {
    let program = match AstParser::parse(Rule::program, content) {
        Ok(mut pairs) => pairs.next().unwrap(),
        Err(error) => return Err(format!("{:#?}", error)),
    };
    let mut ast = Ast::default();
    for pair in program.into_inner() {
        match pair.as_rule() {
            Rule::import_elm => ast.imports.push(parse_import(pair)),
            Rule::inject_elm => ast.injects.push(parse_inject(pair)),
            Rule::replace_elm => ast.replacements.push(parse_replace(pair)),
            Rule::extern_elm => ast.externs.push(parse_extern(pair)),
            Rule::struct_elm => ast.structs.push(parse_struct(pair)),
            Rule::enum_elm => ast.enums.push(parse_enum(pair)),
            Rule::impl_elm => ast.implementations.push(parse_implementation(pair)),
            Rule::EOI => {}
            _ => panic!("{:?}", pair.as_rule()),
        }
    }
    Ok(ast)
}

fn parse_import(pair: Pair<Rule>) -> String {
    parse_string(pair.into_inner().next().unwrap())
}

fn parse_inject(pair: Pair<Rule>) -> AstCode {
    parse_code(pair.into_inner().next().unwrap())
}

fn parse_replace(pair: Pair<Rule>) -> AstReplace {
    let mut pairs = pair.into_inner();
    let pattern = parse_string(pairs.next().unwrap()).replace("\\\\", "\\");
    let template = parse_code(pairs.next().unwrap());
    AstReplace { pattern, template }
}

fn parse_extern(pair: Pair<Rule>) -> AstExtern {
    let mut pairs = pair.into_inner();
    let types = parse_extern_types(pairs.next().unwrap());
    let implementations = parse_extern_implementations(pairs.next().unwrap());
    AstExtern {
        types,
        implementations,
    }
}

fn parse_extern_types(pair: Pair<Rule>) -> Vec<String> {
    pair.into_inner().map(parse_string).collect::<Vec<_>>()
}

fn parse_extern_implementations(pair: Pair<Rule>) -> Vec<(String, AstCode)> {
    pair.into_inner()
        .map(parse_extern_implementation)
        .collect::<Vec<_>>()
}

fn parse_extern_implementation(pair: Pair<Rule>) -> (String, AstCode) {
    let mut pairs = pair.into_inner();
    let identifier = parse_identifier(pairs.next().unwrap());
    let code = parse_code(pairs.next().unwrap());
    (identifier, code)
}

fn parse_struct(pair: Pair<Rule>) -> AstStruct {
    let mut result = AstStruct::default();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::tags => result.tags = parse_tags(pair),
            Rule::identifier => result.name = parse_identifier(pair),
            Rule::fields => result.fields = parse_struct_fields(pair),
            _ => panic!("{:?}", pair.as_rule()),
        }
    }
    result
}

fn parse_struct_fields(pair: Pair<Rule>) -> Vec<(String, AstType)> {
    pair.into_inner()
        .map(parse_struct_field)
        .collect::<Vec<_>>()
}

fn parse_struct_field(pair: Pair<Rule>) -> (String, AstType) {
    let mut pairs = pair.into_inner();
    let identifier = parse_identifier(pairs.next().unwrap());
    let type_ = parse_type(pairs.next().unwrap());
    (identifier, type_)
}

fn parse_enum(pair: Pair<Rule>) -> AstEnum {
    let mut result = AstEnum::default();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::tags => result.tags = parse_tags(pair),
            Rule::identifier => result.name = parse_identifier(pair),
            Rule::enum_fields => result.fields = parse_enum_fields(pair),
            _ => panic!("{:?}", pair.as_rule()),
        }
    }
    result
}

fn parse_enum_fields(pair: Pair<Rule>) -> Vec<String> {
    pair.into_inner().map(parse_identifier).collect::<Vec<_>>()
}

fn parse_implementation(pair: Pair<Rule>) -> AstImplementation {
    let mut result = AstImplementation::default();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::impl_target => result.target = parse_implementation_target(pair),
            Rule::identifier => result.name = parse_identifier(pair),
            Rule::where_rules => result.where_rules = parse_where_rules(pair),
            Rule::code => result.code = parse_code(pair),
            _ => panic!("{:?}", pair.as_rule()),
        }
    }
    result
}

fn parse_implementation_target(pair: Pair<Rule>) -> AstImplementationTarget {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::impl_target_struct => AstImplementationTarget::Struct,
        Rule::impl_target_enum => AstImplementationTarget::Enum,
        _ => panic!("{:?}", pair.as_rule()),
    }
}

fn parse_type(pair: Pair<Rule>) -> AstType {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::string => AstType::Extern(parse_string(pair)),
        Rule::identifier => AstType::Local(parse_identifier(pair)),
        _ => panic!("{:?}", pair.as_rule()),
    }
}

fn parse_tags(pair: Pair<Rule>) -> Vec<(String, HashMap<String, String>)> {
    pair.into_inner().map(parse_tag).collect::<Vec<_>>()
}

fn parse_tag(pair: Pair<Rule>) -> (String, HashMap<String, String>) {
    let mut pairs = pair.into_inner();
    let identifier = parse_identifier(pairs.next().unwrap());
    let parameters = if let Some(pair) = pairs.next() {
        pair.into_inner()
            .map(parse_tag_parameter)
            .collect::<HashMap<_, _>>()
    } else {
        Default::default()
    };
    (identifier, parameters)
}

fn parse_tag_parameter(pair: Pair<Rule>) -> (String, String) {
    let mut pairs = pair.into_inner();
    let identifier = parse_identifier(pairs.next().unwrap());
    let value = if let Some(pair) = pairs.next() {
        parse_string(pair)
    } else {
        Default::default()
    };
    (identifier, value)
}

fn parse_code(pair: Pair<Rule>) -> AstCode {
    let pair = pair.into_inner().next().unwrap();
    let mut code = AstCode::default();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::code_chars => {
                code.0.push(AstCodeChunk::Content(pair.as_str().to_owned()));
            }
            Rule::code_op => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::variable => {
                        code.0.push(AstCodeChunk::Variable(parse_variable(pair)));
                    }
                    Rule::code_op_for => {
                        code.0.push(AstCodeChunk::For(parse_code_for(pair)));
                    }
                    _ => panic!("{:?}", pair.as_rule()),
                }
            }
            _ => panic!("{:?}", pair.as_rule()),
        }
    }
    code
}

fn parse_code_for(pair: Pair<Rule>) -> AstCodeFor {
    let mut result = AstCodeFor::default();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::vars => result.variables = parse_variables(pair),
            Rule::code_op_in => result.container = parse_in(pair),
            Rule::where_rules => result.where_rules = parse_where_rules(pair),
            Rule::code => result.code = parse_code(pair),
            _ => panic!("{:?}", pair.as_rule()),
        }
    }
    result
}

fn parse_in(pair: Pair<Rule>) -> AstIn {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::variable => AstIn::Variable(parse_variable(pair)),
        Rule::code_op_in_fields => AstIn::Fields,
        _ => panic!("{:?}", pair.as_rule()),
    }
}

fn parse_where_rules(pair: Pair<Rule>) -> Vec<AstWhereRule> {
    pair.into_inner().map(parse_where_rule).collect::<Vec<_>>()
}

fn parse_where_rule(pair: Pair<Rule>) -> AstWhereRule {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::where_rule_exists => {
            AstWhereRule::Exists(parse_variable(pair.into_inner().next().unwrap()))
        }
        Rule::where_rule_is => AstWhereRule::Is(parse_where_rule_is(pair)),
        Rule::where_rule_impl => AstWhereRule::Impl(parse_where_rule_impl(pair)),
        _ => panic!("{:?}", pair.as_rule()),
    }
}

fn parse_where_rule_is(pair: Pair<Rule>) -> AstWhereRuleIs {
    let mut pairs = pair.into_inner();
    let variable = parse_variable(pairs.next().unwrap());
    let value = parse_string(pairs.next().unwrap());
    AstWhereRuleIs { variable, value }
}

fn parse_where_rule_impl(pair: Pair<Rule>) -> AstWhereRuleImpl {
    let mut pairs = pair.into_inner();
    let container = parse_in(pairs.next().unwrap());
    let implements = pairs
        .next()
        .unwrap()
        .into_inner()
        .map(parse_identifier)
        .collect::<Vec<_>>();
    AstWhereRuleImpl {
        container,
        implements,
    }
}

fn parse_variables(pair: Pair<Rule>) -> Vec<String> {
    pair.into_inner().map(parse_variable).collect::<Vec<_>>()
}

fn parse_string(pair: Pair<Rule>) -> String {
    pair.into_inner().next().unwrap().as_str().to_owned()
}

fn parse_identifier(pair: Pair<Rule>) -> String {
    pair.as_str().to_owned()
}

fn parse_variable(pair: Pair<Rule>) -> String {
    pair.into_inner().next().unwrap().as_str().to_owned()
}
