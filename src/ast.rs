use crate::parse::Rule;

pub type Identifier = String;

#[derive(Debug, Clone)]
pub struct ProgramTree {
    pub items: Vec<Item>,
}

impl ProgramTree {
    /// Searches for a function definition in the global scope and returns it if it exists.
    pub fn get_function(&self, ident: impl AsRef<str>) -> Option<&FunctionDef> {
        for item in &self.items {
            if let ItemKind::Function(f @ FunctionDef { name, .. }) = &item.kind
                && name == ident.as_ref()
            {
                return Some(f);
            }
        }

        None
    }
}

/// An item is a top-level definition such as a function, global variable or type definition.
#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Function(FunctionDef),
    Static(VariableDef),
    // TODO add types
}

#[derive(Debug, Clone)]
pub struct VariableDef {
    pub name: Identifier,
    pub ty: Option<Type>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Variable(VariableDef),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
}
impl Expression {
    pub(crate) fn UnaryOp(op: UnaryOp, expr: Box<Expression>) -> Expression {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Value(Value),
    Block(Block),
    Var(Identifier),
    FunctionCall {
        function: Identifier,
        parameters: Vec<Expression>,
    },
    BinOp(Box<Expression>, BinOp, Box<Expression>),
    UnaryOp(UnaryOp, Box<Expression>),
    Conditional {
        condition: Box<Expression>,
        if_path: Box<Expression>,
        else_path: Option<Box<Expression>>,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: Identifier,
    pub params: Vec<FunctionParam>,
    pub block: Block,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: Identifier,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    // This has to be boxed since Expression could be a Block
    pub value: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    Neq,
    Gt,
    Geq,
    Lt,
    Leq,
    And,
    Or,
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone)]
pub struct UnaryOpType {
    pub input: Type,
    pub result: Type,
}

impl UnaryOpType {
    pub fn new(input: Type, result: Type) -> Self {
        Self { input, result }
    }
}

impl UnaryOp {
    pub fn accepted_types(&self) -> Vec<UnaryOpType> {
        match self {
            UnaryOp::Neg => vec![
                UnaryOpType::new(Type::Int, Type::Int),
                UnaryOpType::new(Type::Float, Type::Float),
            ],
            UnaryOp::Not => vec![UnaryOpType::new(Type::Bool, Type::Bool)],
        }
    }

    pub fn from_rule(rule: Rule) -> Option<UnaryOp> {
        let op = match rule {
            Rule::neg => UnaryOp::Neg,
            Rule::not => UnaryOp::Not,
            _ => return None,
        };
        Some(op)
    }
}

#[derive(Debug, Clone)]
pub struct BinOpType {
    pub lhs: Type,
    pub rhs: Type,
    pub result: Type,
}

impl BinOpType {
    pub fn new(lhs: Type, rhs: Type, result: Type) -> Self {
        Self { lhs, rhs, result }
    }
}

impl BinOp {
    pub fn accepted_types(&self) -> Vec<BinOpType> {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Mult | BinOp::Div => {
                vec![
                    BinOpType::new(Type::Int, Type::Int, Type::Int),
                    BinOpType::new(Type::Float, Type::Float, Type::Float),
                ]
            }
            BinOp::Gt | BinOp::Lt | BinOp::Geq | BinOp::Leq => {
                vec![
                    BinOpType::new(Type::Int, Type::Int, Type::Bool),
                    BinOpType::new(Type::Float, Type::Float, Type::Bool),
                ]
            }
            BinOp::And | BinOp::Or => {
                vec![BinOpType::new(Type::Bool, Type::Bool, Type::Bool)]
            }
            BinOp::Eq | BinOp::Neq => {
                let accepted = [Type::Bool, Type::Float, Type::Int, Type::Unit, Type::String];
                accepted
                    .into_iter()
                    .map(|ty| BinOpType::new(ty.clone(), ty, Type::Bool))
                    .collect()
            }
        }
    }

    pub fn from_rule(rule: Rule) -> Option<Self> {
        let op = match rule {
            Rule::add => BinOp::Add,
            Rule::sub => BinOp::Sub,
            Rule::mult => BinOp::Mult,
            Rule::div => BinOp::Div,
            Rule::eq => BinOp::Eq,
            Rule::neq => BinOp::Neq,
            Rule::gt => BinOp::Gt,
            Rule::geq => BinOp::Geq,
            Rule::lt => BinOp::Lt,
            Rule::leq => BinOp::Leq,
            Rule::and => BinOp::And,
            Rule::or => BinOp::Or,
            _ => return None,
        };

        Some(op)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    Unit,
    Int,
    Float,
    String,
    Bool,
}

impl Default for Type {
    fn default() -> Self {
        Self::Unit
    }
}
