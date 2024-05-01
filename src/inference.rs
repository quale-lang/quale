//! Type inference mechanism for qcc.
use crate::ast::{Expr, Qast};
use crate::error::{QccErrorKind, Result};
use crate::types::Type;

/// Type inference method.
pub(crate) fn infer(ast: &mut Qast) -> Result<()> {
    for module in ast.iter() {
        for function in module.iter() {
            if *function.get_output_type() == Type::Bottom {
                // println!("{function}");
            }
            for param_type in function.get_input_type() {
                if *param_type == Type::Bottom {
                    // println!("{}", function.get_name());
                }
            }

            for instruction in function.iter() {
                match &**instruction {
                    Expr::Var(v) => {
                        if *v.get_type() == Type::Bottom {
                            println!("Untyped: {}", v);
                            println!("{}", function);
                        }
                        // i want to mutate type
                    }
                    Expr::BinaryExpr(lhs, op, rhs) => {}
                    Expr::FnCall(f, args) => {}
                    Expr::Let(var, val) => {}
                    Expr::Literal(lit) => {}
                }
            }
        }
    }

    Ok(())
}
