//! Utils module contains help documentation.
use crate::ast::{Expr, FunctionAST, Ident, ModuleAST, Qast, QccCell};
use crate::error::Result;

/// It takes an expression and a slice of expressions, and validates if atleast
/// one of the predicates match to the given expression.
#[macro_export]
macro_rules! assert_eq_any {
    ($left:expr, $preds:expr $(,)?) => {{
        let mut result = false;
        for pred in $preds {
            result |= $left == pred.into();
        }
        assert!(result);
    }};
}

/// It takes a predicate and a slice of expressions, and validates if every
/// member in the slice matches against the given predicate.
#[macro_export]
macro_rules! assert_eq_all {
    ($left:expr, $preds:expr $(,)?) => {{
        let mut result = true;
        for pred in $preds {
            result &= $left == pred.into();
        }
        assert!(result);
    }};
}

pub(crate) fn usage() {
    print!(
        "usage: qcc [options] <quale-file>
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
    {:14}\t{:<20}
",
        "--help",
        "show this page",
        "--print-ast",
        "print AST",
        "--print-ast-only",
        "print AST without translating to assemmbly",
        "--print-qasm",
        "print OpenQASM IR",
        "--analyze",
        "run static analyzer",
        "-O0",
        "disable optimizations (NA)",
        "-O1",
        "enable first-level optimizations (NA)",
        "-Og",
        "enable all optimizations (NA)",
        "-d,--debug",
        "run compiler in debug-mode",
        "-o",
        "compiled output"
    );
}

/// Simple Name Mangler
///
/// This simple mangler uses module name as prefix and dollar'ed with function
/// names.
pub(crate) fn mangle(ast: &mut Qast) -> Result<()> {
    for mut module in ast {
        let mod_name = module.get_name();
        for mut function in &mut *module {
            let fn_name = function.get_name().clone();
            function.set_name(format!("{}_{}", mod_name.clone(), fn_name).into());

            for instruction in &mut *function {
                mangle_expr(instruction, mod_name.clone() + "$");
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
                f.set_name(mod_name.to_owned() + "$" + f.get_name());
            }
        }
        _ => {}
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
            sanitized += "_";
        }
    }
    sanitized
}

/// RefSet is a wrapper over HashSet but with no Eq trait. Equality is assumed
/// to be different because it only stores references.

#[cfg(test)]
mod tests {
    #[test]
    fn check_assert_eq_any() {
        assert_eq_any!(true, [false, true]);
        assert_eq_any!(false, [true, false]);
    }

    #[test]
    fn check_assert_eq_all() {
        assert_eq_all!(true, [true, true]);
        assert_eq_all!(false, [false, false]);
    }
}
