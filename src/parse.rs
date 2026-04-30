//! This module defines the abstract syntax tree (AST) of the language and provides a function to
//! parse the AST from the source code ([parse]).

// Most of the functions in this module are of the form parse_RULE which simply transforms the
// concrete syntax tree given to us by Pest's parser and turns it into the abstract syntax tree.

use std::collections::HashMap;

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

pub type Identifier = String;

#[derive(Debug, Clone)]
pub struct ProgramTree {
    pub functions: HashMap<Identifier, FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub block: Block,
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
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Value),
    Block(Block),
    Var(Identifier),
    FunctionCall {
        function: Identifier,
        parameters: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    VariableDefinition { name: Identifier, value: Expression },
    Expression(Expression),
}

type ParseResult<T> = Result<T, pest::error::Error<Rule>>;

/// Struct
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LangParser;

/// Parses the entire text of a program file and converts it into an abstract syntax tree
/// representing the declarations in the file.
pub fn parse(source: &str) -> ParseResult<ProgramTree> {
    let parse = LangParser::parse(Rule::program, source)?.next().unwrap();

    let functions = parse
        .into_inner()
        .filter_map(|parsed| match parsed.as_rule() {
            Rule::function_def => Some(parse_function_def(parsed).map(|f| (f.name.clone(), f))),
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect::<ParseResult<_>>()?;

    Ok(ProgramTree { functions })
}

fn parse_function_def(input: Pair<'_, Rule>) -> ParseResult<FunctionDefinition> {
    let mut input = input.into_inner(); // { "fun" ~ ident ~ "(" ~ ")" ~ block }
    let name = input.next().unwrap();
    let block = input.next().unwrap();

    let name = match name.as_rule() {
        Rule::ident => name.to_string(),
        _ => unreachable!(),
    };

    let block = parse_block(block)?;

    Ok(FunctionDefinition { name, block })
}

fn parse_block(input: Pair<'_, Rule>) -> ParseResult<Block> {
    let mut input = input.into_inner(); // { "{" ~ statement_list ~ "}" }
    let statements = input.next().unwrap().into_inner(); // { (statement ~ ";")* ~ expression? }
    let mut result = Block {
        statements: vec![],
        value: None,
    };

    for rule in statements {
        match rule.as_rule() {
            Rule::statement => {
                result.statements.push(parse_statement(rule)?);
            }
            Rule::expression => {
                result.value = Some(Box::new(parse_expression(rule)?));
            }
            _ => unreachable!(),
        }
    }

    Ok(result)
}

fn parse_statement(input: Pair<'_, Rule>) -> ParseResult<Statement> {
    let mut input = input.into_inner(); // { variable_def | expression }
    let rule = input.next().unwrap();

    match rule.as_rule() {
        Rule::variable_def => {
            let mut input = rule.into_inner();
            let name = input.next().unwrap();
            let value = input.next().unwrap();

            Ok(Statement::VariableDefinition {
                name: name.to_string(),
                value: parse_expression(value)?,
            })
        }
        Rule::expression => Ok(Statement::Expression(parse_expression(rule)?)),
        _ => unreachable!(),
    }
}

fn parse_expression(input: Pair<'_, Rule>) -> ParseResult<Expression> {
    let input = input.into_inner().next().unwrap();
    Ok(match input.as_rule() {
        Rule::unit => Expression::Value(Value::Unit),
        Rule::number => {
            let mut input = input.into_inner();
            let int_part = input.next().unwrap();
            let decimal_part = input.next();
            let exponential_part = input.next();

            if matches!((decimal_part, exponential_part), (None, None)) {
                assert!(matches!(int_part.as_rule(), Rule::int));
                Expression::Value(Value::Int(int_part.as_str().parse().unwrap()))
            } else {
                Expression::Value(Value::Float(input.as_str().parse().unwrap()))
            }
        }
        Rule::string => {
            let input = input.into_inner().next().unwrap();

            // TODO handle escape chars
            Expression::Value(Value::String(input.to_string()))
        }
        Rule::function_call => {
            let mut input = input.into_inner();
            let name = input.next().unwrap();
            let mut parameters = Vec::new();

            for expr in input {
                match expr.as_rule() {
                    Rule::expression => parameters.push(parse_expression(expr)?),
                    _ => unreachable!(),
                }
            }

            Expression::FunctionCall {
                function: name.to_string(),
                parameters,
            }
        }
        Rule::ident => Expression::Var(input.to_string()),
        Rule::block => Expression::Block(parse_block(input)?),
        _ => unreachable!(),
    })
}
