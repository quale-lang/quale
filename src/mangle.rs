/// Simple Name Mangler
///
/// This simple mangler uses module name as prefix and dollar'ed with function
/// names.
use crate::ast::{Expr, FunctionAST, Ident, ModuleAST, Qast, QccCell};
use crate::error::Result;

pub(crate) fn mangle(ast: &mut Qast) -> Result<()> {
    for mut module in ast {
        let mod_name = module.get_name();
        for mut function in &mut *module {
            let fn_name = function.get_name().clone();
            function.set_name(format!("{}${}", mod_name.clone(), fn_name).into());

            for instruction in &mut *function {
                mangle_expr(instruction, mod_name.clone() + "$");
            }
        }
    }

    Ok(())
}

pub(crate) fn mangle_expr(expr: &mut QccCell<Expr>, prefix: Ident) {
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
        Expr::Tensor(ref mut exprs) => {
            for expr in exprs {
                mangle_expr(expr, prefix.clone());
            }
        }
        Expr::Conditional(ref mut c, ref mut t, ref mut f) => {
            mangle_expr(c, prefix.clone());
            for expr in t {
                mangle_expr(expr, prefix.clone());
            }
            for expr in f {
                mangle_expr(expr, prefix.clone());
            }
        }
        Expr::Var(_) => {}
        Expr::Literal(_) => {}
    }
}

pub(crate) fn mangle_fns(expr: &mut QccCell<Expr>, module_name: &String, functions: &Vec<String>) {
    match *expr.as_ref().borrow_mut() {
        Expr::BinaryExpr(ref mut lhs, _, ref mut rhs) => {
            mangle_fns(lhs, module_name, functions);
            mangle_fns(rhs, module_name, functions);
        }
        Expr::Let(ref mut var, ref mut val) => {
            mangle_fns(val, module_name, functions);
        }
        Expr::FnCall(ref mut f, ref mut args) => {
            for arg in args {
                mangle_fns(arg, module_name, functions);
            }

            let fn_name = f.get_name();
            if let Some(x) = functions.iter().find(|&f| f == fn_name) {
                if !x.contains('$') {
                    f.set_name(module_name.clone() + "$" + fn_name);
                }
            }
        }
        Expr::Tensor(ref mut exprs) => {
            for expr in exprs {
                mangle_fns(expr, module_name, functions);
            }
        }
        Expr::Conditional(ref mut c, ref mut t, ref mut f) => {
            mangle_fns(c, module_name, functions);
            for expr in t {
                mangle_fns(expr, module_name, functions);
            }
            for expr in f {
                mangle_fns(expr, module_name, functions);
            }
        }
        Expr::Var(_) => {}
        Expr::Literal(_) => {}
    }
}

/// Replaces all occurences of `fn_name` in instructions with
/// (`mod_name + `$` + `fn_name`).
fn mangle_expr_check(expr: &mut QccCell<Expr>, mod_name: &Ident, fn_name: &Ident) {
    // TODO: Support tensors
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
                f.set_name(mod_name.to_owned() + "$" + f.get_name());
            }
        }
        Expr::Tensor(ref mut exprs) => {
            for expr in exprs {
                mangle_expr_check(expr, mod_name, fn_name);
            }
        }
        Expr::Conditional(ref mut c, ref mut t, ref mut f) => {
            mangle_expr_check(c, mod_name, fn_name);
            for expr in t {
                mangle_expr_check(expr, mod_name, fn_name);
            }
            for expr in f {
                mangle_expr_check(expr, mod_name, fn_name);
            }
        }
        Expr::Var(_) => {}
        Expr::Literal(_) => {}
    }
}

/// Takes in a mutable reference to a module and replaces all function call
/// instances with a mangled string, which is calculated from a module name and
/// a function name.
pub(crate) fn mangle_module(module: &mut ModuleAST, mod_name: Ident, fn_name: Ident) -> Result<()> {
    for mut function in module {
        for instruction in &mut *function {
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
            sanitized += "_"; // TODO: Using '$'
        }
    }
    sanitized
}
