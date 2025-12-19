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
    {:14}\t{:<20}
",
        "-h,--help",
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
        "compiled output",
        "-v,--version",
        "compiler version"
    );
}

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
