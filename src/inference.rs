//! Type inference mechanism for qcc.
use crate::ast::{Expr, LiteralAST, Qast, VarAST};
use crate::error::{QccError, QccErrorKind, Result};
use crate::types::Type;
use std::borrow::{Borrow, BorrowMut};

/// Type checker
pub(crate) fn checker(ast: &mut Qast) /*-> Result<()>*/ {}

/// Type inference method.
pub(crate) fn infer(ast: &mut Qast) -> Result<()> {
    for module in ast.iter_mut() {
        for function in module.iter_mut() {
            // parameter symbols
            let mut param_symbol_table = vec![];
            for param in function.iter_params() {
                param_symbol_table.push(param.clone());
            }

            // local variables
            let mut local_symbol_table: Vec<VarAST> = vec![];
            for instruction in function.iter() {
                local_symbol_table.extend(gather_already_typed(instruction));
            }

            print!("Fn {} = ", function.get_name());
            for param in &param_symbol_table {
                print!("{} | ", param);
            }
            for local in &local_symbol_table {
                print!("{} | ", local);
            }
            println!();

            // infer local var types
            for instruction in function.iter_mut() {
                let instruction_type = infer_expr(instruction);
                if instruction_type.is_none() {
                    // we couldn't infer all types for expression
                    // see if either param_symbol_table or local_symbol_table
                    // contain any information
                    if infer_from_table(instruction, &param_symbol_table, &local_symbol_table)
                        .is_some()
                    {
                        // some vars were remained untyped, TODO print them
                        return Err(QccErrorKind::UnknownType)?;
                    }
                }
            }

            let fn_return_type = *function.get_output_type();

            let last_instruction = function.iter_mut().last();
            if last_instruction.is_some() {
                let last = last_instruction.unwrap();
                let last_instruction_type = infer_expr(last);
                if fn_return_type == Type::Bottom && last_instruction_type.is_some() {
                    function.set_output_type(last_instruction_type.unwrap());
                } else {
                    if last_instruction_type != Some(fn_return_type) {
                        let err: QccError = QccErrorKind::TypeMismatch.into();
                        err.report(&last.to_string());

                        return Err(QccErrorKind::TypeMismatch)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Infer type for expression.
fn infer_expr(expr: &mut Box<Expr>) -> Option<Type> {
    match expr.borrow_mut() {
        Expr::Var(var) => {
            if *var.get_type() == Type::Bottom {
                // let err: QccError = QccErrorKind::UnknownType.into();
                // err.report(format!("for `{}` {}", &var.to_string(), &var.location()).as_str());
                // return Err(QccErrorKind::TypeError)?;
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
                    // return Err(QccErrorKind::TypeMismatch)?;
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
/// tables, then return that expression.
fn infer_from_table<'a>(
    expr: &'a mut Box<Expr>,
    param_st: &Vec<VarAST>,
    local_st: &Vec<VarAST>,
) -> Option<&'a VarAST> {
    match expr.as_mut().borrow_mut() {
        Expr::Var(var) => {
            let mut param_type = Type::Bottom;
            let mut local_type = Type::Bottom;
            for param in param_st {
                if param.name() == var.name() && param.is_typed()
                /*trivial*/
                {
                    param_type = *param.get_type();
                }
            }
            for local in local_st {
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
            let lhs_info = infer_from_table(lhs, param_st, local_st);
            if lhs_info.is_some() {
                return lhs_info;
            }
            let rhs_info = infer_from_table(rhs, param_st, local_st);
            if rhs_info.is_some() {
                return rhs_info;
            }
            None
        }
        Expr::FnCall(f, args) => None,
        Expr::Let(var, val) => None,
        _ => None,
    }
}

/// Given an expression return a vector of all variable references irrespective
/// of whether they are typed or not.
fn gather_all_vars(expr: &mut Box<Expr>) -> Vec<&mut VarAST> {
    vec![]
}
