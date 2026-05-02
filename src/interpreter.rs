//! A basic interpreter for the language.
//!
//! This interpreter is not meant to be particularly fast or good, but it just has to run the
//! language so that I can have a basic working prototype before I implement LLVM translation.
use std::collections::HashMap;

use crate::parse::{BinOp, Block, Expression, ProgramTree, Statement, UnaryOp, Value};

type BuiltinFn = &'static dyn Fn(&[Value]) -> Value;

#[derive(Default)]
struct ProgramContext {
    variable_stack: Vec<HashMap<String, Value>>,
    builtins: HashMap<String, BuiltinFn>,
}

fn builtin_print(params: &[Value]) -> Value {
    assert_eq!(params.len(), 1);
    let s: &dyn std::fmt::Display = match &params[0] {
        Value::Unit => &"()",
        Value::Int(i) => i,
        Value::Float(f) => f,
        Value::String(s) => s,
        Value::Bool(b) => b,
    };
    println!("{s}");
    Value::Unit
}

impl ProgramContext {
    fn new() -> Self {
        let mut default = Self::default();
        let f: &'static dyn Fn(&[Value]) -> Value = &builtin_print;
        default.builtins = HashMap::from_iter([("print".to_string(), f)]);
        default
    }
}

pub fn interpret(program: &ProgramTree) {
    let mut ctx = ProgramContext::new();
    evaluate_function(program, "main", &[], &mut ctx);
}

fn evaluate_function(
    program: &ProgramTree,
    function_name: &str,
    params: &[Value],
    ctx: &mut ProgramContext,
) -> Value {
    if let Some(function) = program.functions.get(function_name) {
        let mut stack_frame = HashMap::new();

        for (i, val) in params.iter().enumerate() {
            stack_frame.insert(function.params[i].name.clone(), val.clone());
        }

        ctx.variable_stack.push(stack_frame);
        let res = evaluate_block(program, &function.block, ctx);
        ctx.variable_stack.pop();
        res
    } else if let Some(function) = ctx.builtins.get(function_name) {
        function(params)
    } else {
        panic!("Function not found")
    }
}

fn evaluate_block(program: &ProgramTree, block: &Block, ctx: &mut ProgramContext) -> Value {
    for s in &block.statements {
        interpret_statement(program, s, ctx);
    }

    block
        .value
        .as_ref()
        .map(|v| evaluate_expression(program, v, ctx))
        .unwrap_or(Value::Unit)
}

fn interpret_statement(program: &ProgramTree, statement: &Statement, ctx: &mut ProgramContext) {
    match statement {
        Statement::VariableDefinition { name, value, .. } => {
            let value = evaluate_expression(program, value, ctx);
            ctx.variable_stack
                .last_mut()
                .unwrap()
                .insert(name.clone(), value);
        }
        Statement::Expression(expression) => {
            // Eval the expression but don't do anything with the value.
            evaluate_expression(program, expression, ctx);
        }
    }
}

fn evaluate_expression(
    program: &ProgramTree,
    expr: &Expression,
    ctx: &mut ProgramContext,
) -> Value {
    match expr {
        Expression::Value(value) => value.clone(),
        Expression::Block(block) => evaluate_block(program, block, ctx),
        Expression::Var(var) => ctx
            .variable_stack
            .last()
            .unwrap()
            .get(var)
            .expect("Variable not defined")
            .clone(),
        Expression::FunctionCall {
            function,
            parameters,
        } => {
            let parameters = parameters
                .iter()
                .map(|e| evaluate_expression(program, e, ctx))
                .collect::<Vec<_>>();
            evaluate_function(program, function, &parameters, ctx)
        }
        Expression::BinOp(lhs, op, rhs) => {
            let lhs = evaluate_expression(program, lhs, ctx);
            let rhs = evaluate_expression(program, rhs, ctx);
            evaluate_bin_op(lhs, *op, rhs)
        }
        Expression::UnaryOp(op, expr) => {
            let expr = evaluate_expression(program, expr, ctx);
            evaluate_unary_op(*op, expr)
        }
        Expression::Conditional {
            condition,
            if_path,
            else_path,
        } => {
            let Value::Bool(condition) = evaluate_expression(program, condition, ctx) else {
                panic!("if statement condition must evaluate to bool");
            };

            if condition {
                evaluate_expression(program, if_path, ctx)
            } else if let Some(else_path) = else_path.as_deref() {
                evaluate_expression(program, else_path, ctx)
            } else {
                Value::Unit
            }
        }
    }
}

fn evaluate_bin_op(lhs: Value, op: BinOp, rhs: Value) -> Value {
    // This will all be so much nicer when I implement type checking
    match op {
        BinOp::Add => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs + rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs + rhs),
            _ => panic!("Invalid types for add operation"),
        },
        BinOp::Sub => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs - rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs - rhs),
            _ => panic!("Invalid types for add operation"),
        },
        BinOp::Mult => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs * rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs * rhs),
            _ => panic!("Invalid types for add operation"),
        },
        BinOp::Div => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs / rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs / rhs),
            _ => panic!("Invalid types for add operation"),
        },
        BinOp::Eq => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs == rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Bool(lhs == rhs),
            (Value::String(lhs), Value::String(rhs)) => Value::Bool(lhs == rhs),
            (Value::Unit, Value::Unit) => Value::Bool(true),
            (Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs == rhs),
            _ => panic!("Invalid types for eq operation"),
        },
        BinOp::Neq => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs != rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Bool(lhs != rhs),
            (Value::String(lhs), Value::String(rhs)) => Value::Bool(lhs != rhs),
            (Value::Unit, Value::Unit) => Value::Bool(false),
            (Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs != rhs),
            _ => panic!("Invalid types for neq operation"),
        },
        BinOp::Gt => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs > rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Bool(lhs > rhs),
            _ => panic!("Invalid types for gt operation"),
        },
        BinOp::Geq => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs >= rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Bool(lhs >= rhs),
            _ => panic!("Invalid types for geq operation"),
        },
        BinOp::Lt => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs < rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Bool(lhs < rhs),
            _ => panic!("Invalid types for lt operation"),
        },
        BinOp::Leq => match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Bool(lhs <= rhs),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Bool(lhs <= rhs),
            _ => panic!("Invalid types for leq operation"),
        },
        BinOp::And => match (lhs, rhs) {
            (Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs && rhs),
            _ => panic!("Invalid types for and operation"),
        },
        BinOp::Or => match (lhs, rhs) {
            (Value::Bool(lhs), Value::Bool(rhs)) => Value::Bool(lhs || rhs),
            _ => panic!("Invalid types for or operation"),
        },
    }
}

fn evaluate_unary_op(op: UnaryOp, value: Value) -> Value {
    match op {
        UnaryOp::Not => match value {
            Value::Bool(b) => Value::Bool(!b),
            _ => panic!("Invalid type for boolean negation operator"),
        },
        UnaryOp::Neg => match value {
            Value::Int(i) => Value::Int(-i),
            Value::Float(f) => Value::Float(-f),
            _ => panic!("Invalid type for negation operator"),
        },
    }
}
