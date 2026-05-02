use std::collections::HashMap;

use log::debug;

use crate::parse::{BinOpType, Block, Expression, ProgramTree, Statement, Type, UnaryOpType};

static BUILTINS: &[&str] = &["print"];

#[derive(Default)]
struct TypeChecker {
    scope_stack: Vec<HashMap<String, Type>>,
}

impl TypeChecker {
    fn new() -> Self {
        Self::default()
    }

    fn typecheck(&mut self, program: &ProgramTree) -> bool {
        program
            .functions
            .keys()
            .all(|name| self.typecheck_function(program, name))
    }

    fn typecheck_function(&mut self, program: &ProgramTree, name: &str) -> bool {
        debug!("typechecking function \"{name}\"");

        let function = program.functions.get(name).unwrap();
        let scope = HashMap::from_iter(
            function
                .params
                .iter()
                .cloned()
                .map(|param| (param.name, param.ty)),
        );
        self.scope_stack.push(scope);

        let Some(res_type) = self.resolve_block(program, &function.block) else {
            return false;
        };

        self.scope_stack.pop();
        res_type == function.return_type.clone().unwrap_or_default()
    }

    fn resolve_block(&mut self, program: &ProgramTree, block: &Block) -> Option<Type> {
        debug!("typechecking block");

        // A block is a new scope
        self.scope_stack.push(HashMap::new());

        for statement in &block.statements {
            if !self.typecheck_statement(program, statement) {
                return None;
            }
        }

        let res = match block.value.as_deref() {
            Some(expr) => self.resolve_expression(program, expr),
            _ => Some(Type::Unit),
        };

        self.scope_stack.pop();

        res
    }

    fn typecheck_statement(&mut self, program: &ProgramTree, statement: &Statement) -> bool {
        match statement {
            Statement::VariableDefinition { name, ty, value } => {
                debug!("typechecking definition of variable \"{name}\"");
                let Some(value_type) = self.resolve_expression(program, value) else {
                    return false;
                };

                let ty = ty
                    .as_ref()
                    .expect("Type inference hasn't been implemented yet")
                    .clone();

                if ty != value_type {
                    debug!("value assigned to variable \"{name}\" did not match expected type");
                    return false;
                }

                self.scope_stack
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), ty);
                true
            }
            Statement::Expression(expression) => {
                // If the expression is treated as a statement, its return value is discarded and
                // thus we only need to check that it has a valid type
                self.resolve_expression(program, expression).is_some()
            }
        }
    }

    fn resolve_expression(&mut self, program: &ProgramTree, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Value(value) => Some(value.ty()),
            Expression::Block(block) => self.resolve_block(program, block),
            Expression::Var(name) => {
                debug!("typechecking reference to variable \"{name}\"");
                for scope in &self.scope_stack {
                    if let res @ Some(_) = scope.get(name) {
                        return res.cloned();
                    }
                }

                debug!("variable was not in scope.");
                None
            }
            Expression::FunctionCall {
                function,
                parameters,
            } => {
                debug!("typechecking call to function \"{function}\"");
                // builtin functions get handled separately
                if BUILTINS.contains(&function.as_str()) {
                    return self.resolve_builtin(program, function, parameters);
                }

                let func_def = program.functions.get(function)?;

                let parameters_correct = parameters
                    .iter()
                    .zip(func_def.params.iter())
                    .enumerate()
                    .all(|(i, (param, expected))| {
                        let Some(ty) = self.resolve_expression(program, param) else {
                            debug!("function parameter #{i} didn't typecheck");
                            return false;
                        };

                        ty == expected.ty
                    });

                parameters_correct.then_some(func_def.return_type.clone().unwrap_or_default())
            }
            Expression::BinOp(lhs, bin_op, rhs) => {
                let op_types = bin_op.accepted_types();
                let lhs_type = self.resolve_expression(program, lhs)?;
                let rhs_type = self.resolve_expression(program, rhs)?;

                for BinOpType { lhs, rhs, result } in &op_types {
                    if *lhs == lhs_type && *rhs == rhs_type {
                        return Some(result.clone());
                    }
                }

                None
            }
            Expression::UnaryOp(unary_op, argument) => {
                let op_types = unary_op.accepted_types();
                let input_type = self.resolve_expression(program, argument)?;

                for UnaryOpType { input, result } in &op_types {
                    if *input == input_type {
                        return Some(result.clone());
                    }
                }

                None
            }
            Expression::Conditional {
                condition,
                if_path,
                else_path,
            } => {
                let if_type = self.resolve_expression(program, if_path)?;
                let else_type = match else_path {
                    Some(else_path) => self.resolve_expression(program, else_path)?,
                    None => Type::Unit,
                };
                let typechecks = self.resolve_expression(program, condition)? == Type::Bool
                    && if_type == else_type;

                typechecks.then_some(if_type)
            }
        }
    }

    fn resolve_builtin(
        &mut self,
        program: &ProgramTree,
        name: &str,
        params: &[Expression],
    ) -> Option<Type> {
        match name {
            "print" => {
                if params.len() == 1 {
                    let ty = self.resolve_expression(program, &params[0])?;

                    matches!(ty, Type::Int | Type::String | Type::Float | Type::Bool)
                        .then_some(Type::Unit)
                } else {
                    debug!("incorrect number of arguments given to print function");
                    None
                }
            }
            _ => todo!("Builtin function \"{name}\" needs to have typechecking implemented"),
        }
    }
}

pub fn typecheck(program: &ProgramTree) -> bool {
    TypeChecker::new().typecheck(program)
}
