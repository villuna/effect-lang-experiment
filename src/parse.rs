use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::ast::{FunctionDefinition, ProgramTree};

type ParseResult<T> = Result<T, pest::error::Error<Rule>>;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LangParser;

pub fn parse(source: &str) -> ParseResult<ProgramTree> {
    let parse = LangParser::parse(Rule::program, source)?.next().unwrap();

    let functions = parse
        .into_inner()
        .filter_map(|parsed| match parsed.as_rule() {
            Rule::function_decl => Some(parse_function_def(parsed)),
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect::<ParseResult<Vec<_>>>()?;

    Ok(ProgramTree { functions })
}

fn parse_function_def(parsed: Pair<'_, Rule>) -> ParseResult<FunctionDefinition> {
    let mut inner = parsed.into_inner();
    let ident = inner.next().unwrap();

    Ok(match ident.as_rule() {
        Rule::ident => FunctionDefinition {
            name: ident.to_string(),
        },
        _ => unreachable!(),
    })
}
