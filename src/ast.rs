//! Abstract syntax tree for the programming language

pub type Identifier = String;

#[derive(Debug, Clone)]
pub struct ProgramTree {
    pub functions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: Identifier,
}
