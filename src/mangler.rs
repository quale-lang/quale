//! Simple Name Mangler
//!
//! This simple mangler uses module name as prefix and underscored with function
//! names.

use crate::ast::{QccCell, Qast, Expr, Ident};
use crate::error::Result;

pub(crate) fn mangle(ast: &mut Qast) -> Result<()> {
    for module in ast.iter_mut() {
        let mod_name = module.as_ref().get_name();
        for function in module.iter_mut() {
            function.set_name(format!("{}_{}", mod_name.clone(), function.get_name()).into());

            for instruction in function.iter_mut() {
                mangle_expr(instruction, mod_name.clone() + "_");
            }
        }
    }

    Ok(())
}

fn mangle_expr(expr: &mut QccCell<Expr>, prefix: Ident) { // TODO: prefix: &str
    match *expr.as_ref().borrow_mut() {
        Expr::BinaryExpr(ref mut lhs, _, ref mut rhs) => {
            mangle_expr(lhs, prefix.clone());
            mangle_expr(rhs, prefix);
        }
        Expr::Let(_, ref mut val) => {
            mangle_expr(val, prefix);
        }
        Expr::FnCall(ref mut f, ref mut args) => {
            for arg in args {
                mangle_expr(arg, prefix.clone());
            }

            f.set_name(prefix + f.get_name());
        }
        _ => {}
    }
}

pub(crate) fn sanitize(identifier: Ident) -> Ident {
    let mut sanitized = String::new();
    for c in identifier.bytes() {
        if c.is_ascii_alphanumeric() {
            sanitized += &(c as char).to_string();
        } else {
            sanitized += "_";
        }
    }
    sanitized
}
