//! Simple Name Mangler
//!
//! This simple mangler uses module name as prefix and underscored with function
//! names.

use crate::ast::{Expr, FunctionAST, Ident, ModuleAST, Qast, QccCell};
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

fn mangle_expr(expr: &mut QccCell<Expr>, prefix: Ident) {
    // TODO: prefix: &str
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

/// Replaces all occurences of `fn_name` in instructions with
/// (`mod_name + `_` + `fn_name`).
fn mangle_expr_check(expr: &mut QccCell<Expr>, mod_name: &Ident, fn_name: &Ident) {
    match *expr.as_ref().borrow_mut() {
        Expr::BinaryExpr(ref mut lhs, _, ref mut rhs) => {
            mangle_expr_check(lhs, mod_name, fn_name);
            mangle_expr_check(rhs, mod_name, fn_name);
        }
        Expr::Let(_, ref mut val) => {
            mangle_expr_check(val, mod_name, fn_name);
        }
        Expr::FnCall(ref mut f, ref mut args) => {
            for arg in args {
                mangle_expr_check(arg, mod_name, fn_name);
            }

            if *f.get_name() == *fn_name {
                f.set_name(mod_name.to_owned() + "_" + f.get_name());
            }
        }
        _ => {}
    }
}

/// Takes in a mutable reference to a module and replaces all function call
/// instances with a mangled string, which is calculated from a module name and
/// a function name.
pub(crate) fn mangle_module(module: &mut ModuleAST, mod_name: Ident, fn_name: Ident) -> Result<()> {

    for function in module.iter_mut() {
        for instruction in function.iter_mut() {
            mangle_expr_check(instruction, &mod_name, &fn_name);
        }
    }

    Ok(())
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
