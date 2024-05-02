//! Type inference mechanism for qcc.
use crate::ast::{Expr, Qast};
use crate::error::{QccErrorKind, Result};
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
                match last_instruction.unwrap().borrow_mut() {
                    Expr::Var(var) => {
                        if *var.get_type() == Type::Bottom {
                            var.set_type(fn_return_type);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
