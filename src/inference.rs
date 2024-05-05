//! Type inference mechanism for qcc.
use crate::ast::{Expr, LiteralAST, Qast};
use crate::error::{QccError, QccErrorKind, Result};
use crate::types::Type;
use std::borrow::{Borrow, BorrowMut};

/// Type checker
pub(crate) fn checker(ast: &mut Qast) /*-> Result<()>*/ {}

/// Type inference method.
pub(crate) fn infer(ast: &mut Qast) -> Result<()> {
    for module in ast.iter_mut() {
        for function in module.iter_mut() {
            let fn_return_type = *function.get_output_type();

            let last_instruction = function.iter_mut().last();
            if last_instruction.is_some() {
                let last_instruction_type = infer_expr(last_instruction.unwrap())?;
                if fn_return_type == Type::Bottom {
                    function.set_output_type(last_instruction_type);
                } else {
                    if last_instruction_type != fn_return_type {
                        return Err(QccErrorKind::TypeMismatch)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Infer type for expression.
fn infer_expr(expr: &mut Box<Expr>) -> Result<Type> {
    match expr.borrow_mut() {
        Expr::Var(var) => {
            if *var.get_type() == Type::Bottom {
                let err: QccError = QccErrorKind::UnknownType.into();
                err.report(format!("for `{}` {}", &var.to_string(), &var.location()).as_str());
                return Err(QccErrorKind::TypeError)?;
            } else {
                return Ok(*var.get_type());
            }
        }
        Expr::BinaryExpr(lhs, op, rhs) => {
            let lhs_type = infer_expr(lhs)?;
            let rhs_type = infer_expr(rhs)?;
            if lhs_type != rhs_type {
                return Err(QccErrorKind::TypeMismatch)?;
            }
            return Ok(lhs_type);
        }
        Expr::FnCall(f, args) => {
            if *f.get_output_type() == Type::Bottom && args.len() != 0 {
                // we can only infer input types by matching against args
                for arg in args {
                    let arg_type = infer_expr(arg)?;
                    f.insert_input_type(arg_type);
                }
                return Ok(*f.get_output_type());
            } else {
                return Ok(*f.get_output_type());
            }
        }
        Expr::Let(var, val) => {
            // val is an expression and it must have the same type as var
            if *var.get_type() == Type::Bottom {
                // we need to type check from expression first
                let rhs_type = infer_expr(val)?;
                var.set_type(rhs_type);
                return Ok(rhs_type);
            } else {
                let lhs_type = *var.get_type();
                let rhs_type = infer_expr(val)?;
                if lhs_type != rhs_type {
                    return Err(QccErrorKind::TypeMismatch)?;
                }
                return Ok(lhs_type);
            }
        }
        Expr::Literal(lit) => {
            return match lit.as_ref() {
                LiteralAST::Lit_Digit(_) => Ok(Type::F64),
                LiteralAST::Lit_Str(_) => Ok(Type::Bottom),
            };
        }
    }
    Ok(Type::Bottom)
}
