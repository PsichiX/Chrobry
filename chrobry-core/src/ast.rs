use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct AstParser;

#[derive(Debug, Default, Clone)]
pub struct Ast {
    pub imports: Vec<String>,
    pub injects: Vec<AstCode>,
    pub externs: Vec<AstExtern>,
    pub structs: Vec<AstStruct>,
    pub enums: Vec<AstEnum>,
    pub implementations: Vec<AstImplementation>,
}

// impl Ast {
//     pub fn merge_with(&mut self, ast: &Ast) {
//
//     }
// }

#[derive(Debug, Default, Clone)]
pub struct AstExtern {
    pub types: Vec<String>,
    pub implementations: Vec<(String, AstCode)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AstType {
    None,
    Extern(String),
    Local(String),
}

impl Default for AstType {
    fn default() -> Self {
        Self::None
    }
}

impl ToString for AstType {
    fn to_string(&self) -> String {
        match self {
            AstType::None => "".to_owned(),
            AstType::Extern(name) => name.to_owned(),
            AstType::Local(name) => name.to_owned(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AstStruct {
    pub tags: Vec<(String, HashMap<String, String>)>,
    pub name: String,
    pub fields: Vec<(String, AstType)>,
}

#[derive(Debug, Default, Clone)]
pub struct AstEnum {
    pub tags: Vec<(String, HashMap<String, String>)>,
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct AstImplementation {
    pub target: AstImplementationTarget,
    pub name: String,
    pub where_rules: Vec<AstWhereRule>,
    pub code: AstCode,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AstImplementationTarget {
    All,
    Struct,
    Enum,
}

impl AstImplementationTarget {
    pub fn is_valid(&self, other: AstImplementationTarget) -> bool {
        match (self, other) {
            (Self::All, _) | (Self::Struct, Self::Struct) | (Self::Enum, Self::Enum) => true,
            _ => false,
        }
    }
}

impl Default for AstImplementationTarget {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone)]
pub enum AstWhereRule {
    None,
    Exists(String),
    Is(AstWhereRuleIs),
    Impl(AstWhereRuleImpl),
}

impl Default for AstWhereRule {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct AstWhereRuleIs {
    pub variable: String,
    pub value: String,
}

#[derive(Debug, Default, Clone)]
pub struct AstWhereRuleImpl {
    pub container: AstIn,
    pub implements: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct AstCode(pub Vec<AstCodeChunk>);

#[derive(Debug, Clone)]
pub enum AstCodeChunk {
    None,
    Content(String),
    Variable(String),
    For(AstCodeFor),
}

impl Default for AstCodeChunk {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct AstCodeMatch {
    pub variables: Vec<String>,
    pub container: AstIn,
    pub where_rules: Vec<AstWhereRule>,
    pub code: AstCode,
}

#[derive(Debug, Default, Clone)]
pub struct AstCodeFor {
    pub variables: Vec<String>,
    pub container: AstIn,
    pub where_rules: Vec<AstWhereRule>,
    pub code: AstCode,
}

#[derive(Debug, Clone)]
pub enum AstIn {
    None,
    Fields,
    Variable(String),
}

impl Default for AstIn {
    fn default() -> Self {
        Self::None
    }
}
