//! Type inference mechanism for qcc.
use crate::ast::{Expr, LiteralAST, Qast, VarAST};
use crate::error::{QccError, QccErrorKind, Result};
use crate::types::Type;
use std::borrow::{Borrow, BorrowMut};

/// A generic symbol table implementation.
struct SymbolTable<T> {
    table: Vec<T>,
}

impl<T> SymbolTable<T> {
    fn new() -> Self {
        Self { table: vec![] }
    }

    fn push(&mut self, value: T) {
        self.table.push(value);
    }

    fn extend(&mut self, values: Vec<T>) {
        self.table.extend(values);
    }

    fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.table.iter()
    }
}

impl<T> std::fmt::Display for SymbolTable<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.iter() {
            write!(f, "{} ", entry)?;
        }
        writeln!(f, "")
    }
}

/// Type checker
pub(crate) fn checker(ast: &mut Qast) /*-> Result<()>*/ {}

/// Type inference method.
pub(crate) fn infer(ast: &mut Qast) -> Result<()> {
    let mut seen_errors = false;

    for module in ast.iter_mut() {
        // functions but only collect their names and return types.
        // let mut function_symbol_table: Vec<VarAST> = vec![];
        let mut function_table: SymbolTable<VarAST> = SymbolTable::new();
        for function in module.iter() {
            function_table.push(VarAST::new_with_type(
                function.get_name().clone(),
                function.get_loc().clone(),
                function.get_output_type().clone(),
            ));
        }

        for function in module.iter_mut() {
            // parameter symbols
            let mut parameter_table: SymbolTable<VarAST> = SymbolTable::new();
            for param in function.iter_params() {
                parameter_table.push(param.clone());
            }

            // local variables
            let mut local_var_table: SymbolTable<VarAST> = SymbolTable::new();
            for instruction in function.iter() {
                local_var_table.extend(gather_already_typed(instruction));
            }

            // infer local var types
            for instruction in function.iter_mut() {
                let instruction_type = infer_expr(instruction);
                if instruction_type.is_none() || instruction_type == Some(Type::Bottom) {
                    // we couldn't infer all types for expression
                    // see if either symbol table contains any information
                    if let Some(untyped) = infer_from_table(
                        instruction,
                        &parameter_table,
                        &local_var_table,
                        &function_table,
                    ) {
                        seen_errors = true;
                        let err: QccError = QccErrorKind::UnknownType.into();
                        err.report(format!("for `{}` {}", untyped, untyped.location()).as_str());
                    }
                }
            }

            // type check between function return type and the last returned
            // expression
            let fn_return_type = *function.get_output_type();

            let last_instruction = function.iter_mut().last();
            if last_instruction.is_some() {
                let last = last_instruction.unwrap();

                // get last expression's type
                let last_instruction_type = infer_expr(last);

                if fn_return_type == Type::Bottom && last_instruction_type.is_some() {
                    function.set_output_type(last_instruction_type.unwrap());
                } else {
                    if last_instruction_type != Some(fn_return_type) {
                        seen_errors = true;
                        let err: QccError = QccErrorKind::TypeMismatch.into();
                        let msg = format!(
                            "between\n\t`{}` ({}) and `{}` ({})",
                            &last.to_string(),
                            last_instruction_type.unwrap(),
                            function.get_name(),
                            function.get_output_type()
                        );
                        err.report(&msg);
                    }
                }
            }
        }
    }

    if seen_errors {
        return Err(QccErrorKind::TypeError)?;
    } else {
        Ok(())
    }
}

/// Infer type for expression.
fn infer_expr(expr: &mut Box<Expr>) -> Option<Type> {
    match expr.borrow_mut() {
        Expr::Var(var) => {
            if *var.get_type() == Type::Bottom {
                return None;
            } else {
                return Some(*var.get_type());
            }
        }
        Expr::BinaryExpr(lhs, op, rhs) => {
            let lhs_type = infer_expr(lhs)?;
            let rhs_type = infer_expr(rhs)?;

            if lhs_type != rhs_type {
                return None;
            }
            return Some(lhs_type);
        }
        Expr::FnCall(f, args) => {
            if *f.get_output_type() == Type::Bottom && args.len() != 0 {
                // we can only infer input types by matching against args
                for arg in args {
                    let arg_type = infer_expr(arg)?;
                    f.insert_input_type(arg_type);
                }
                // TODO: we cannot infer function return type and it may return
                // a Bottom type.
                return Some(*f.get_output_type());
            } else {
                return Some(*f.get_output_type());
            }
        }
        Expr::Let(var, val) => {
            // val is an expression and it must have the same type as var
            if *var.get_type() == Type::Bottom {
                // we need to type check from expression first
                let rhs_type = infer_expr(val)?;
                var.set_type(rhs_type);
                return Some(rhs_type);
            } else {
                let lhs_type = *var.get_type();
                let rhs_type = infer_expr(val)?;
                if lhs_type != rhs_type {
                    return None;
                }
                return Some(lhs_type);
            }
        }
        Expr::Literal(lit) => {
            return match lit.as_ref() {
                LiteralAST::Lit_Digit(_) => Some(Type::F64),
                LiteralAST::Lit_Str(_) => Some(Type::Bottom),
            };
        }
    }
    Some(Type::Bottom)
}

/// Given an expression gather all variable references which have already been
/// typed and return them.
fn gather_already_typed(expr: &Box<Expr>) -> Vec<VarAST> {
    let mut symbol_table = vec![];
    match expr.as_ref() {
        Expr::Var(var) => {
            if var.is_typed() {
                symbol_table.push(var.clone());
            }
        }
        Expr::BinaryExpr(lhs, op, rhs) => {
            let lhs_symbols = gather_already_typed(lhs);
            symbol_table.extend(lhs_symbols);

            let rhs_symbols = gather_already_typed(rhs);
            symbol_table.extend(rhs_symbols);
        }
        Expr::FnCall(f, args) => {
            for arg in args {
                let arg_table = gather_already_typed(arg);
                symbol_table.extend(arg_table);
            }
        }
        Expr::Let(var, val) => {
            if var.is_typed() {
                symbol_table.push(var.clone());
            }
            let val_table = gather_already_typed(val);
            symbol_table.extend(val_table);
        }
        _ => {}
    }
    symbol_table
}

/// Infer types for each part of expression from symbol tables. If some
/// expression cannot be typed, because no information was found in symbol
/// tables, then return that expression. Otherwise if complete expression is
/// typed then return None.
fn infer_from_table<'a>(
    expr: &'a mut Box<Expr>,
    param_st: &SymbolTable<VarAST>,
    local_st: &SymbolTable<VarAST>,
    function_st: &SymbolTable<VarAST>,
) -> Option<&'a VarAST> {
    match expr.as_mut().borrow_mut() {
        Expr::Var(var) => {
            let mut param_type = Type::Bottom;
            let mut local_type = Type::Bottom;
            for param in param_st.iter() {
                if param.name() == var.name() && param.is_typed()
                /*trivial*/
                {
                    param_type = *param.get_type();
                }
            }
            for local in local_st.iter() {
                if local.name() == var.name() && local.is_typed() {
                    local_type = *local.get_type();
                }
            }
            if param_type == local_type && param_type == Type::Bottom {
                // couldn't find any type information
                return Some(var);
            }
            if param_type != Type::Bottom {
                var.set_type(param_type);
            } else if local_type != Type::Bottom {
                var.set_type(local_type);
            }
            None
        }
        Expr::BinaryExpr(lhs, op, rhs) => {
            let lhs_info = infer_from_table(lhs, param_st, local_st, function_st);
            if lhs_info.is_some() {
                return lhs_info;
            }
            let rhs_info = infer_from_table(rhs, param_st, local_st, function_st);
            if rhs_info.is_some() {
                return rhs_info;
            }
            None
        }
        Expr::FnCall(f, args) => {
            for arg in args {
                let info = infer_from_table(arg, param_st, local_st, function_st);
                if info.is_some() {
                    return info;
                }
            }
            for func in function_st.iter() {
                if func.name() == f.get_name() && func.is_typed() {
                    f.set_output_type(*func.get_type());
                    break;
                }
            }
            None
        }
        Expr::Let(var, val) => {
            let rhs_info = infer_from_table(val, param_st, local_st, function_st);
            var.set_type(val.get_type());
            None
        }
        _ => None,
    }
}

/// Given an expression return a vector of all variable references irrespective
/// of whether they are typed or not.
fn gather_all_vars(expr: &mut Box<Expr>) -> Vec<&mut VarAST> {
    vec![]
}
